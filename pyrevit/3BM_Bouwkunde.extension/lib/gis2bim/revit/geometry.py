"""Revit geometry creation helpers voor GIS data.

Tekent Model Lines, Text Notes en transformeert coordinaten
van Rijksdriehoek (RD/EPSG:28992) naar Revit interne eenheden.
Compatibel met IronPython 2.7.
"""
from __future__ import print_function
import math

import clr
clr.AddReference("RevitAPI")
from Autodesk.Revit.DB import (  # noqa: E402
    XYZ,
    Line,
    Plane,
    SketchPlane,
    TextNote,
    FilteredElementCollector,
    BuiltInCategory,
    ModelCurve,
)


# ---------------------------------------------------------------------------
# Constanten
# ---------------------------------------------------------------------------

FEET_PER_METER = 1.0 / 0.3048
TEXT_SIZE_FEET = 0.006  # ~1.8mm op papier (standaard annotatie)


# ---------------------------------------------------------------------------
# Coordinaten transformatie
# ---------------------------------------------------------------------------

def rd_to_revit_xyz(rd_x, rd_y, origin_rd_x, origin_rd_y, z=0.0):
    """Transformeer RD coordinaat naar Revit XYZ (feet).

    Args:
        rd_x: X in RD meters (oost).
        rd_y: Y in RD meters (noord).
        origin_rd_x: RD X van het model origin.
        origin_rd_y: RD Y van het model origin.
        z: Hoogte in meters (default 0).

    Returns:
        Revit XYZ in interne eenheden (feet).
    """
    dx_m = rd_x - origin_rd_x
    dy_m = rd_y - origin_rd_y
    return XYZ(
        dx_m * FEET_PER_METER,
        dy_m * FEET_PER_METER,
        z * FEET_PER_METER,
    )


# ---------------------------------------------------------------------------
# Model Lines
# ---------------------------------------------------------------------------

def create_model_lines(doc, polylines, line_style=None):
    """Teken Model Lines voor een lijst polygonen.

    Args:
        doc: Revit Document.
        polylines: Lijst van lijsten met XYZ punten (gesloten polygonen).
        line_style: Optionele GraphicsStyle voor lijnstijl.

    Returns:
        Aantal aangemaakte lijnen.
    """
    count = 0
    # SketchPlane op Z=0
    plane = Plane.CreateByNormalAndOrigin(XYZ.BasisZ, XYZ.Zero)
    sketch_plane = SketchPlane.Create(doc, plane)

    for poly in polylines:
        if len(poly) < 2:
            continue

        for i in range(len(poly)):
            p1 = poly[i]
            p2 = poly[(i + 1) % len(poly)]

            # Skip punten die te dicht bij elkaar liggen
            if p1.DistanceTo(p2) < 0.001:
                continue

            try:
                line = Line.CreateBound(p1, p2)
                model_line = doc.Create.NewModelCurve(
                    line, sketch_plane
                )

                if line_style and model_line:
                    model_line.LineStyle = line_style

                count += 1
            except Exception:
                # Ongeldige lijn (bijv. lengte 0), overslaan
                pass

    return count


# ---------------------------------------------------------------------------
# Text Notes
# ---------------------------------------------------------------------------

def create_text_notes(doc, view, annotations, text_type=None):
    """Maak Text Notes aan voor labels.

    Args:
        doc: Revit Document.
        view: View waarin de text notes worden geplaatst.
        annotations: Lijst van dicts met:
            - position: XYZ punt
            - text: Label tekst
            - rotation: Rotatie in radialen (optioneel)
        text_type: Optioneel TextNoteType ElementId.

    Returns:
        Aantal aangemaakte text notes.
    """
    if text_type is None:
        text_type = _get_default_text_type(doc)
        if text_type is None:
            return 0

    count = 0
    for ann in annotations:
        position = ann.get("position")
        text = ann.get("text", "")
        rotation = ann.get("rotation", 0.0)

        if not position or not text:
            continue

        try:
            opts = TextNote.GetDefaultOptions()
            if rotation != 0.0:
                opts.Rotation = rotation

            TextNote.Create(
                doc,
                view.Id,
                position,
                text,
                text_type,
            )
            count += 1
        except Exception:
            pass

    return count


def _get_default_text_type(doc):
    """Zoek het eerste beschikbare TextNoteType."""
    try:
        from Autodesk.Revit.DB import TextNoteType
        collector = FilteredElementCollector(doc).OfClass(TextNoteType)
        types = list(collector)
        if types:
            return types[0].Id
    except Exception:
        pass
    return None


# ---------------------------------------------------------------------------
# Feature conversie helpers
# ---------------------------------------------------------------------------

def features_to_polylines(features, origin_rd_x, origin_rd_y):
    """Converteer GeoJSON polygon features naar Revit polylines.

    Args:
        features: Lijst van Feature objecten met polygon geometry.
        origin_rd_x: RD X van model origin.
        origin_rd_y: RD Y van model origin.

    Returns:
        Lijst van lijsten met XYZ punten.
    """
    polylines = []
    for f in features:
        rings = _extract_polygon_rings(f)
        for ring in rings:
            pts = []
            for coord in ring:
                if len(coord) >= 2:
                    pts.append(
                        rd_to_revit_xyz(
                            coord[0], coord[1],
                            origin_rd_x, origin_rd_y,
                        )
                    )
            if len(pts) >= 3:
                polylines.append(pts)
    return polylines


def features_to_annotations(features, label_field, origin_rd_x, origin_rd_y):
    """Converteer GeoJSON point features naar annotatie dicts.

    Args:
        features: Lijst van Feature objecten met point geometry.
        label_field: Property naam voor het label.
        origin_rd_x: RD X van model origin.
        origin_rd_y: RD Y van model origin.

    Returns:
        Lijst van dicts met position, text, rotation.
    """
    annotations = []
    for f in features:
        label = f.get_label(label_field)
        if not label:
            continue

        coords = _extract_point_coords(f)
        if not coords:
            continue

        position = rd_to_revit_xyz(
            coords[0], coords[1],
            origin_rd_x, origin_rd_y,
        )

        rotation = _extract_rotation(f)

        annotations.append({
            "position": position,
            "text": str(label),
            "rotation": rotation,
        })

    return annotations


def _extract_polygon_rings(feature):
    """Haal polygon ringen op uit een feature (Polygon of MultiPolygon)."""
    geom_type = feature.geometry_type
    coords = feature.coordinates

    if geom_type == "Polygon":
        return coords  # [[ring1], [ring2], ...]
    elif geom_type == "MultiPolygon":
        rings = []
        for polygon in coords:
            rings.extend(polygon)
        return rings
    return []


def _extract_point_coords(feature):
    """Haal punt coordinaten op uit een feature (Point of centroid)."""
    geom_type = feature.geometry_type
    coords = feature.coordinates

    if geom_type == "Point" and len(coords) >= 2:
        return coords
    elif geom_type == "MultiPoint" and coords and len(coords[0]) >= 2:
        return coords[0]
    return None


def _extract_rotation(feature):
    """Haal rotatie op uit feature properties (PDOK 'hoek' veld)."""
    props = feature.properties or {}
    hoek = props.get("hoek") or props.get("rotatie") or props.get("rotation")
    if hoek is not None:
        try:
            return math.radians(float(hoek))
        except (ValueError, TypeError):
            pass
    return 0.0


# ---------------------------------------------------------------------------
# Locatie helpers
# ---------------------------------------------------------------------------

def get_project_location_rd(doc):
    """Probeer RD coordinaten van het project te bepalen.

    Zoekt eerst naar project parameters GIS2BIM_RD_X / GIS2BIM_RD_Y.
    Valt terug op Survey Point positie (als die RD-waarden bevat).

    Args:
        doc: Revit Document.

    Returns:
        Tuple (rd_x, rd_y) of None als niet gevonden.
    """
    # Methode 1: Project parameters
    rd = _try_project_parameters(doc)
    if rd:
        return rd

    # Methode 2: Survey Point
    rd = _try_survey_point(doc)
    if rd:
        return rd

    return None


def _try_project_parameters(doc):
    """Zoek GIS2BIM_RD_X/Y in ProjectInformation parameters."""
    try:
        project_info = doc.ProjectInformation
        rd_x = None
        rd_y = None

        for param in project_info.Parameters:
            name = param.Definition.Name
            if name == "GIS2BIM_RD_X":
                rd_x = param.AsDouble()
                if rd_x == 0:
                    rd_x = _parse_param_string(param)
            elif name == "GIS2BIM_RD_Y":
                rd_y = param.AsDouble()
                if rd_y == 0:
                    rd_y = _parse_param_string(param)

        if rd_x and rd_y and rd_x > 0 and rd_y > 0:
            return (rd_x, rd_y)
    except Exception:
        pass
    return None


def _try_survey_point(doc):
    """Haal Survey Point positie op (als RD coordinaten).

    Leest de E/W en N/S parameters van het Survey Point
    (OST_SharedBasePoint). Controleert of de waarden
    binnen het RD-bereik vallen.
    """
    try:
        from Autodesk.Revit.DB import BuiltInParameter

        collector = FilteredElementCollector(doc).OfCategory(
            BuiltInCategory.OST_SharedBasePoint
        )
        for bp in collector:
            ew_param = bp.get_Parameter(
                BuiltInParameter.BASEPOINT_EASTWEST_PARAM
            )
            ns_param = bp.get_Parameter(
                BuiltInParameter.BASEPOINT_NORTHSOUTH_PARAM
            )
            if ew_param and ns_param:
                x = ew_param.AsDouble() / FEET_PER_METER
                y = ns_param.AsDouble() / FEET_PER_METER
                # RD-bereik check: X 0-300km, Y 300-625km
                if 0 < x < 300000 and 300000 < y < 625000:
                    return (x, y)
    except Exception:
        pass

    return None


def _parse_param_string(param):
    """Probeer parameter als string te lezen en naar float te converteren."""
    try:
        val = param.AsString()
        if val:
            return float(val.replace(",", "."))
    except Exception:
        pass
    return None
