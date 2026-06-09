#!/usr/bin/env python3
"""Extract a Vabi Elements ``.vp`` project into our ISSO 51 project-JSON format.

The Vabi ``.vp`` file is a ZIP archive containing ``Elements.sqlite3`` (an
NHibernate table-per-subclass SQLite database). This script reads the inputs
(rooms, climate, design temperatures, geometry, constructions) and emits a JSON
document conforming to ``schemas/v1/project.schema.json`` so it can be fed to the
ISSO 51 calculation engine.

Pure standard library: ``zipfile``, ``sqlite3``, ``json``, ``argparse``,
``pathlib``, ``logging``. No third-party dependencies.

Mapping notes (see ``README.md`` for the full account):

* The SQL join chains mirror the authoritative Rust importer in
  ``crates/isso51-core/src/import/vabi/mapper.rs``.
* **Geometry** (face area, slope, boundary type) is read per room via
  ``Room -> MainFace -> CellFace -> BuildingPart -> Face -> FaceGeometryEngine``.
  This is the reliable source and is taken for *every* face of the room, not only
  the ``HasConstruction = 1`` faces (the Rust importer's filter drops ~94% of the
  envelope in the ISSO 51 examples).
* **U-values**: the example ``.vp`` files store a 9-entry construction *palette*
  but leave ``BuildingPart.ConstructionID`` NULL for almost every face (Vabi
  resolves the per-face construction from the architectural template at calc
  time). When a face has an explicit ``ConstructionID`` we use it; otherwise we
  fall back to a *type-based palette lookup* (Floor/Wall/Roof/Window/Door) and
  emit a WARNING. Faces for which no U-value can be resolved at all get a sentinel
  value and a WARNING.
"""

from __future__ import annotations

import argparse
import json
import logging
import sqlite3
import tempfile
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

logger = logging.getLogger("extract_vp")

# --- Constants (no magic values) -------------------------------------------

ELEMENTS_DB_NAME = "Elements.sqlite3"
DEFAULT_THETA_E_C = -10.0
DEFAULT_ROOM_HEIGHT_M = 2.6
FALLBACK_ROOM_HEIGHT_M = 2.7
DEFAULT_QV10_DM3S = 100.0
SENTINEL_U_VALUE = 2.5  # used + WARNING when no U-value can be resolved
TRANSPARENT_FALLBACK_U = 2.5

# ISO 6946 surface resistances by slope band (R_si, R_se) in m^2K/W.
R_FLOOR = (0.17, 0.04)
R_ROOF = (0.10, 0.04)
R_WALL = (0.13, 0.04)
SLOPE_FLOOR_MAX_DEG = 30.0
SLOPE_ROOF_MIN_DEG = 150.0

# Vabi BoundaryConditions.Type -> our schema boundary_type.
BOUNDARY_TYPE_MAP = {
    "OutsideAir": "exterior",
    "Ground": "ground",
    "CrawlSpace": "ground",
    "AdjacentRoom": "adjacent_room",
    "InternalSpace": "adjacent_room",
    "AdjacentBuilding": "adjacent_building",
    "OtherBuilding": "adjacent_building",
    "UnconditionedSpace": "unheated_space",
}

# Vabi BuildingPartType -> our schema vertical_position.
VERTICAL_POSITION_MAP = {
    "Floor": "floor",
    "Roof": "ceiling",
    "FlatRoof": "ceiling",
    "SlopingRoof": "ceiling",
    "Ceiling": "ceiling",
}

# Vabi ConstructionData.Type that are transparent (window/door).
TRANSPARENT_TYPES = {"Window", "Door"}

# Vabi room-name prefix / keyword -> our RoomFunction enum.
# The example rooms are named like "01:Woonkamer", "08:Wc".
ROOM_FUNCTION_KEYWORDS = [
    ("woonkamer", "living_room"),
    ("woon", "living_room"),
    ("keuken", "kitchen"),
    ("slaapkamer", "bedroom"),
    ("slaap", "bedroom"),
    ("badkamer", "bathroom"),
    ("bad", "bathroom"),
    ("douche", "bathroom"),
    ("toilet", "toilet"),
    ("wc", "toilet"),
    ("entree", "hallway"),
    ("hal", "hallway"),
    ("gang", "hallway"),
    ("overloop", "landing"),
    ("berging", "storage"),
    ("zolder", "attic"),
]

# theta_i (design day temperature) -> room function fallback when name is ambiguous.
TEMP_TO_FUNCTION = {
    20.0: "living_room",
    22.0: "bathroom",
    24.0: "bathroom",
    15.0: "hallway",
}


@dataclass
class PaletteEntry:
    """A construction from the project's construction palette."""

    construction_id: int
    type_: str
    rc_value: Optional[float]
    u_window: Optional[float]


@dataclass
class ExtractionStats:
    """Counters surfaced at the end of a run."""

    rooms: int = 0
    faces: int = 0
    faces_with_explicit_construction: int = 0
    faces_palette_fallback: int = 0
    faces_no_u_value: int = 0
    warnings: list[str] = field(default_factory=list)


# --- ZIP / SQLite plumbing -------------------------------------------------


def extract_database(vp_path: Path, work_dir: Path) -> Path:
    """Extract ``Elements.sqlite3`` from a ``.vp`` ZIP into ``work_dir``.

    Uses ``ZipFile.read`` + manual write rather than ``extract`` because the
    latter produced 0-byte files in some Windows/bash mount combinations.
    """
    with zipfile.ZipFile(vp_path) as zf:
        names = zf.namelist()
        if ELEMENTS_DB_NAME not in names:
            raise FileNotFoundError(
                f"{ELEMENTS_DB_NAME} not found in {vp_path} (members: {names})"
            )
        data = zf.read(ELEMENTS_DB_NAME)
    out_path = work_dir / ELEMENTS_DB_NAME
    out_path.write_bytes(data)
    logger.info("Extracted %s (%d bytes) -> %s", ELEMENTS_DB_NAME, len(data), out_path)
    return out_path


def open_db(db_path: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    return conn


# --- U-value helpers -------------------------------------------------------


def surface_resistances(
    slope_deg: float, part_type: Optional[str] = None
) -> tuple[float, float]:
    """Return (R_si, R_se) for a face per ISO 6946 heat-flow bands.

    ``part_type`` (BuildingPartType) takes precedence when given, because a
    horizontal face has slope 0 or 180 degrees and slope alone cannot tell a
    floor (downward heat flow, R_si=0.17) from a roof/ceiling (upward, R_si=0.10).
    The raw slope band is only used as a fallback for unlabelled faces.
    """
    if part_type is not None:
        if part_type == "Floor":
            return R_FLOOR
        if part_type in ("Roof", "FlatRoof", "SlopingRoof", "Ceiling"):
            return R_ROOF
        if part_type in ("Wall", "Window", "Door"):
            return R_WALL
    if slope_deg < SLOPE_FLOOR_MAX_DEG:
        return R_FLOOR
    if slope_deg > SLOPE_ROOF_MIN_DEG:
        return R_ROOF
    return R_WALL


def u_from_rc(
    rc_value: float, slope_deg: float, part_type: Optional[str] = None
) -> float:
    """U = 1 / (R_si + Rc + R_se)."""
    r_si, r_se = surface_resistances(slope_deg, part_type)
    return 1.0 / (r_si + rc_value + r_se)


def load_palette(conn: sqlite3.Connection) -> dict[int, PaletteEntry]:
    """Load every construction in the project palette keyed by Construction.ID.

    Opaque entries carry an ``Rc`` value; transparent entries carry a composite
    window U-value (frame % weighting of frame/glazing U, identical to the Rust
    ``compute_u_window``).
    """
    palette: dict[int, PaletteEntry] = {}

    # NOTE: in the ISSO 51 example DBs BOTH ConstructionData.OpaqueConstructionDataID
    # AND TransparentConstructionDataID are populated for every construction (the
    # FK is never NULL), so the discriminator is ConstructionData.Type, not FK
    # nullability. We therefore classify on cd.Type: Window/Door -> transparent
    # window U-chain, everything else -> opaque Rc.
    opaque_sql = """
        SELECT c.ID AS cid, cd.Type AS ctype, sc.RcValue AS rc
        FROM Construction c
        JOIN ConstructionData cd ON cd.ID = c.DataID
        LEFT JOIN OpaqueConstructionData ocd ON ocd.ID = cd.OpaqueConstructionDataID
        LEFT JOIN StandardConstruction sc ON sc.ID = ocd.StandardConstructionID
        WHERE cd.Type NOT IN ('Window', 'Door')
    """
    for row in conn.execute(opaque_sql):
        rc = row["rc"]
        palette[row["cid"]] = PaletteEntry(
            construction_id=row["cid"],
            type_=row["ctype"],
            rc_value=float(rc) if rc is not None else None,
            u_window=None,
        )

    # Transparent (window/door) composite U-values.
    window_sql = """
        SELECT c.ID AS cid, cd.Type AS ctype,
               fr.U AS frame_u, fr.FramePercentage AS frame_pct,
               g.U AS glazing_u, tcd.Psi AS psi
        FROM Construction c
        JOIN ConstructionData cd ON cd.ID = c.DataID
        JOIN TransparentConstructionData tcd ON tcd.ID = cd.TransparentConstructionDataID
        LEFT JOIN Frame fr ON fr.ID = tcd.FrameID
        LEFT JOIN StandardWindow sw ON sw.ID = tcd.StandardWindowID
        LEFT JOIN Glazing g ON g.ID = sw.GlazingID
        WHERE cd.Type IN ('Window', 'Door')
    """
    for row in conn.execute(window_sql):
        u_win = _composite_window_u(
            row["frame_u"], row["frame_pct"], row["glazing_u"]
        )
        palette[row["cid"]] = PaletteEntry(
            construction_id=row["cid"],
            type_=row["ctype"],
            rc_value=None,
            u_window=u_win,
        )
    return palette


def _composite_window_u(
    frame_u: Optional[float], frame_pct: Optional[float], glazing_u: Optional[float]
) -> Optional[float]:
    """Frame-percentage weighted window U, mirroring the Rust importer."""
    if frame_u is not None and glazing_u is not None:
        pct = (frame_pct or 0.0) / 100.0
        return pct * frame_u + (1.0 - pct) * glazing_u
    if glazing_u is not None:
        return glazing_u
    if frame_u is not None:
        return frame_u
    return None


def palette_default_for_type(
    palette: dict[int, PaletteEntry], part_type: str, boundary: str
) -> Optional[PaletteEntry]:
    """Pick the most plausible palette entry for an unassigned face.

    Walls are ambiguous (the palette holds exterior Rc 2.6 *and* party-wall
    Rc 0.22 *and* Rc 0.2). We disambiguate on the resolved boundary type:
    adjacent-building walls take the lowest-Rc party-wall entry, exterior walls
    take the highest-Rc insulated entry.
    """
    candidates = [p for p in palette.values() if _type_matches(p.type_, part_type)]
    if not candidates:
        return None
    if part_type in ("Wall",):
        opaque = [p for p in candidates if p.rc_value is not None]
        if not opaque:
            return candidates[0]
        if boundary in ("adjacent_building", "adjacent_room", "unheated_space"):
            # Internal / party walls -> lowest Rc (uninsulated).
            return min(opaque, key=lambda p: p.rc_value)
        # Exterior walls -> highest Rc (insulated facade).
        return max(opaque, key=lambda p: p.rc_value)
    return candidates[0]


def _type_matches(palette_type: str, part_type: str) -> bool:
    if palette_type == part_type:
        return True
    roofs = {"Roof", "FlatRoof", "SlopingRoof"}
    if palette_type in roofs and part_type in roofs:
        return True
    return False


def resolve_u_value(
    palette: dict[int, PaletteEntry],
    construction_id: Optional[int],
    part_type: str,
    boundary: str,
    slope_deg: float,
    stats: ExtractionStats,
    face_id: int,
) -> float:
    """Resolve a U-value for a face, with fallbacks + WARNINGs."""
    entry: Optional[PaletteEntry] = None
    explicit = False
    if construction_id is not None and construction_id in palette:
        entry = palette[construction_id]
        explicit = True
    else:
        entry = palette_default_for_type(palette, part_type, boundary)

    if entry is None:
        msg = (
            f"face {face_id} ({part_type}/{boundary}): no construction in palette; "
            f"using sentinel U={SENTINEL_U_VALUE}"
        )
        logger.warning(msg)
        stats.warnings.append(msg)
        stats.faces_no_u_value += 1
        return SENTINEL_U_VALUE

    if entry.type_ in TRANSPARENT_TYPES:
        u = entry.u_window if entry.u_window is not None else TRANSPARENT_FALLBACK_U
    elif entry.rc_value is not None and entry.rc_value > 0.0:
        u = u_from_rc(entry.rc_value, slope_deg, part_type)
    else:
        # Rc == 0 in palette (e.g. unfinished wall) -> sentinel.
        u = SENTINEL_U_VALUE
        msg = (
            f"face {face_id} ({part_type}/{boundary}): palette construction "
            f"{entry.construction_id} has Rc=0; using sentinel U={SENTINEL_U_VALUE}"
        )
        logger.warning(msg)
        stats.warnings.append(msg)
        stats.faces_no_u_value += 1
        return u

    if explicit:
        stats.faces_with_explicit_construction += 1
    else:
        stats.faces_palette_fallback += 1
        msg = (
            f"face {face_id} ({part_type}/{boundary}): no explicit ConstructionID, "
            f"used type-based palette entry {entry.construction_id} "
            f"(Rc={entry.rc_value}, U_win={entry.u_window}) -> U={u:.3f}"
        )
        logger.warning(msg)
        stats.warnings.append(msg)
    return u


# --- High-level mappers ----------------------------------------------------


def map_info(conn: sqlite3.Connection) -> dict:
    sql = """
        SELECT p.Name AS name, p.Description AS descr, pd.ReferenceNumber AS ref
        FROM Project p
        JOIN ProjectData pd ON p.ProjectDataID = pd.ID
        LIMIT 1
    """
    row = conn.execute(sql).fetchone()
    if row is None:
        return {"name": "Imported Vabi project"}
    return {
        "name": row["name"] or "Imported Vabi project",
        "project_number": row["ref"],
        "notes": row["descr"],
        "engineer": "Vabi import (extract_vp.py)",
    }


def map_climate(conn: sqlite3.Connection) -> dict:
    row = conn.execute(
        "SELECT DesignOutsideTemperatureWinter AS theta_e "
        "FROM ClimateHeatLossCalculation LIMIT 1"
    ).fetchone()
    theta_e = row["theta_e"] if row and row["theta_e"] is not None else DEFAULT_THETA_E_C
    return {"theta_e": float(theta_e)}


def map_building_type(shape: Optional[str], hood: Optional[str]) -> str:
    mapping = {
        "Detached": "detached",
        "SemiDetached": "semi_detached",
        "CornerBuilding": "end_of_terrace",
        "Terraced": "terraced",
        "Gallery": "gallery",
        "Porch": "porch",
        "Apartment": "stacked",
    }
    if shape in mapping:
        return mapping[shape]
    if hood == "WithHood":
        return "stacked"
    return "terraced"


def map_building(conn: sqlite3.Connection, stats: ExtractionStats) -> dict:
    sql = """
        SELECT bdc.SpecificQv10 AS spec_qv10, bdc.MeasuredQv10 AS meas_qv10,
               bdc.Qv10Type AS qv10_type, bdc.BuildingShapeType AS shape,
               bdc.BuildingWithHoodType AS hood, bdc.CertaintyClass AS cclass,
               b.UsageArea AS usage_area, b.NumberOfFloors AS floors
        FROM Building b
        JOIN Project p ON b.ProjectVersionID = p.CurrentProjectVersionID
        JOIN VarAsp_BuildingRequirementsData var ON var.AspectID = b.RequirementsID
        JOIN BuildingRequirementsTemplate brt ON brt.ID = var.TemplateID
        JOIN BuildingRequirementsData brd ON brd.ID = brt.DataID
        JOIN BuildingDesignConditions bdc ON bdc.ID = brd.ConditionsID
        LIMIT 1
    """
    row = conn.execute(sql).fetchone()
    if row is None:
        msg = "No BuildingDesignConditions row; emitting building defaults."
        logger.warning(msg)
        stats.warnings.append(msg)
        return {
            "building_type": "terraced",
            "qv10": DEFAULT_QV10_DM3S,
            "security_class": "b",
            "total_floor_area": 0.0,
            "num_floors": 1,
        }

    qv10_type = row["qv10_type"]
    spec_qv10 = row["spec_qv10"]
    meas_qv10 = row["meas_qv10"]
    if qv10_type == "Measured" and meas_qv10:
        qv10 = float(meas_qv10)
    elif qv10_type == "Specific" and spec_qv10:
        qv10 = float(spec_qv10)
    else:
        # FlatRate / unknown / zero -> no usable qv10 stored. WARN.
        qv10 = float(spec_qv10 or meas_qv10 or 0.0) or DEFAULT_QV10_DM3S
        msg = (
            f"Qv10Type={qv10_type!r} with SpecificQv10={spec_qv10}, "
            f"MeasuredQv10={meas_qv10}: no usable air-tightness stored; "
            f"using fallback qv10={qv10}"
        )
        logger.warning(msg)
        stats.warnings.append(msg)

    cclass = {"ClassA": "a", "ClassB": "b", "ClassC": "c"}.get(row["cclass"], "b")
    usage_area = float(row["usage_area"] or 0.0)
    floors = int(row["floors"] or 0) or 1

    return {
        "building_type": map_building_type(row["shape"], row["hood"]),
        "qv10": qv10,
        "security_class": cclass,
        "total_floor_area": usage_area,  # often 0 in examples; patched from rooms later
        "num_floors": floors,
        "has_night_setback": True,
        "warmup_time": 2.0,
    }


def map_ventilation(conn: sqlite3.Connection) -> dict:
    sql = """
        SELECT v.SupplySource AS supply, v.CirculationRateMethod2017 AS circ,
               v.LocalHeatRecoverySystemXID AS wtw_id, hr.ValueBasedOnUnit AS hr_eff
        FROM Ventilation v
        LEFT JOIN LocalHeatRecoverySystemX hr ON hr.ID = v.LocalHeatRecoverySystemXID
        LIMIT 1
    """
    row = conn.execute(sql).fetchone()
    if row is None:
        return {"system_type": "system_c", "has_heat_recovery": False}

    has_hr = row["wtw_id"] is not None
    if has_hr:
        system = "system_d"
    else:
        system = _ventilation_system(row["supply"], row["circ"])
    cfg = {"system_type": system, "has_heat_recovery": has_hr}
    if has_hr and row["hr_eff"] is not None:
        cfg["heat_recovery_efficiency"] = float(row["hr_eff"])
    return cfg


def _ventilation_system(supply: Optional[str], circ: Optional[str]) -> str:
    table = {
        ("Natural", "Mechanical"): "system_c",
        ("Mechanical", "Natural"): "system_b",
        ("Mechanical", "Mechanical"): "system_d",
    }
    if (supply, circ) in table:
        return table[(supply, circ)]
    if supply == "Natural":
        return "system_a"
    return "system_c"


def room_function(name: str, theta_i: float) -> str:
    lowered = name.lower()
    for keyword, func in ROOM_FUNCTION_KEYWORDS:
        if keyword in lowered:
            return func
    return TEMP_TO_FUNCTION.get(theta_i, "living_room")


def room_volume(conn: sqlite3.Connection, room_id: int) -> Optional[float]:
    sql = """
        SELECT tvd.Volume AS vol
        FROM Room r
        JOIN VariantVolumeInfo vvi ON vvi.VolumeInfoID = r.VolumeInfoID
        JOIN TypedVolumeData tvd ON tvd.VariantVolumeInfoID = vvi.ID
        WHERE r.ID = ? AND tvd.Type = 'InternalDimensionsIncludingPlenum'
        LIMIT 1
    """
    row = conn.execute(sql, (room_id,)).fetchone()
    if row and row["vol"]:
        return float(row["vol"])
    return None


def map_room_faces(
    conn: sqlite3.Connection,
    room_id: int,
    palette: dict[int, PaletteEntry],
    stats: ExtractionStats,
) -> list[dict]:
    """Read every constructed/non-virtual face of a room.

    Unlike the Rust importer this does NOT filter on ``HasConstruction = 1``;
    it takes all real (non-virtual) faces so the envelope is complete. The
    U-value is resolved per face from the explicit construction or the palette.
    """
    sql = """
        SELECT bp.ID AS bp_id, bp.BuildingPartType AS part_type,
               bp.IsVirtual AS is_virtual, bp.ConstructionID AS construction_id,
               fge.Area AS area, fge.Slope AS slope,
               bc.Type AS boundary, cd.Type AS construction_type,
               bp.PsiThermalBridge AS psi
        FROM Room r
        JOIN MainFace mf ON mf.CellID = r.CellID
        JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
        JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
        JOIN Face f ON f.ID = bp.FaceID
        JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
        LEFT JOIN BoundaryConditions bc ON bc.ID = bp.BoundaryConditionsID
        LEFT JOIN Construction cc ON cc.ID = bp.ConstructionID
        LEFT JOIN ConstructionData cd ON cd.ID = cc.DataID
        WHERE r.ID = ? AND bp.IsVirtual = 0
        ORDER BY bp.BuildingPartType, bp.ID
    """
    faces: list[dict] = []
    counter = 1
    for row in conn.execute(sql, (room_id,)):
        area = float(row["area"] or 0.0)
        if area <= 0.0:
            continue
        part_type = row["part_type"] or "Wall"
        slope = float(row["slope"]) if row["slope"] is not None else 90.0
        boundary = BOUNDARY_TYPE_MAP.get(row["boundary"], "exterior")
        if row["boundary"] not in BOUNDARY_TYPE_MAP and row["boundary"] is not None:
            msg = f"face {row['bp_id']}: unknown boundary {row['boundary']!r} -> exterior"
            logger.warning(msg)
            stats.warnings.append(msg)

        u_value = resolve_u_value(
            palette, row["construction_id"], part_type, boundary,
            slope, stats, row["bp_id"],
        )

        # Material type from BuildingPartType (reliably populated, unlike the
        # face-level ConstructionID which is NULL for most faces): Window/Door
        # -> non_masonry, everything else -> masonry.
        material = "non_masonry" if (part_type in TRANSPARENT_TYPES) else "masonry"

        vertical = VERTICAL_POSITION_MAP.get(part_type, "wall")
        psi = row["psi"]
        use_forfaitaire = (psi is None) or (psi <= 0.0)

        face = {
            "id": f"bp_{row['bp_id']}",
            "description": f"{part_type} {counter}",
            "area": round(area, 4),
            "u_value": round(u_value, 4),
            "boundary_type": boundary,
            "material_type": material,
            "vertical_position": vertical,
            "use_forfaitaire_thermal_bridge": use_forfaitaire,
            "has_embedded_heating": False,
        }
        if psi and psi > 0.0:
            face["custom_delta_u_tb"] = float(psi)
        faces.append(face)
        counter += 1
        stats.faces += 1
    return faces


def map_rooms(
    conn: sqlite3.Connection,
    palette: dict[int, PaletteEntry],
    stats: ExtractionStats,
) -> list[dict]:
    sql = """
        SELECT r.ID AS rid, r.RoomNumber AS number, r.Name AS name,
               dt.TemperatureDay AS theta_i
        FROM Room r
        JOIN Project p ON r.ProjectVersionID = p.CurrentProjectVersionID
        LEFT JOIN VarAsp_RoomRequirementsData var ON var.AspectID = r.RoomRequirementsID
        LEFT JOIN RoomRequirementsTemplate rrt ON rrt.ID = var.TemplateID
        LEFT JOIN RoomRequirementsData rrd ON rrd.ID = rrt.DataID
        LEFT JOIN RoomDesignConditions rdc ON rdc.ID = rrd.ConditionsID
        LEFT JOIN DesignTemperatures dt ON dt.ID = rdc.DesignTemperaturesWinterID
        WHERE r.UseInCalculations = 1
        ORDER BY r.RoomNumber
    """
    rooms: list[dict] = []
    for row in conn.execute(sql):
        room_id = row["rid"]
        name = row["name"] or f"Room {row['number']}"
        theta_i = float(row["theta_i"]) if row["theta_i"] is not None else 20.0
        faces = map_room_faces(conn, room_id, palette, stats)

        floor_area = sum(f["area"] for f in faces if f["vertical_position"] == "floor")
        volume = room_volume(conn, room_id)
        if floor_area > 0.0 and volume:
            height = round(volume / floor_area, 3)
        elif floor_area > 0.0:
            height = DEFAULT_ROOM_HEIGHT_M
        else:
            height = FALLBACK_ROOM_HEIGHT_M

        func = room_function(name, theta_i)
        room = {
            "id": str(row["number"]),
            "name": name,
            "function": func,
            "floor_area": round(floor_area, 4),
            "height": height,
            "constructions": faces,
            "heating_system": "radiator_ht",
            "clamp_positive": True,
        }
        # Preserve the Vabi design temperature when it diverges from the
        # function default (e.g. toilets/halls authored at 15 C).
        if func == "custom":
            room["custom_temperature"] = theta_i
        rooms.append(room)
        stats.rooms += 1
    return rooms


def build_project(conn: sqlite3.Connection, stats: ExtractionStats) -> dict:
    palette = load_palette(conn)
    logger.info("Loaded construction palette with %d entries", len(palette))

    info = map_info(conn)
    climate = map_climate(conn)
    ventilation = map_ventilation(conn)
    building = map_building(conn, stats)
    rooms = map_rooms(conn, palette, stats)

    # Patch total_floor_area from rooms when the building row stored 0.
    if building.get("total_floor_area", 0.0) <= 0.0 and rooms:
        building["total_floor_area"] = round(sum(r["floor_area"] for r in rooms), 4)

    return {
        "info": info,
        "building": building,
        "climate": climate,
        "ventilation": ventilation,
        "rooms": rooms,
    }


# --- CLI -------------------------------------------------------------------


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract a Vabi Elements .vp into ISSO 51 project-JSON."
    )
    parser.add_argument("vp_path", type=Path, help="Path to the .vp project file")
    parser.add_argument(
        "-o", "--output", type=Path, default=None,
        help="Output JSON path (default: stdout)",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true",
        help="Enable INFO/WARNING logging to stderr",
    )
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    logging.basicConfig(
        level=logging.INFO if args.verbose else logging.WARNING,
        format="[%(levelname)s] %(message)s",
    )
    if not args.vp_path.exists():
        logger.error("File not found: %s", args.vp_path)
        return 1

    stats = ExtractionStats()
    with tempfile.TemporaryDirectory(prefix="vabi-extract-") as tmp:
        db_path = extract_database(args.vp_path, Path(tmp))
        conn = open_db(db_path)
        try:
            project = build_project(conn, stats)
        finally:
            conn.close()

    payload = json.dumps(project, indent=2, ensure_ascii=False)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
        logger.info("Wrote %s", args.output)
    else:
        print(payload)

    logger.info(
        "Stats: %d rooms, %d faces (%d explicit construction, %d palette fallback, "
        "%d no U-value), %d warnings",
        stats.rooms, stats.faces, stats.faces_with_explicit_construction,
        stats.faces_palette_fallback, stats.faces_no_u_value, len(stats.warnings),
    )
    # Always print a compact summary to stderr so non-verbose runs see coverage.
    import sys
    print(
        f"[summary] rooms={stats.rooms} faces={stats.faces} "
        f"explicit_u={stats.faces_with_explicit_construction} "
        f"palette_fallback={stats.faces_palette_fallback} "
        f"no_u={stats.faces_no_u_value} warnings={len(stats.warnings)}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
