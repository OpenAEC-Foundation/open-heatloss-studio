import type { MaterialType, VerticalPosition } from "../../types";
import type { CatalogueCategory, CatalogueLayer } from "../../lib/constructionCatalogue";

export interface Point2D {
  x: number;
  y: number;
}

// ---------------------------------------------------------------------------
// Project construction (per-project layer-based construction)
// ---------------------------------------------------------------------------

export interface ProjectConstruction {
  id: string;
  /** Auto-generated from layers ("KZS 100 | PIR 110 | ..."). */
  name: string;
  category: CatalogueCategory;
  materialType: MaterialType;
  verticalPosition: VerticalPosition;
  /**
   * Layer build-up. Mag leeg zijn voor kozijnen/vullingen die alleen een
   * directe U-waarde hebben (triple-glas, buitendeur, etc.). Voor laag-gebaseerde
   * constructies wordt de U-waarde afgeleid uit `layers`; voor leeg-laag entries
   * fallbackt het systeem op `uValue`.
   */
  layers: CatalogueLayer[];
  /**
   * Directe U-waarde (W/(m²·K)). Alleen gebruikt wanneer `layers` leeg is
   * (kozijnen/vullingen die geen laag-gebaseerde Rc-berekening hebben).
   */
  uValue?: number;
  /** ID of the catalogue entry this was copied from (if any). */
  catalogueSourceId?: string;
  /** Traceability to IFC source (optional). */
  ifcSource?: {
    wallTypeName: string;
    globalId: string;
    originalMaterialNames: string[];
  };
}

export interface ModelRoom {
  id: string;
  name: string;
  /** Room function — kept as generic string so the modeller stays decoupled. */
  function: string;
  polygon: Point2D[];
  floor: number;
  /** Room height in mm. */
  height: number;
  /** Absolute floor elevation in mm relative to reference level +0.00. */
  elevation?: number;
  /** Design temperature in °C (default based on function). */
  temperature?: number;
}

export interface ModelWindow {
  roomId: string;
  /** Edge index of the room polygon (0 = first edge). */
  wallIndex: number;
  /** Offset from wall start to window center, in mm. */
  offset: number;
  /** Window width in mm. */
  width: number;
  /** Window height in mm (from IFC OverallHeight). */
  height?: number;
  /** Sill height above floor level in mm. */
  sillHeight?: number;
}

export interface ModelDoor {
  roomId: string;
  wallIndex: number;
  offset: number;
  width: number;
  /** Door height in mm (from IFC OverallHeight). */
  height?: number;
  swing: "left" | "right";
}

// ---------------------------------------------------------------------------
// Wall boundary type (for heat loss calculation)
// ---------------------------------------------------------------------------

/** How a wall relates to the building boundary — determines which temperature applies. */
export type WallBoundaryType =
  | "auto"          // Determine automatically from geometry (default)
  | "exterior"      // Gevel — buitenwand (θe)
  | "interior"      // Binnenwand — naar verwarmde ruimte (θi)
  | "neighbor"      // Scheidingsmuur — wand naar buren (θadj)
  | "unheated"      // Naar onverwarmde ruimte (θu)
  | "ground"        // Grenzend aan grond
  | "curtain_wall"; // Vliesgevel — volledig beglazing (θe)

export const BOUNDARY_TYPE_LABELS: Record<WallBoundaryType, string> = {
  auto: "Automatisch",
  exterior: "Gevel (buiten)",
  interior: "Binnenwand",
  neighbor: "Scheidingsmuur (buren)",
  unheated: "Naar onverwarmd",
  ground: "Naar grond",
  curtain_wall: "Vliesgevel",
};

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

export type ModellerTool =
  | "select"
  | "pan"
  | "draw_rect"
  | "draw_polygon"
  | "draw_circle"
  | "draw_window"
  | "draw_door"
  | "place_supply"
  | "place_exhaust"
  | "split_room"
  | "annotate_text"
  | "annotate_dimension"
  | "annotate_leader"
  | "measure";

// ---------------------------------------------------------------------------
// Imported thermal boundaries (from Revit thermal import)
// ---------------------------------------------------------------------------

export interface ImportedBoundary {
  id: string;
  roomId: string;
  adjacentRoomId: string;
  orientation: 'wall' | 'floor' | 'ceiling' | 'roof';
  boundaryCondition: 'exterior' | 'ground' | 'water' | 'unheated' | 'adjacent';
  area_m2: number;
  compass?: string;
}

/**
 * Real 3D geometry carried over from a thermal import (v1.1), kept so the 3D
 * viewer can render the actual room boundaries (instead of the derived
 * rectangle) and — in a later step — apply true-north rotation and a north
 * arrow. All coordinates are stored in METERS, exactly as the backend returns
 * them; consumers convert to the modeller's mm coordinate space where needed.
 */
export interface ImportSurfaceGeometry {
  /** Matches the source construction.id / opening.id. */
  id: string;
  /** 3D vertices in meters. */
  vertices: [number, number, number][];
}

export interface ImportRoomPolygon {
  /** Matches the calc Room.id. */
  roomId: string;
  /** 2D boundary polygon in meters. */
  polygon: [number, number][];
  name?: string;
  level?: string;
  heightM?: number;
  /**
   * Real floor elevation in meters (raw Revit Z), derived from the room's
   * floor-construction vertices (min-Z). When present, the 3D viewer stacks
   * this room at its true height instead of along a Y-pitch. Undefined when no
   * floor construction / geometry was available — viewer falls back to
   * level-name grouping + cumulative height.
   */
  floorZ?: number;
}

export interface ImportGeometry {
  /** Per-room real 2D boundary polygons (meters). */
  roomPolygons: ImportRoomPolygon[];
  /** True-north rotation in degrees, if present in the export. Step 3b applies it. */
  trueNorthDeg?: number;
  /** Per-construction 3D vertices (meters). For step 3b heatmap/box rendering. */
  constructionGeometries: ImportSurfaceGeometry[];
  /** Per-opening 3D vertices (meters). For step 3b. */
  openingGeometries: ImportSurfaceGeometry[];
}

export type ViewMode = "2d" | "3d";

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

export type Selection =
  | { type: "room"; roomId: string }
  | { type: "wall"; roomId: string; wallIndex: number; segmentEdges?: number[] }
  | { type: "window"; roomId: string; wallIndex: number; offset: number }
  | { type: "door"; roomId: string; wallIndex: number; offset: number }
  | null;

// ---------------------------------------------------------------------------
// Snap
// ---------------------------------------------------------------------------

export type SnapMode =
  | "grid"
  | "endpoint"
  | "midpoint"
  | "perpendicular"
  | "nearest"
  | "underlay";

export interface SnapSettings {
  enabled: boolean;
  modes: SnapMode[];
  gridSize: number; // mm
}

export const DEFAULT_SNAP_SETTINGS: SnapSettings = {
  enabled: true,
  modes: ["endpoint", "midpoint", "nearest", "perpendicular"],
  gridSize: 100,
};
