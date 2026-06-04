/**
 * Derive 2D room geometry (polygons + walls) from calc-side `Room` data.
 *
 * Reasoning:
 * - The Modeller is a read-only viewer of the calc data. Polygons aren't
 *   drawn by the user; they're inferred from the construction list.
 * - For each room, walls are constructions with `vertical_position === "wall"`
 *   (or undefined, which we treat as wall by historical default).
 * - Total wall *length* (perimeter) = Σ (wall.area / room.height).
 * - For a rectangle, perimeter `p` and area `a` give:
 *     w + h = p / 2
 *     w · h = a
 *   Solving the quadratic w² − (p/2)·w + a = 0 yields the side lengths.
 *   When the discriminant is negative (data inconsistency), we fall back
 *   to a square with side √a — visualization, not architectural truth.
 * - Walls are mapped to rectangle sides ordered by area descending so the
 *   "largest" wall is consistently the bottom edge (south). Boundary type
 *   on each wall is preserved for color-coding downstream.
 *
 * Floor placement: rooms are laid out in a grid per "floor". `Room` has no
 * explicit floor field, so we parse a `[BG]`/`[1V]` prefix from `room.name`
 * if present, otherwise group everything on floor 0. Future `Room.floor`
 * field would replace the prefix-parsing.
 *
 * Units: input `floor_area` and `area` are m² and m, output coordinates
 * are mm to match the existing modeller types (`Point2D` in mm).
 */

import type { ConstructionElement, Project, Room } from "../types";
import type {
  ImportRoomPolygon,
  ModelDoor,
  ModelRoom,
  ModelWindow,
  Point2D,
} from "../components/modeller/types";

/** mm per m. */
const MM = 1000;

/** Default room height in m if not specified on the Room. */
const DEFAULT_HEIGHT_M = 2.6;

/** Spacing between rooms in the grid layout, in mm. */
const ROOM_GAP_MM = 1500;

/**
 * Derived geometry for a single room: rectangle polygon + ordered walls.
 *
 * `walls[i]` corresponds to polygon edge `i` (between vertex i and i+1).
 * Side order is bottom (south, longest), right (east), top (north), left (west).
 */
export interface DerivedRoomGeometry {
  /** Polygon in mm, closed (4 points for a rectangle). */
  polygon: Point2D[];
  /**
   * Walls per polygon edge — same index order as polygon edges.
   * Each entry is the source ConstructionElement (or null if room has fewer
   * than 4 wall constructions and this side is "synthetic").
   */
  walls: Array<ConstructionElement | null>;
  /** Width of the rectangle in mm. */
  widthMm: number;
  /** Height of the rectangle in mm (depth, not 3D height). */
  depthMm: number;
}

/**
 * Solve rectangle dimensions from perimeter and area.
 *
 * Returns (width, depth) in meters, where width >= depth. If the system
 * has no real solution (perimeter too small for the given area), returns
 * a square with side √area so the viewer always renders something.
 */
export function rectangleFromPerimeterAndArea(
  perimeterM: number,
  areaM2: number,
): { width: number; depth: number } {
  if (areaM2 <= 0) {
    return { width: 0, depth: 0 };
  }
  if (perimeterM <= 0) {
    const s = Math.sqrt(areaM2);
    return { width: s, depth: s };
  }
  const halfP = perimeterM / 2;
  const disc = halfP * halfP - 4 * areaM2;
  if (disc < 0) {
    const s = Math.sqrt(areaM2);
    return { width: s, depth: s };
  }
  const sqrtDisc = Math.sqrt(disc);
  const width = (halfP + sqrtDisc) / 2;
  const depth = areaM2 / width;
  return { width, depth };
}

/**
 * Pick wall constructions from a room. Treats `undefined` vertical_position
 * as wall (historical default — older fixtures don't set the field).
 */
export function wallConstructions(room: Room): ConstructionElement[] {
  return room.constructions.filter(
    (c) =>
      c.vertical_position === "wall" || c.vertical_position === undefined,
  );
}

/**
 * Derive geometry for a single room. Pure function — no I/O, no store reads.
 */
export function deriveRoomGeometry(room: Room): DerivedRoomGeometry {
  const heightM = room.height ?? DEFAULT_HEIGHT_M;
  const walls = wallConstructions(room);

  // Perimeter = Σ (wall.area / room.height).
  const perimeterM = walls.reduce((sum, w) => sum + w.area / heightM, 0);

  const { width, depth } = rectangleFromPerimeterAndArea(
    perimeterM,
    room.floor_area,
  );

  const widthMm = width * MM;
  const depthMm = depth * MM;

  // Rectangle polygon: bottom-left, bottom-right, top-right, top-left (CCW).
  const polygon: Point2D[] = [
    { x: 0, y: 0 },
    { x: widthMm, y: 0 },
    { x: widthMm, y: depthMm },
    { x: 0, y: depthMm },
  ];

  // Map walls to sides. Order walls by area descending; assign in side order
  // bottom (south) → right (east) → top (north) → left (west). This places
  // the "largest" wall consistently at the bottom for visual coherence.
  const sortedWalls = [...walls].sort((a, b) => b.area - a.area);
  const sides: Array<ConstructionElement | null> = [
    sortedWalls[0] ?? null,
    sortedWalls[1] ?? null,
    sortedWalls[2] ?? null,
    sortedWalls[3] ?? null,
  ];

  return { polygon, walls: sides, widthMm, depthMm };
}

/**
 * Rotate a 2D point (in meters) by `deg` degrees about the origin.
 *
 * --- ROTATION SIGN (single source of truth) ---------------------------------
 * Positive `deg` rotates the raw Revit XY coordinates counter-clockwise to
 * align the footprint north-up. If the rendered plan turns out MIRRORED in the
 * true-north direction, flip the sign of `rad` on the line below (one-line
 * change) — this is the only place rotation is computed.
 * ----------------------------------------------------------------------------
 *
 * When `deg` is undefined, 0, or non-finite the input is returned unchanged
 * (identity) so non-rotated / v1.0 imports never shift.
 */
function rotatePoint2D(
  x: number,
  y: number,
  deg: number | undefined,
): [number, number] {
  if (deg === undefined || !Number.isFinite(deg) || deg === 0) {
    return [x, y];
  }
  const rad = (deg * Math.PI) / 180; // ← flip to `-(deg * Math.PI) / 180` if mirrored
  const cos = Math.cos(rad);
  const sin = Math.sin(rad);
  return [x * cos - y * sin, x * sin + y * cos];
}

/** Rotate a whole 2D polygon (meters) about the origin. */
export function rotatePolygon2D(
  polygonM: [number, number][],
  deg?: number,
): [number, number][] {
  if (deg === undefined || !Number.isFinite(deg) || deg === 0) {
    return polygonM;
  }
  return polygonM.map(([x, y]) => rotatePoint2D(x, y, deg));
}

/** Shared 2D transform applied to every imported surface: a true-north
 * rotation about the origin followed by a single global offset (in meters).
 * Keeping one transform object guarantees rooms, walls, and surface vertices
 * all land in the same coordinate frame, preserving their real relative
 * positions. */
export interface ImportTransform {
  /** True-north rotation in degrees (about the world origin). */
  trueNorthDeg?: number;
  /** Global offset (meters) subtracted after rotation, so the whole building
   * sits in the positive quadrant. */
  originXM: number;
  originYM: number;
}

/** Apply the shared import transform to a single point (meters → meters). */
export function applyImportTransform(
  x: number,
  y: number,
  t: ImportTransform,
): [number, number] {
  const [rx, ry] = rotatePoint2D(x, y, t.trueNorthDeg);
  return [rx - t.originXM, ry - t.originYM];
}

/**
 * Build geometry for a room from a real imported boundary polygon (meters),
 * using the SHARED import transform (global rotation + global origin) so the
 * room keeps its true position relative to every other imported room. No
 * per-room normalization — that would discard the inter-room layout.
 *
 * Returns absolute mm coordinates in the shared building frame (already
 * positioned; the caller does NOT grid-place these). `widthMm`/`depthMm` are
 * the bbox dimensions, kept for API compatibility. Returns `null` when the
 * polygon is degenerate (< 3 points) so the caller can fall back to the
 * derived rectangle.
 */
export function geometryFromImportPolygon(
  polygonM: [number, number][],
  transform: ImportTransform,
): DerivedRoomGeometry | null {
  if (!Array.isArray(polygonM) || polygonM.length < 3) return null;

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  const polygon: Point2D[] = [];
  for (const [px, py] of polygonM) {
    const [x, y] = applyImportTransform(px, py, transform);
    polygon.push({ x: x * MM, y: y * MM });
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
  }
  if (!Number.isFinite(minX) || !Number.isFinite(minY)) return null;

  const widthMm = (maxX - minX) * MM;
  const depthMm = (maxY - minY) * MM;

  // The real polygon has one edge per vertex; we have no per-edge construction
  // mapping for imported polygons (that linking is fase 4), so walls are left
  // unassigned. The 2D canvas tolerates a null-filled walls array.
  const walls: Array<ConstructionElement | null> = polygon.map(() => null);

  return { polygon, walls, widthMm, depthMm };
}

/**
 * Compute the shared import transform for a set of imported room polygons.
 *
 * Step 1: rotate every polygon by `trueNorthDeg` about the world origin.
 * Step 2: take the global bbox-min over ALL rotated polygons as the shared
 *         origin, so the whole building shifts into the positive quadrant
 *         while every inter-room distance is preserved exactly.
 *
 * Returns `null` when there are no usable polygons (callers fall back to grid).
 */
export function computeImportTransform(
  polygons: [number, number][][],
  trueNorthDeg?: number,
): ImportTransform | null {
  let minX = Infinity;
  let minY = Infinity;
  let any = false;
  for (const poly of polygons) {
    if (!Array.isArray(poly) || poly.length < 3) continue;
    for (const [px, py] of poly) {
      const [x, y] = rotatePoint2D(px, py, trueNorthDeg);
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      any = true;
    }
  }
  if (!any || !Number.isFinite(minX) || !Number.isFinite(minY)) return null;
  return { trueNorthDeg, originXM: minX, originYM: minY };
}

// ---------------------------------------------------------------------------
// Floor parsing
// ---------------------------------------------------------------------------

/**
 * Parse a floor index from a room name with a Dutch convention prefix.
 *
 * Examples:
 *   "[BG] Berging"   → 0   (begane grond)
 *   "[1V] Slaapkamer" → 1  (eerste verdieping)
 *   "[2V] Zolder"    → 2
 *   "[KE] Kelder"    → -1
 *   "Berging"        → 0   (no prefix → ground floor by default)
 */
export function parseFloorFromName(name: string): number {
  const m = name.match(/^\s*\[\s*(BG|KE|(\d+)V?)\s*\]/i);
  if (!m || !m[1]) return 0;
  const tag = m[1].toUpperCase();
  if (tag === "BG") return 0;
  if (tag === "KE") return -1;
  // [1V], [2V], [3V] etc.
  const num = m[2];
  if (num) return parseInt(num, 10);
  // [1], [2] without V suffix
  const numOnly = parseInt(tag, 10);
  return Number.isFinite(numOnly) ? numOnly : 0;
}

// ---------------------------------------------------------------------------
// Grid layout
// ---------------------------------------------------------------------------

/**
 * Convert an entire `Project` to read-only ModelRooms positioned in a grid.
 *
 * Per floor (parsed from name prefix), rooms are arranged in rows of
 * approximately `cols` columns. The widest room in each row determines the
 * row spacing. Different floors are vertically separated by `floorGap`.
 *
 * Output ModelRooms have:
 * - `id`: the calc Room.id (so wallConstructions/wallBoundaryTypes maps,
 *   keyed by ModelRoom.id, continue to apply)
 * - `name`, `function`: from the calc Room
 * - `polygon`: from `deriveRoomGeometry`, translated to grid position
 * - `floor`: parsed from name prefix
 * - `height`: from Room.height in mm (or DEFAULT_HEIGHT_M·1000)
 */
export function deriveModelRooms(
  project: Project,
  options?: {
    cols?: number;
    gapMm?: number;
    floorGapMm?: number;
    /**
     * Real imported boundary polygons (meters), keyed by Room.id. When a room
     * has an entry here its actual boundary is used instead of the derived
     * rectangle. Rooms without an entry fall back to `deriveRoomGeometry`.
     */
    roomPolygons?: ImportRoomPolygon[];
    /**
     * Model true-north rotation in degrees (from the v1.1 import). Applied to
     * every imported room polygon so footprints render north-up. Undefined/0 →
     * no rotation (no regression for non-imported / v1.0 projects).
     */
    trueNorthDeg?: number;
  },
): ModelRoom[] {
  const cols = options?.cols ?? 4;
  const gapMm = options?.gapMm ?? ROOM_GAP_MM;
  const floorGapMm = options?.floorGapMm ?? 5000;
  const trueNorthDeg = options?.trueNorthDeg;

  // Index imported polygons by room id for O(1) lookup.
  const polygonByRoom = new Map<string, ImportRoomPolygon>();
  for (const rp of options?.roomPolygons ?? []) {
    polygonByRoom.set(rp.roomId, rp);
  }

  // Shared import transform: one global true-north rotation + one global
  // origin over ALL imported polygons. This keeps every imported room on its
  // real position relative to the rest of the building (no per-room normalize,
  // no grid). Rooms without a boundary keep using the schematic grid.
  const importPolys: [number, number][][] = [];
  for (const room of project.rooms) {
    const ip = polygonByRoom.get(room.id);
    if (ip) importPolys.push(ip.polygon);
  }
  const transform = importPolys.length
    ? computeImportTransform(importPolys, trueNorthDeg)
    : null;

  // Split rooms into "imported" (real absolute geometry) and "grid" (fallback).
  const imported: Array<{ room: Room; geom: DerivedRoomGeometry; floor: number }> = [];
  const gridByFloor = new Map<number, Array<{ room: Room; geom: DerivedRoomGeometry }>>();

  for (const room of project.rooms) {
    const floor = parseFloorFromName(room.name);
    const ip = polygonByRoom.get(room.id);
    const importedGeom =
      ip && transform ? geometryFromImportPolygon(ip.polygon, transform) : null;
    if (importedGeom) {
      imported.push({ room, geom: importedGeom, floor });
    } else {
      // No real boundary (pseudo-room, missing polygon, degenerate): keep the
      // perimeter+area derived rectangle in the schematic grid.
      const list = gridByFloor.get(floor) ?? [];
      list.push({ room, geom: deriveRoomGeometry(room) });
      gridByFloor.set(floor, list);
    }
  }

  const out: ModelRoom[] = [];

  // ── Imported rooms: real absolute XY, floors separated along the Y axis ──
  // Every imported polygon already shares one frame (positive quadrant). To
  // avoid stacked floors overlapping in the top-down plan, shift each floor's
  // rooms by a per-floor Y offset based on the global building depth; XY within
  // a floor stays exactly real.
  if (imported.length) {
    // Global building span (mm) in the shared frame, to size the floor gap.
    let buildingMaxY = 0;
    for (const { geom } of imported) {
      for (const p of geom.polygon) {
        if (p.y > buildingMaxY) buildingMaxY = p.y;
      }
    }
    const floorPitchMm = buildingMaxY + floorGapMm;

    const importedFloors = [...new Set(imported.map((r) => r.floor))].sort(
      (a, b) => a - b,
    );
    const floorRank = new Map<number, number>();
    importedFloors.forEach((f, i) => floorRank.set(f, i));

    for (const { room, geom, floor } of imported) {
      const yShift = (floorRank.get(floor) ?? 0) * floorPitchMm;
      const polygon = geom.polygon.map((p) => ({ x: p.x, y: p.y + yShift }));
      out.push({
        id: room.id,
        name: room.name,
        function: String(room.function),
        polygon,
        floor,
        height: (room.height ?? DEFAULT_HEIGHT_M) * MM,
      });
    }
  }

  // ── Grid rooms (fallback): schematic layout, parked clear of the imported
  // block so the two never overlap. ──
  let floorYOffset = 0;
  if (imported.length) {
    for (const r of out) {
      for (const p of r.polygon) {
        if (p.y > floorYOffset) floorYOffset = p.y;
      }
    }
    if (floorYOffset > 0) floorYOffset += floorGapMm;
  }

  const sortedGridFloors = [...gridByFloor.keys()].sort((a, b) => a - b);
  for (const floor of sortedGridFloors) {
    const rooms = gridByFloor.get(floor)!;
    let rowHeight = 0;
    let rowYStart = floorYOffset;
    let cursorX = 0;

    rooms.forEach(({ room, geom }, idx) => {
      const col = idx % cols;
      // New row: reset X, advance Y by previous row height + gap.
      if (col === 0 && idx > 0) {
        rowYStart += rowHeight + gapMm;
        rowHeight = 0;
        cursorX = 0;
      }

      // Translate the geometry to grid position.
      const translated = geom.polygon.map((p) => ({
        x: cursorX + p.x,
        y: rowYStart + p.y,
      }));

      out.push({
        id: room.id,
        name: room.name,
        function: String(room.function),
        polygon: translated,
        floor,
        height: (room.height ?? DEFAULT_HEIGHT_M) * MM,
      });

      cursorX += geom.widthMm + gapMm;
      rowHeight = Math.max(rowHeight, geom.depthMm);
    });

    // Advance Y past this floor's rooms before starting the next floor.
    floorYOffset = rowYStart + rowHeight + floorGapMm;
  }

  return out;
}

/**
 * Derived windows are constructions with vertical_position "wall" + zero
 * or low U-value not really — actually we don't currently distinguish
 * windows from walls in the calc model except by description heuristics.
 * For PR D iteration 1, we return an empty array. Future iteration can
 * synthesize windows from `kozijnen_vullingen` constructions by inspecting
 * their project_construction_id → ProjectConstruction.category.
 */
export function deriveModelWindows(_project: Project): ModelWindow[] {
  return [];
}

/** Same reasoning as deriveModelWindows. */
export function deriveModelDoors(_project: Project): ModelDoor[] {
  return [];
}
