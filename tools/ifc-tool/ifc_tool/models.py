"""Pydantic models mirroring the TypeScript interfaces in modeller/types.ts.

Serialization uses ``by_alias=True`` so JSON keys are camelCase,
matching what the frontend expects.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


# ---------------------------------------------------------------------------
# Geometry primitives
# ---------------------------------------------------------------------------


class Point2D(BaseModel):
    """2D point in mm coordinates."""

    x: float
    y: float


# ---------------------------------------------------------------------------
# Room / Window / Door — mirrors ModelRoom, ModelWindow, ModelDoor
# ---------------------------------------------------------------------------


class ModelRoom(BaseModel):
    """A room extracted from an IfcSpace."""

    model_config = ConfigDict(populate_by_name=True)

    name: str
    function: str
    polygon: list[Point2D]
    floor: int
    height: float = Field(description="Room height in mm")
    elevation: float | None = Field(
        default=None,
        description="Absolute floor elevation in mm relative to +0.00",
    )
    temperature: float | None = Field(
        default=None,
        description="Design temperature in °C",
    )


class ModelWindow(BaseModel):
    """A window extracted from an IfcWindow."""

    model_config = ConfigDict(populate_by_name=True)

    room_id: str = Field(alias="roomId")
    wall_index: int = Field(alias="wallIndex")
    offset: float = Field(description="Offset from wall start to center, mm")
    width: float = Field(description="Window width in mm")
    height: float | None = Field(
        default=None,
        description="Window height in mm (from OverallHeight)",
    )
    sill_height: float | None = Field(
        default=None,
        alias="sillHeight",
        description="Sill height above floor in mm",
    )


class ModelDoor(BaseModel):
    """A door extracted from an IfcDoor."""

    model_config = ConfigDict(populate_by_name=True)

    room_id: str = Field(alias="roomId")
    wall_index: int = Field(alias="wallIndex")
    offset: float = Field(description="Offset from wall start to center, mm")
    width: float = Field(description="Door width in mm")
    height: float | None = Field(
        default=None,
        description="Door height in mm (from OverallHeight)",
    )
    swing: str = Field(description="'left' or 'right'")


# ---------------------------------------------------------------------------
# Wall type info — mirrors IfcWallTypeInfo from ifc-wall-types.ts
# ---------------------------------------------------------------------------


class WallTypeLayer(BaseModel):
    """A single material layer in a wall type."""

    model_config = ConfigDict(populate_by_name=True)

    material_name: str = Field(alias="materialName")
    thickness_mm: float = Field(alias="thicknessMm")
    match: str | None = Field(
        default=None,
        description="Matched catalogue material (None = unmatched)",
    )


class IfcWallTypeInfo(BaseModel):
    """Wall type extracted from IfcWallType with material layers."""

    model_config = ConfigDict(populate_by_name=True)

    name: str
    global_id: str = Field(alias="globalId")
    layers: list[WallTypeLayer]
    original_material_names: list[str] = Field(alias="originalMaterialNames")


# ---------------------------------------------------------------------------
# Import result — top-level output of the import command
# ---------------------------------------------------------------------------


class ImportWarning(BaseModel):
    """A non-fatal warning from the import process."""

    model_config = ConfigDict(populate_by_name=True)

    space_name: str = Field(alias="spaceName")
    message: str


class ImportStats(BaseModel):
    """Statistics about the import run."""

    model_config = ConfigDict(populate_by_name=True)

    spaces_found: int = Field(alias="spacesFound")
    spaces_imported: int = Field(alias="spacesImported")
    spaces_skipped: int = Field(alias="spacesSkipped")


class SpaceDiagnostic(BaseModel):
    """Detailed diagnostic for a single IfcSpace extraction."""

    model_config = ConfigDict(populate_by_name=True)

    space_id: int = Field(alias="spaceId")
    space_name: str = Field(alias="spaceName")
    strategy: str
    polygon_points: int = Field(alias="polygonPoints")
    area_mm2: float = Field(alias="areaMm2")


class IfcImportResult(BaseModel):
    """Complete result of an IFC import operation.

    This is the JSON that goes to stdout.
    """

    model_config = ConfigDict(populate_by_name=True)

    rooms: list[ModelRoom]
    windows: list[ModelWindow] = Field(default_factory=list)
    doors: list[ModelDoor] = Field(default_factory=list)
    wall_types: list[IfcWallTypeInfo] = Field(
        default_factory=list, alias="wallTypes"
    )
    warnings: list[ImportWarning] = Field(default_factory=list)
    diagnostics: list[SpaceDiagnostic] = Field(default_factory=list)
    stats: ImportStats


class ImportError(BaseModel):
    """Error result when import fails."""

    error: str
    detail: str | None = None
