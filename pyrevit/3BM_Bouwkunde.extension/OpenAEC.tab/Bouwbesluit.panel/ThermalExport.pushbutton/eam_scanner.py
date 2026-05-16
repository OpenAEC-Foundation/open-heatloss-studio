# -*- coding: utf-8 -*-
"""EnergyAnalysisDetailModel scanner voor thermische schil export.

Scant het Revit model via de EAM API (SecondLevelBoundaries) en bouwt
een data-dict op met rooms, constructions, openings en open_connections.

IronPython 2.7 — geen f-strings, geen type hints.
"""
import math

from Autodesk.Revit.DB import (
    FilteredElementCollector,
    BuiltInCategory,
    BuiltInParameter,
    ElementId,
    Transaction,
    XYZ,
)
from Autodesk.Revit.DB.Analysis import (
    EnergyAnalysisDetailModel,
    EnergyAnalysisDetailModelOptions,
    EnergyAnalysisDetailModelTier,
    EnergyModelType,
)

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "lib"))
from warmteverlies.unit_utils import (
    internal_to_sqm,
    internal_to_meters,
    internal_to_mm,
    get_param_value,
)
from warmteverlies.constants import FEET_TO_M


# =============================================================================
# Compass helpers
# =============================================================================
_COMPASS_DIRS = [
    ("N", 0), ("NE", 45), ("E", 90), ("SE", 135),
    ("S", 180), ("SW", 225), ("W", 270), ("NW", 315),
]


def _angle_to_compass(angle_deg):
    """Converteer een hoek (0=Noord, CW) naar kompasrichting."""
    angle_deg = angle_deg % 360
    best = "N"
    best_diff = 999
    for name, ref in _COMPASS_DIRS:
        diff = abs(angle_deg - ref)
        if diff > 180:
            diff = 360 - diff
        if diff < best_diff:
            best_diff = diff
            best = name
    return best


def _normal_to_compass(normal):
    """Bereken kompasrichting uit een surface normal XYZ vector.

    Revit: X = Oost, Y = Noord. We berekenen de hoek vanaf Noord (Y-as), CW.
    """
    angle_rad = math.atan2(normal.X, normal.Y)
    angle_deg = math.degrees(angle_rad)
    if angle_deg < 0:
        angle_deg += 360
    return _angle_to_compass(angle_deg)


# =============================================================================
# Orientation helpers
# =============================================================================
def _classify_orientation(normal):
    """Classificeer de orientatie van een vlak op basis van z-component.

    Returns:
        tuple: (orientation_str, is_vertical)
    """
    z = normal.Z
    if z > 0.7:
        return "roof", False  # wijst omhoog = dak of plafond
    elif z < -0.7:
        return "floor", False  # wijst omlaag = vloer
    else:
        return "wall", True


def _refine_orientation(orientation, adj_type):
    """Verfijn roof/floor op basis van adjacency.

    Dak naar buiten = roof, dak naar andere ruimte = ceiling.
    """
    if orientation == "roof" and adj_type in ("room", "unheated"):
        return "ceiling"
    return orientation


# =============================================================================
# Polyloop area
# =============================================================================
def _polyloop_area_sqft(polyloop):
    """Bereken oppervlakte van een polyloop in square feet (Shoelace 3D)."""
    pts = list(polyloop.GetPoints())
    if len(pts) < 3:
        return 0.0

    # Bereken normaal via cross product van eerste twee edges
    v1 = XYZ(pts[1].X - pts[0].X, pts[1].Y - pts[0].Y, pts[1].Z - pts[0].Z)
    v2 = XYZ(pts[2].X - pts[0].X, pts[2].Y - pts[0].Y, pts[2].Z - pts[0].Z)
    normal = XYZ(
        v1.Y * v2.Z - v1.Z * v2.Y,
        v1.Z * v2.X - v1.X * v2.Z,
        v1.X * v2.Y - v1.Y * v2.X,
    )
    length = math.sqrt(normal.X ** 2 + normal.Y ** 2 + normal.Z ** 2)
    if length < 1e-10:
        return 0.0
    normal = XYZ(normal.X / length, normal.Y / length, normal.Z / length)

    # 3D polygon area via Newell's method
    area = 0.0
    n = len(pts)
    for i in range(n):
        j = (i + 1) % n
        cross = XYZ(
            pts[i].Y * pts[j].Z - pts[i].Z * pts[j].Y,
            pts[i].Z * pts[j].X - pts[i].X * pts[j].Z,
            pts[i].X * pts[j].Y - pts[i].Y * pts[j].X,
        )
        area += normal.X * cross.X + normal.Y * cross.Y + normal.Z * cross.Z

    return abs(area) / 2.0


def _polyloop_to_2d(polyloop):
    """Converteer polyloop naar 2D punten in meters [[x, y], ...]."""
    pts = list(polyloop.GetPoints())
    result = []
    for p in pts:
        result.append([round(p.X * FEET_TO_M, 4), round(p.Y * FEET_TO_M, 4)])
    return result


# =============================================================================
# Compound structure layer extraction
# =============================================================================
def _extract_layers(doc, source_element):
    """Extraheer laagopbouw uit een Revit element's CompoundStructure.

    Returns:
        list[dict]: Lagen van interieur naar exterieur met material, dikte, type, lambda.
    """
    if source_element is None:
        return []

    try:
        elem_type = doc.GetElement(source_element.GetTypeId())
        if elem_type is None:
            return []

        compound = elem_type.GetCompoundStructure()
        if compound is None:
            return []

        raw_layers = compound.GetLayers()
        if not raw_layers or raw_layers.Count == 0:
            return []
    except Exception:
        return []

    result = []
    cumulative_mm = 0.0

    for layer in raw_layers:
        thickness_ft = layer.Width
        thickness_mm = round(internal_to_mm(thickness_ft), 1)

        # Materiaal ophalen
        mat_name = "Onbekend"
        lambda_val = None
        layer_type = "solid"

        mat_id = layer.MaterialId
        if mat_id is not None and mat_id != ElementId.InvalidElementId:
            material = doc.GetElement(mat_id)
            if material is not None:
                mat_name = material.Name or "Onbekend"
                lambda_val = _get_lambda(material)

        # Air gap detectie via MaterialFunctionAssignment
        try:
            func_assignment = layer.Function
            # MaterialFunctionAssignment enum: 0=Structure, 1=Substrate,
            # 2=Insulation, 3=Finish1, 4=Finish2, 5=Membrane,
            # 6=StructuralDeck, 7=ThermalAirGap (indien beschikbaar)
            func_name = str(func_assignment)
            if "Air" in func_name or "Thermal" in func_name:
                layer_type = "air_gap"
        except Exception:
            pass

        layer_dict = {
            "material": mat_name,
            "thickness_mm": thickness_mm,
            "distance_from_interior_mm": round(cumulative_mm, 1),
            "type": layer_type,
        }
        if lambda_val is not None and lambda_val > 0:
            layer_dict["lambda"] = round(lambda_val, 4)

        result.append(layer_dict)
        cumulative_mm += thickness_mm

    return result


def _get_lambda(material):
    """Haal thermische geleidbaarheid (lambda) uit een Revit Material."""
    try:
        thermal_asset_id = material.ThermalAssetId
        if (thermal_asset_id is None
                or thermal_asset_id == ElementId.InvalidElementId):
            return None

        doc = material.Document
        prop_set = doc.GetElement(thermal_asset_id)
        if prop_set is None:
            return None

        thermal_asset = prop_set.GetThermalAsset()
        if thermal_asset is None:
            return None

        return thermal_asset.ThermalConductivity
    except Exception:
        return None


# =============================================================================
# Opening extraction from analytical openings
# =============================================================================
def _extract_eam_opening(doc, opening, constr_id, opening_idx):
    """Extraheer opening data uit een EnergyAnalysisOpening.

    Returns:
        dict conform thermal-import schema Opening definitie
    """
    opening_dict = {
        "id": "opening-{0}".format(opening_idx),
        "construction_id": constr_id,
    }

    # Type bepalen
    open_type = str(opening.OpeningType)
    if "Window" in open_type or "Skylight" in open_type:
        opening_dict["type"] = "window"
    elif "Door" in open_type:
        opening_dict["type"] = "door"
    else:
        opening_dict["type"] = "window"  # default

    # Afmetingen uit polyloop
    try:
        polyloop = opening.GetPolyloop()
        pts = list(polyloop.GetPoints())
        if len(pts) >= 3:
            # Bounding box benadering
            xs = [p.X for p in pts]
            ys = [p.Y for p in pts]
            zs = [p.Z for p in pts]
            dx = max(xs) - min(xs)
            dy = max(ys) - min(ys)
            dz = max(zs) - min(zs)

            # Breedte = max van dx, dy; hoogte = dz (voor wanden)
            width_ft = max(dx, dy)
            height_ft = dz if dz > 0.01 else max(dx, dy)
            if dz < 0.01:
                # Horizontale opening (daklicht) — gebruik dx en dy
                width_ft = dx
                height_ft = dy

            opening_dict["width_mm"] = round(width_ft * FEET_TO_M * 1000, 0)
            opening_dict["height_mm"] = round(height_ft * FEET_TO_M * 1000, 0)
        else:
            opening_dict["width_mm"] = 1000
            opening_dict["height_mm"] = 1000
    except Exception:
        opening_dict["width_mm"] = 1000
        opening_dict["height_mm"] = 1000

    # Sill height
    try:
        sill_param = None
        origin_elem = doc.GetElement(opening.OriginatingElementId)
        if origin_elem is not None:
            sill_param = get_param_value(origin_elem, "Sill Height")
            if sill_param is None:
                sill_param = get_param_value(origin_elem, "Rough Sill Height")
        if sill_param is not None and sill_param > 0:
            opening_dict["sill_height_mm"] = round(
                internal_to_mm(sill_param), 0
            )
    except Exception:
        pass

    # Revit element reference
    try:
        orig_id = opening.OriginatingElementId
        if orig_id is not None and orig_id != ElementId.InvalidElementId:
            opening_dict["revit_element_id"] = orig_id.IntegerValue
            orig_elem = doc.GetElement(orig_id)
            if orig_elem is not None:
                try:
                    elem_type = doc.GetElement(orig_elem.GetTypeId())
                    if elem_type is not None:
                        family = getattr(elem_type, "FamilyName", "")
                        tname = elem_type.get_Parameter(
                            BuiltInParameter.ALL_MODEL_TYPE_NAME
                        )
                        if tname and tname.HasValue:
                            opening_dict["revit_type_name"] = "{0}: {1}".format(
                                family, tname.AsString()
                            )
                        elif family:
                            opening_dict["revit_type_name"] = family
                except Exception:
                    pass
    except Exception:
        pass

    return opening_dict


# =============================================================================
# Room mapping: EnergyAnalysisSpace -> Revit Room
# =============================================================================
def _build_room_map(doc, eam, output):
    """Bouw een mapping van EnergyAnalysisSpace ID -> room data dict.

    Returns:
        tuple: (room_map, rooms_list, room_id_counter)
            room_map: dict van space_id -> room dict
            rooms_list: list van alle room dicts
    """
    rooms_list = []
    room_map = {}  # space_id -> room dict
    room_counter = 0

    spaces = eam.GetAnalyticalSpaces()
    if output:
        output.print_md("Analytische spaces gevonden: **{0}**".format(
            len(spaces) if spaces else 0
        ))

    if not spaces:
        return room_map, rooms_list

    for space in spaces:
        space_elem = doc.GetElement(space)
        if space_elem is None:
            continue

        space_name = space_elem.SpaceName or "Space"
        space_id = space.IntegerValue

        # Probeer gekoppelde Revit Room te vinden via CADObjectUniqueId
        revit_room = None
        revit_id = None
        try:
            cad_uid = space_elem.CADObjectUniqueId
            if cad_uid:
                # CADObjectUniqueId is vaak de UniqueId van het Room element
                collector = (
                    FilteredElementCollector(doc)
                    .OfCategory(BuiltInCategory.OST_Rooms)
                    .WhereElementIsNotElementType()
                )
                for room in collector:
                    if room.UniqueId == cad_uid:
                        revit_room = room
                        revit_id = room.Id.IntegerValue
                        break
        except Exception:
            pass

        # Room data opbouwen
        room_id = "room-{0}".format(room_counter)
        room_counter += 1

        area_m2 = 0.0
        height_m = 0.0
        level_name = ""
        boundary_polygon = None

        if revit_room is not None:
            area_m2 = internal_to_sqm(revit_room.Area) if revit_room.Area > 0 else 0.0
            from warmteverlies.unit_utils import get_room_height
            height_m = get_room_height(revit_room)

            level = doc.GetElement(revit_room.LevelId)
            level_name = level.Name if level else ""

            name = ""
            name_param = revit_room.get_Parameter(BuiltInParameter.ROOM_NAME)
            if name_param and name_param.HasValue:
                name = name_param.AsString() or ""
            if name:
                space_name = name
        else:
            # Fallback: probeer area uit de space zelf
            try:
                area_m2 = internal_to_sqm(space_elem.Area) if space_elem.Area > 0 else 0.0
            except Exception:
                pass

        room_dict = {
            "id": room_id,
            "name": space_name,
            "type": "heated",
            "level": level_name,
            "area_m2": round(area_m2, 2),
            "height_m": round(height_m, 2),
            "volume_m3": round(area_m2 * height_m, 2),
        }
        if revit_id is not None:
            room_dict["revit_id"] = revit_id

        rooms_list.append(room_dict)
        room_map[space_id] = room_dict

    return room_map, rooms_list


# =============================================================================
# Pseudo-ruimtes
# =============================================================================
_PSEUDO_OUTSIDE = {
    "id": "room-outside",
    "name": "Buiten",
    "type": "outside",
    "level": "",
    "area_m2": 0.0,
    "height_m": 0.0,
    "volume_m3": 0.0,
}

_PSEUDO_GROUND = {
    "id": "room-ground",
    "name": "Grond",
    "type": "ground",
    "level": "",
    "area_m2": 0.0,
    "height_m": 0.0,
    "volume_m3": 0.0,
}


# =============================================================================
# Main scan function
# =============================================================================
def scan_thermal_shell(doc, output=None):
    """Scan de thermische schil via de EnergyAnalysisDetailModel API.

    Args:
        doc: Revit Document
        output: pyrevit script output (optioneel, voor voortgangsmeldingen)

    Returns:
        dict met keys: rooms, constructions, openings, open_connections
        of None bij fout
    """
    if output:
        output.print_md("### EAM Scanner")
        output.print_md("EnergyAnalysisDetailModel aanmaken...")

    # ------------------------------------------------------------------
    # 1. EAM aanmaken
    # ------------------------------------------------------------------
    options = EnergyAnalysisDetailModelOptions()
    options.Tier = EnergyAnalysisDetailModelTier.SecondLevelBoundaries
    options.EnergyModelType = EnergyModelType.SpatialElement

    eam = None
    trans = Transaction(doc, "Create EAM for Thermal Export")
    try:
        trans.Start()
        eam = EnergyAnalysisDetailModel.Create(doc, options)
        trans.Commit()
    except Exception as ex:
        if trans.HasStarted():
            trans.RollBack()
        if output:
            output.print_md("**FOUT:** Kan EAM niet aanmaken: {0}".format(str(ex)))
        return None

    if eam is None:
        if output:
            output.print_md("**FOUT:** EAM is None na Create()")
        return None

    # ------------------------------------------------------------------
    # 2. Rooms opbouwen vanuit analytische spaces
    # ------------------------------------------------------------------
    room_map, rooms_list = _build_room_map(doc, eam, output)

    if not rooms_list:
        if output:
            output.print_md("**Waarschuwing:** Geen analytische spaces gevonden.")

    # Pseudo-ruimtes toevoegen
    has_outside = False
    has_ground = False

    # ------------------------------------------------------------------
    # 3. Surfaces doorlopen -> constructions + openings
    # ------------------------------------------------------------------
    constructions = []
    openings = []
    constr_counter = 0
    opening_counter = 0

    surfaces = eam.GetAnalyticalSurfaces()
    if output:
        output.print_md("Analytische surfaces: **{0}**".format(
            len(surfaces) if surfaces else 0
        ))

    if surfaces:
        for surface_id in surfaces:
            surface = doc.GetElement(surface_id)
            if surface is None:
                continue

            # Polyloop en area
            try:
                polyloop = surface.GetPolyloop()
                area_sqft = _polyloop_area_sqft(polyloop)
                area_m2 = area_sqft * 0.09290304
            except Exception:
                continue

            if area_m2 < 0.01:
                continue

            # Normaal vector
            try:
                normal = surface.Normal
            except Exception:
                normal = XYZ(0, 0, 1)

            # Orientatie
            orientation, is_vertical = _classify_orientation(normal)

            # Room A (de space waar dit surface bij hoort)
            room_a_dict = None
            try:
                analytical_space = surface.GetAnalyticalSpace()
                if analytical_space is not None:
                    space_int_id = analytical_space.IntegerValue
                    room_a_dict = room_map.get(space_int_id)
            except Exception:
                pass

            if room_a_dict is None:
                # Surface zonder space — overslaan
                continue

            # Room B (adjacent space)
            room_b_dict = None
            adj_type = "outside"  # default
            try:
                adj_space = surface.GetAdjacentAnalyticalSpace()
                if adj_space is not None and adj_space != ElementId.InvalidElementId:
                    adj_int_id = adj_space.IntegerValue
                    room_b_dict = room_map.get(adj_int_id)
                    if room_b_dict is not None:
                        adj_type = "room"
            except Exception:
                pass

            if room_b_dict is None:
                # Geen adjacent space: buiten of grond
                if orientation == "floor":
                    room_b_dict = _PSEUDO_GROUND
                    adj_type = "ground"
                    has_ground = True
                else:
                    room_b_dict = _PSEUDO_OUTSIDE
                    adj_type = "outside"
                    has_outside = True

            # Verfijn orientatie
            orientation = _refine_orientation(orientation, adj_type)

            # Compass richting voor wanden
            compass = None
            if is_vertical:
                compass = _normal_to_compass(normal)

            # Source element voor laagopbouw
            source_element = None
            revit_element_id = None
            revit_type_name = None
            try:
                orig_id = surface.OriginatingElementId
                if orig_id is not None and orig_id != ElementId.InvalidElementId:
                    source_element = doc.GetElement(orig_id)
                    revit_element_id = orig_id.IntegerValue
                    if source_element is not None:
                        try:
                            et = doc.GetElement(source_element.GetTypeId())
                            if et is not None:
                                tn = et.get_Parameter(
                                    BuiltInParameter.ALL_MODEL_TYPE_NAME
                                )
                                if tn and tn.HasValue:
                                    revit_type_name = tn.AsString()
                        except Exception:
                            pass
            except Exception:
                pass

            # Laagopbouw extraheren
            layers = _extract_layers(doc, source_element)

            # Construction dict
            constr_id = "constr-{0}".format(constr_counter)
            constr_counter += 1

            constr_dict = {
                "id": constr_id,
                "room_a": room_a_dict["id"],
                "room_b": room_b_dict["id"],
                "orientation": orientation,
                "gross_area_m2": round(area_m2, 2),
            }
            if compass:
                constr_dict["compass"] = compass
            if layers:
                constr_dict["layers"] = layers
            if revit_element_id is not None:
                constr_dict["revit_element_id"] = revit_element_id
            if revit_type_name:
                constr_dict["revit_type_name"] = revit_type_name

            constructions.append(constr_dict)

            # Openings van dit surface
            try:
                surface_openings = surface.GetAnalyticalOpenings()
                if surface_openings:
                    for open_id in surface_openings:
                        open_elem = doc.GetElement(open_id)
                        if open_elem is None:
                            continue
                        opening_dict = _extract_eam_opening(
                            doc, open_elem, constr_id, opening_counter
                        )
                        openings.append(opening_dict)
                        opening_counter += 1
            except Exception:
                pass

    # ------------------------------------------------------------------
    # 4. Pseudo-ruimtes toevoegen als ze gebruikt worden
    # ------------------------------------------------------------------
    if has_outside:
        rooms_list.append(dict(_PSEUDO_OUTSIDE))
    if has_ground:
        rooms_list.append(dict(_PSEUDO_GROUND))

    # ------------------------------------------------------------------
    # 5. EAM opruimen
    # ------------------------------------------------------------------
    try:
        trans2 = Transaction(doc, "Delete EAM")
        trans2.Start()
        EnergyAnalysisDetailModel.Destroy(doc)
        trans2.Commit()
    except Exception:
        if trans2.HasStarted():
            trans2.RollBack()

    # ------------------------------------------------------------------
    # 6. Resultaat
    # ------------------------------------------------------------------
    result = {
        "rooms": rooms_list,
        "constructions": constructions,
        "openings": openings,
        "open_connections": [],
    }

    if output:
        output.print_md(
            "Scan compleet: **{0}** rooms, **{1}** constructies, "
            "**{2}** openings".format(
                len(rooms_list), len(constructions), len(openings)
            )
        )

    return result
