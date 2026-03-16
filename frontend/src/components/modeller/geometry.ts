/** Shared geometry helpers for the modeller. */
import type { ModelRoom, Point2D } from "./types";

// ---------------------------------------------------------------------------
// Wall segment grouping
// ---------------------------------------------------------------------------

/** Angle threshold (degrees) — edges differing by less are part of the same segment. */
const SEGMENT_ANGLE_THRESHOLD_DEG = 15;

/** Edges shorter than this (mm) never start a new segment. */
const MIN_SEGMENT_EDGE_MM = 100;

export interface WallSegment {
  segmentIndex: number;
  /** Which polygon edge indices belong to this segment. */
  edgeIndices: number[];
  /** Total length of all edges in mm. */
  length: number;
  /** Cardinal direction label based on first→last point. */
  direction: string;
}

/**
 * Group polygon edges into logical wall segments.
 *
 * Two consecutive edges belong to the same segment when their angle
 * difference is below the threshold OR the next edge is shorter than the
 * minimum edge length.
 */
export function computeWallSegments(polygon: Point2D[]): WallSegment[] {
  const n = polygon.length;
  if (n < 3) return [];

  const edgeLengths: number[] = [];
  const edgeAngles: number[] = [];

  for (let i = 0; i < n; i++) {
    const a = polygon[i]!;
    const b = polygon[(i + 1) % n]!;
    edgeLengths.push(Math.hypot(b.x - a.x, b.y - a.y));
    edgeAngles.push(Math.atan2(b.y - a.y, b.x - a.x));
  }

  // Start first segment with edge 0
  const segments: WallSegment[] = [];
  let currentEdges: number[] = [0];

  for (let i = 1; i < n; i++) {
    const prevAngle = edgeAngles[i - 1]!;
    const currAngle = edgeAngles[i]!;

    // Angle difference in degrees, wrapped to [0, 180]
    let angleDiff = Math.abs(currAngle - prevAngle) * (180 / Math.PI);
    if (angleDiff > 180) angleDiff = 360 - angleDiff;

    const isShortEdge = edgeLengths[i]! < MIN_SEGMENT_EDGE_MM;
    const isSameDirection = angleDiff < SEGMENT_ANGLE_THRESHOLD_DEG;

    if (isSameDirection || isShortEdge) {
      // Continue current segment
      currentEdges.push(i);
    } else {
      // Finish current segment and start a new one
      segments.push(buildSegment(segments.length, currentEdges, polygon, edgeLengths));
      currentEdges = [i];
    }
  }

  // Finish last segment
  segments.push(buildSegment(segments.length, currentEdges, polygon, edgeLengths));

  // Wrap-around: check if first and last segment should merge
  if (segments.length > 1) {
    const first = segments[0]!;
    const last = segments[segments.length - 1]!;
    const lastEdgeIdx = last.edgeIndices[last.edgeIndices.length - 1]!;
    const firstEdgeIdx = first.edgeIndices[0]!;

    const lastAngle = edgeAngles[lastEdgeIdx]!;
    const firstAngle = edgeAngles[firstEdgeIdx]!;
    let angleDiff = Math.abs(firstAngle - lastAngle) * (180 / Math.PI);
    if (angleDiff > 180) angleDiff = 360 - angleDiff;

    const firstIsShort = edgeLengths[firstEdgeIdx]! < MIN_SEGMENT_EDGE_MM;

    if (angleDiff < SEGMENT_ANGLE_THRESHOLD_DEG || firstIsShort) {
      // Merge: last absorbs first
      const mergedEdges = [...last.edgeIndices, ...first.edgeIndices];
      segments[segments.length - 1] = buildSegment(
        last.segmentIndex,
        mergedEdges,
        polygon,
        edgeLengths,
      );
      segments.shift();
    }
  }

  // Re-index after potential merge
  for (let i = 0; i < segments.length; i++) {
    segments[i]!.segmentIndex = i;
  }

  return segments;
}

/** Build a WallSegment from a list of edge indices. */
function buildSegment(
  index: number,
  edgeIndices: number[],
  polygon: Point2D[],
  edgeLengths: number[],
): WallSegment {
  const n = polygon.length;
  const totalLength = edgeIndices.reduce((sum, ei) => sum + edgeLengths[ei]!, 0);

  // Direction: from the start of the first edge to the end of the last edge
  const firstEdge = edgeIndices[0]!;
  const lastEdge = edgeIndices[edgeIndices.length - 1]!;
  const start = polygon[firstEdge]!;
  const end = polygon[(lastEdge + 1) % n]!;

  // Cardinal direction based on outward normal (midpoint relative to centroid)
  const mx = (start.x + end.x) / 2;
  const my = (start.y + end.y) / 2;
  const cx = polygon.reduce((s, p) => s + p.x, 0) / n;
  const cy = polygon.reduce((s, p) => s + p.y, 0) / n;
  const nx = mx - cx;
  const ny = my - cy;
  const direction = Math.abs(nx) > Math.abs(ny)
    ? (nx > 0 ? "Oost" : "West")
    : (ny > 0 ? "Zuid" : "Noord");

  return { segmentIndex: index, edgeIndices, length: totalLength, direction };
}

/**
 * Map each edge index to its parent segment index.
 * Returns a Map<edgeIndex, segmentIndex>.
 */
export function edgeToSegmentMap(segments: WallSegment[]): Map<number, number> {
  const map = new Map<number, number>();
  for (const seg of segments) {
    for (const ei of seg.edgeIndices) {
      map.set(ei, seg.segmentIndex);
    }
  }
  return map;
}

export function polygonCenter(polygon: Point2D[]): Point2D {
  const n = polygon.length;
  return {
    x: polygon.reduce((s, p) => s + p.x, 0) / n,
    y: polygon.reduce((s, p) => s + p.y, 0) / n,
  };
}

/** Shoelace formula — returns positive area in mm². */
export function polygonArea(polygon: Point2D[]): number {
  return Math.abs(signedPolygonArea(polygon));
}

/** Signed area (positive for CW in screen coords / Y-down). */
export function signedPolygonArea(polygon: Point2D[]): number {
  let area = 0;
  const n = polygon.length;
  for (let i = 0; i < n; i++) {
    const a = polygon[i]!;
    const b = polygon[(i + 1) % n]!;
    area += a.x * b.y - b.x * a.y;
  }
  return area / 2;
}

/**
 * Offset a 2D polygon outward by `dist` mm.
 * Uses mitered corners for clean joins.
 */
export function offsetPolygon(polygon: Point2D[], dist: number): Point2D[] {
  const n = polygon.length;
  const area = signedPolygonArea(polygon);
  // CW in screen coords (area > 0): outward normal of edge (dx,dy) is (dy,-dx)/len
  const sign = area > 0 ? 1 : -1;

  const result: Point2D[] = [];

  for (let i = 0; i < n; i++) {
    const prev = polygon[(i - 1 + n) % n]!;
    const curr = polygon[i]!;
    const next = polygon[(i + 1) % n]!;

    const e1dx = curr.x - prev.x;
    const e1dy = curr.y - prev.y;
    const e1len = Math.hypot(e1dx, e1dy);

    const e2dx = next.x - curr.x;
    const e2dy = next.y - curr.y;
    const e2len = Math.hypot(e2dx, e2dy);

    if (e1len < 0.1 || e2len < 0.1) {
      result.push({ x: curr.x, y: curr.y });
      continue;
    }

    const n1x = sign * e1dy / e1len;
    const n1y = sign * (-e1dx) / e1len;
    const n2x = sign * e2dy / e2len;
    const n2y = sign * (-e2dx) / e2len;

    const mx = n1x + n2x;
    const my = n1y + n2y;
    const mlen = Math.hypot(mx, my);

    if (mlen < 0.001) {
      result.push({ x: curr.x + n1x * dist, y: curr.y + n1y * dist });
    } else {
      const dot = n1x * (mx / mlen) + n1y * (my / mlen);
      const miterScale = Math.abs(dot) > 0.25 ? dist / dot : dist * 2;
      result.push({
        x: curr.x + (mx / mlen) * miterScale,
        y: curr.y + (my / mlen) * miterScale,
      });
    }
  }

  return result;
}

/** Ray-casting point-in-polygon test. */
export function pointInPolygon(p: Point2D, polygon: Point2D[]): boolean {
  let inside = false;
  const n = polygon.length;
  for (let i = 0, j = n - 1; i < n; j = i++) {
    const pi = polygon[i]!;
    const pj = polygon[j]!;
    if (
      pi.y > p.y !== pj.y > p.y &&
      p.x < ((pj.x - pi.x) * (p.y - pi.y)) / (pj.y - pi.y) + pi.x
    ) {
      inside = !inside;
    }
  }
  return inside;
}

/** Check if two collinear segments share a non-trivial overlap. */
export function segmentsShareEdge(
  a: Point2D, b: Point2D,
  c: Point2D, d: Point2D,
): boolean {
  // Use perpendicular distance (not raw cross product) so tolerance is
  // independent of wall length. 10 mm perpendicular tolerance catches
  // typical BIM/IFC offsets; 20 mm overlap is enough for small geometry gaps.
  const PERP_TOL = 10;
  const OVERLAP_TOL = 20;

  const abLen = Math.hypot(b.x - a.x, b.y - a.y);
  if (abLen < 1) return false;

  const cross1 = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
  const cross2 = (b.x - a.x) * (d.y - a.y) - (b.y - a.y) * (d.x - a.x);
  if (Math.abs(cross1) / abLen > PERP_TOL || Math.abs(cross2) / abLen > PERP_TOL) return false;

  const horiz = Math.abs(b.x - a.x) > Math.abs(b.y - a.y);
  const [a1, b1] = horiz
    ? [Math.min(a.x, b.x), Math.max(a.x, b.x)]
    : [Math.min(a.y, b.y), Math.max(a.y, b.y)];
  const [c1, d1] = horiz
    ? [Math.min(c.x, d.x), Math.max(c.x, d.x)]
    : [Math.min(c.y, d.y), Math.max(c.y, d.y)];

  return Math.min(b1, d1) - Math.max(a1, c1) > OVERLAP_TOL;
}

/**
 * Compute shared wall edges between rooms.
 * Returns a Set of "roomId:wallIndex" keys for edges shared with another room.
 * Excludes nested rooms (one inside the other) — only truly adjacent rooms count.
 */
export function getSharedEdges(rooms: ModelRoom[]): Set<string> {
  const shared = new Set<string>();
  for (let i = 0; i < rooms.length; i++) {
    const ri = rooms[i]!;
    const ni = ri.polygon.length;
    for (let j = i + 1; j < rooms.length; j++) {
      const rj = rooms[j]!;
      const nj = rj.polygon.length;

      // Skip if one room's center is inside the other (nested, not adjacent)
      const ci = polygonCenter(ri.polygon);
      const cj = polygonCenter(rj.polygon);
      if (pointInPolygon(ci, rj.polygon) || pointInPolygon(cj, ri.polygon)) continue;

      for (let wi = 0; wi < ni; wi++) {
        const a = ri.polygon[wi]!;
        const b = ri.polygon[(wi + 1) % ni]!;
        for (let wj = 0; wj < nj; wj++) {
          const c = rj.polygon[wj]!;
          const d = rj.polygon[(wj + 1) % nj]!;
          if (segmentsShareEdge(a, b, c, d)) {
            shared.add(`${ri.id}:${wi}`);
            shared.add(`${rj.id}:${wj}`);
          }
        }
      }
    }
  }
  return shared;
}

/**
 * Merge two adjacent room polygons by removing their shared wall.
 * Returns the merged polygon, or null if the rooms don't share exactly one edge pair.
 *
 * @param polyA - First room polygon
 * @param wallA - Edge index in polyA that is shared
 * @param polyB - Second room polygon
 * @param wallB - Edge index in polyB that is shared
 */
export function mergePolygons(
  polyA: Point2D[],
  wallA: number,
  polyB: Point2D[],
  wallB: number,
): Point2D[] | null {
  const nA = polyA.length;
  const nB = polyB.length;
  if (nA < 3 || nB < 3) return null;

  // Build merged polygon:
  // Walk A from wallA+2 around to wallA (skip shared edge endpoint wallA+1)
  // Then walk B from wallB+2 around to wallB (skip shared edge endpoint wallB+1)
  // Note: A[wallA]→A[wallA+1] shares with B[wallB+1]←B[wallB] (reversed direction)

  const merged: Point2D[] = [];

  // Walk polyA: start after shared edge, go around to the start of shared edge
  for (let step = 0; step < nA - 1; step++) {
    const i = (wallA + 1 + step) % nA;
    merged.push(polyA[i]!);
  }

  // Walk polyB: start after shared edge, go around to the start of shared edge
  for (let step = 0; step < nB - 1; step++) {
    const i = (wallB + 1 + step) % nB;
    merged.push(polyB[i]!);
  }

  // Clean up near-duplicate and collinear vertices
  const cleaned = removeCollinearVertices(removeNearDuplicateVertices(merged));

  if (cleaned.length < 3) return null;
  if (polygonArea(cleaned) < 100) return null;

  return cleaned;
}

/**
 * Remove collinear vertices that lie on a straight line between their neighbours.
 * Uses cross-product test: if |cross(prev→curr, prev→next)| ≤ tolerance the vertex is redundant.
 * Tolerance is in mm² (cross product = parallelogram area).
 */
export function removeCollinearVertices(poly: Point2D[], tolerance = 1): Point2D[] {
  const result: Point2D[] = [];
  const n = poly.length;
  for (let i = 0; i < n; i++) {
    const prev = poly[(i - 1 + n) % n]!;
    const curr = poly[i]!;
    const next = poly[(i + 1) % n]!;
    const cross = (curr.x - prev.x) * (next.y - prev.y)
                - (curr.y - prev.y) * (next.x - prev.x);
    if (Math.abs(cross) > tolerance) {
      result.push(curr);
    }
  }
  return result.length >= 3 ? result : poly;
}

/** Remove consecutive vertices that are nearly at the same position (< threshold mm apart). */
function removeNearDuplicateVertices(poly: Point2D[], threshold = 50): Point2D[] {
  const result: Point2D[] = [];
  const n = poly.length;
  for (let i = 0; i < n; i++) {
    const next = poly[(i + 1) % n]!;
    const d = Math.hypot(poly[i]!.x - next.x, poly[i]!.y - next.y);
    if (d >= threshold) {
      result.push(poly[i]!);
    }
  }
  return result;
}

/**
 * Split a room polygon along a path between two edge points.
 * The path can be a straight line (no intermediate points) or a polyline.
 * Returns two new polygons, or null if the split is invalid.
 *
 * @param polygon - Room polygon vertices
 * @param edgeA - Index of the first edge to split on
 * @param tA - Parametric position (0..1) along edgeA
 * @param edgeB - Index of the second edge to split on
 * @param tB - Parametric position (0..1) along edgeB
 * @param intermediatePoints - Optional points between splitA and splitB
 */
export function splitPolygon(
  polygon: Point2D[],
  edgeA: number,
  tA: number,
  edgeB: number,
  tB: number,
  intermediatePoints: Point2D[] = [],
): [Point2D[], Point2D[]] | null {
  const n = polygon.length;
  if (edgeA === edgeB) return null;
  if (n < 3) return null;

  // Compute split points
  const a1 = polygon[edgeA]!;
  const a2 = polygon[(edgeA + 1) % n]!;
  const splitA: Point2D = {
    x: Math.round(a1.x + (a2.x - a1.x) * tA),
    y: Math.round(a1.y + (a2.y - a1.y) * tA),
  };

  const b1 = polygon[edgeB]!;
  const b2 = polygon[(edgeB + 1) % n]!;
  const splitB: Point2D = {
    x: Math.round(b1.x + (b2.x - b1.x) * tB),
    y: Math.round(b1.y + (b2.y - b1.y) * tB),
  };

  // Ensure edgeA < edgeB for consistent traversal
  let eA = edgeA, eB = edgeB;
  let pA = splitA, pB = splitB;
  // Path from A to B — reverse if we swap edges
  let path = intermediatePoints;
  if (eA > eB) {
    [eA, eB] = [eB, eA];
    [pA, pB] = [pB, pA];
    path = [...intermediatePoints].reverse();
  }

  // Polygon 1: pA → path → pB → polygon vertices from eB+1 to eA
  const poly1: Point2D[] = [pA, ...path, pB];
  for (let i = (eB + 1) % n; i !== (eA + 1) % n; i = (i + 1) % n) {
    poly1.push(polygon[i]!);
  }

  // Polygon 2: pB → reverse(path) → pA → polygon vertices from eA+1 to eB
  const reversePath = [...path].reverse();
  const poly2: Point2D[] = [pB, ...reversePath, pA];
  for (let i = (eA + 1) % n; i !== (eB + 1) % n; i = (i + 1) % n) {
    poly2.push(polygon[i]!);
  }

  // Validate: both polygons should have at least 3 vertices and positive area
  if (poly1.length < 3 || poly2.length < 3) return null;
  if (polygonArea(poly1) < 100 || polygonArea(poly2) < 100) return null;

  return [poly1, poly2];
}
