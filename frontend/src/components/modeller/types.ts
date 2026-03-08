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
// Tools
// ---------------------------------------------------------------------------

export type ModellerTool =
  | "select"
  | "pan"
  | "draw_rect"
  | "draw_polygon"
  | "draw_circle"
  | "draw_wall"
  | "draw_window"
  | "draw_door"
  | "draw_floor"
  | "draw_roof"
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
  modes: ["grid", "endpoint", "midpoint", "perpendicular"],
  gridSize: 100,
};
