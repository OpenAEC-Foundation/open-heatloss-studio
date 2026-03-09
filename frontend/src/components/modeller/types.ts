export interface Point2D {
  x: number;
  y: number;
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
}

export interface ModelDoor {
  roomId: string;
  wallIndex: number;
  offset: number;
  width: number;
  swing: "left" | "right";
}

// ---------------------------------------------------------------------------
// Wall boundary type (for heat loss calculation)
// ---------------------------------------------------------------------------

/** How a wall relates to the building boundary — determines which temperature applies. */
export type WallBoundaryType =
  | "auto"       // Determine automatically from geometry (default)
  | "exterior"   // Gevel — buitenwand (θe)
  | "interior"   // Binnenwand — naar verwarmde ruimte (θi)
  | "neighbor"   // Scheidingsmuur — wand naar buren (θadj)
  | "unheated"   // Naar onverwarmde ruimte (θu)
  | "ground";    // Grenzend aan grond

export const BOUNDARY_TYPE_LABELS: Record<WallBoundaryType, string> = {
  auto: "Automatisch",
  exterior: "Gevel (buiten)",
  interior: "Binnenwand",
  neighbor: "Scheidingsmuur (buren)",
  unheated: "Naar onverwarmd",
  ground: "Naar grond",
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
  | "split_room"
  | "annotate_text"
  | "annotate_dimension"
  | "annotate_leader"
  | "measure";

export type ViewMode = "2d" | "3d";

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

export type Selection =
  | { type: "room"; roomId: string }
  | { type: "wall"; roomId: string; wallIndex: number }
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
  modes: ["grid", "endpoint", "midpoint", "nearest", "perpendicular"],
  gridSize: 100,
};
