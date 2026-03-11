"""Extract room polygons from IfcSpace entities.

This is the core module — it converts each IfcSpace to a 2D floor polygon
plus height, using IfcOpenShell's geometry engine.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import numpy as np

from ifc_tool.constants import (
    FLOOR_HEIGHT_DEFAULT_MM,
    MIN_POLYGON_POINTS,
    MIN_ROOM_AREA_MM2,
)
from ifc_tool.import_ifc.function_mapper import match_room_function
from ifc_tool.import_ifc.geometry import (
    dedup_vertices,
    ensure_ccw,
    extract_floor_polygon,
    polygon_area,
    remove_collinear,
)
from ifc_tool.import_ifc.storey_resolver import StoreyInfo, get_space_storey
from ifc_tool.models import (
    ImportWarning,
    ModelRoom,
    Point2D,
    SpaceDiagnostic,
)

if TYPE_CHECKING:
    import ifcopenshell
    import ifcopenshell.geom

logger = logging.getLogger(__name__)


def extract_spaces(
    ifc_file: ifcopenshell.file,
    unit_to_mm: float,
    storeys: dict[str, StoreyInfo],
) -> tuple[list[ModelRoom], list[ImportWarning], list[SpaceDiagnostic]]:
    """Extract all IfcSpace entities as ModelRoom objects.

    Args:
        ifc_file: An opened IfcOpenShell file.
        unit_to_mm: Conversion factor from IFC units to mm.
        storeys: Resolved storey mapping from storey_resolver.

    Returns:
        Tuple of (rooms, warnings, diagnostics).
    """
    import ifcopenshell.geom

    spaces = ifc_file.by_type("IfcSpace")
    logger.info("Found %d IfcSpace entities", len(spaces))

    settings = ifcopenshell.geom.settings()
    # Note: do NOT set "use-python-opencascade" — even setting it to False
    # raises if OCC is not installed. The default is already False.
    settings.set("apply-default-materials", False)
    settings.set("use-world-coords", True)

    rooms: list[ModelRoom] = []
    warnings: list[ImportWarning] = []
    diagnostics: list[SpaceDiagnostic] = []

    for space in spaces:
        space_name = space.Name or space.LongName or f"Space_{space.id()}"
        logger.debug("Processing space: %s (%s)", space_name, space.GlobalId)

        try:
            result = _extract_single_space(
                space, space_name, settings, unit_to_mm, storeys
            )
        except Exception as exc:
            logger.warning(
                "Failed to extract space %s: %s", space_name, exc
            )
            warnings.append(
                ImportWarning(
                    space_name=space_name,
                    message=f"Extraction failed: {exc}",
                )
            )
            diagnostics.append(
                SpaceDiagnostic(
                    space_id=space.id(),
                    space_name=space_name,
                    strategy="error",
                    polygon_points=0,
                    area_mm2=0.0,
                )
            )
            continue

        if result is None:
            diagnostics.append(
                SpaceDiagnostic(
                    space_id=space.id(),
                    space_name=space_name,
                    strategy="skipped",
                    polygon_points=0,
                    area_mm2=0.0,
                )
            )
            continue

        room, diag = result
        rooms.append(room)
        diagnostics.append(diag)

    return rooms, warnings, diagnostics


def _extract_single_space(
    space: ifcopenshell.entity_instance,
    space_name: str,
    settings: ifcopenshell.geom.settings,
    unit_to_mm: float,
    storeys: dict[str, StoreyInfo],
) -> tuple[ModelRoom, SpaceDiagnostic] | None:
    """Extract a single IfcSpace → ModelRoom.

    Returns None if the space should be skipped (too small, no geometry).
    """
    import ifcopenshell.geom

    try:
        shape = ifcopenshell.geom.create_shape(settings, space)
    except Exception as exc:
        logger.debug("No geometry for space %s: %s", space_name, exc)
        return None

    # Get vertices from the shape
    verts_flat = shape.geometry.verts
    if len(verts_flat) == 0:
        return None

    # Reshape to Nx3 and convert to mm.
    # IfcOpenShell create_shape always returns coordinates in meters,
    # regardless of the file's native units, so multiply by 1000.
    vertices = np.array(verts_flat).reshape(-1, 3) * 1000.0

    # Extract floor polygon from bottom face
    polygon, height = extract_floor_polygon(vertices)

    if len(polygon) < MIN_POLYGON_POINTS:
        logger.debug(
            "Space %s: only %d polygon points, skipping",
            space_name,
            len(polygon),
        )
        return None

    # Clean up polygon
    polygon = dedup_vertices(polygon)
    polygon = remove_collinear(polygon)
    polygon = ensure_ccw(polygon)

    if len(polygon) < MIN_POLYGON_POINTS:
        return None

    # Check minimum area
    area = abs(polygon_area(polygon))
    if area < MIN_ROOM_AREA_MM2:
        logger.debug(
            "Space %s: area %.0f mm² < minimum %.0f mm², skipping",
            space_name,
            area,
            MIN_ROOM_AREA_MM2,
        )
        return None

    # Use extracted height or fallback
    if height < 100:  # Less than 100mm is clearly wrong
        height = FLOOR_HEIGHT_DEFAULT_MM

    # Resolve floor index and elevation
    storey_info = get_space_storey(space, storeys, unit_to_mm)
    floor_index = storey_info.floor_index if storey_info else 0
    elevation = storey_info.elevation_mm if storey_info else None

    # Transform polygon: IFC Y-up → screen Y-down
    transformed = [
        Point2D(x=p.x, y=-p.y) for p in polygon
    ]

    # Match room function from name
    function = match_room_function(space_name)

    room = ModelRoom(
        name=space_name,
        function=function,
        polygon=transformed,
        floor=floor_index,
        height=height,
        elevation=elevation,
    )

    diagnostic = SpaceDiagnostic(
        space_id=space.id(),
        space_name=space_name,
        strategy="ifcopenshell-geom",
        polygon_points=len(transformed),
        area_mm2=area,
    )

    return room, diagnostic
