/** Shared geometry helpers for the modeller. */
import type { ModelRoom, Point2D } from "./types";

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
  const eps = 1;
  const cross1 = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
  const cross2 = (b.x - a.x) * (d.y - a.y) - (b.y - a.y) * (d.x - a.x);
  if (Math.abs(cross1) > eps || Math.abs(cross2) > eps) return false;

  const horiz = Math.abs(b.x - a.x) > Math.abs(b.y - a.y);
  const [a1, b1] = horiz
    ? [Math.min(a.x, b.x), Math.max(a.x, b.x)]
    : [Math.min(a.y, b.y), Math.max(a.y, b.y)];
  const [c1, d1] = horiz
    ? [Math.min(c.x, d.x), Math.max(c.x, d.x)]
    : [Math.min(c.y, d.y), Math.max(c.y, d.y)];

  return Math.min(b1, d1) - Math.max(a1, c1) > eps;
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
 * Split a room polygon along a line between two edge points.
 * Returns two new polygons, or null if the split is invalid.
 *
 * @param polygon - Room polygon vertices
 * @param edgeA - Index of the first edge to split on
 * @param tA - Parametric position (0..1) along edgeA
 * @param edgeB - Index of the second edge to split on
 * @param tB - Parametric position (0..1) along edgeB
 */
export function splitPolygon(
  polygon: Point2D[],
  edgeA: number,
  tA: number,
  edgeB: number,
  tB: number,
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
  if (eA > eB) {
    [eA, eB] = [eB, eA];
    [pA, pB] = [pB, pA];
  }

  // Polygon 1: vertices from after eA split to eB split
  // Walk: pA → vertices[eA+1] → ... → vertices[eB] → pB → pA
  const poly1: Point2D[] = [pA];
  for (let i = (eA + 1) % n; i !== (eB + 1) % n; i = (i + 1) % n) {
    poly1.push(polygon[i]!);
  }
  poly1.push(pB);

  // Polygon 2: vertices from after eB split to eA split
  // Walk: pB → vertices[eB+1] → ... → vertices[eA] → pA → pB
  const poly2: Point2D[] = [pB];
  for (let i = (eB + 1) % n; i !== (eA + 1) % n; i = (i + 1) % n) {
    poly2.push(polygon[i]!);
  }
  poly2.push(pA);

  // Validate: both polygons should have at least 3 vertices and positive area
  if (poly1.length < 3 || poly2.length < 3) return null;
  if (polygonArea(poly1) < 100 || polygonArea(poly2) < 100) return null;

  return [poly1, poly2];
}
