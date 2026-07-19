/**
 * Geometry helpers, ported 1:1 from the fase-1 PoC (`lib/geom.mjs`).
 *
 * IMPORTANT convention note (discovered empirically on the PoC's acceptance
 * model, see orchestrator session notes / RAPPORT-fase1.md): web-ifc's
 * `FlatMesh.flatTransformation` composes the FULL placement chain
 * (IfcLocalPlacement hierarchy) into a matrix that lands vertices in a
 * **Y-up, metre** frame (the three.js/glTF convention: Z-up mm input ->
 * x,y,z -> x,z,-y). That axis permutation is UNDOCUMENTED in web-ifc's own
 * types and breaks silently on a version upgrade that changes it — hence the
 * dedicated `toZUpMM` unit test (see geom.test.ts). We convert BACK to a
 * Z-up millimetre frame so the "z > 0.7 = ceiling / z < -0.7 = floor"
 * classification convention (used elsewhere, e.g. the pyrevit scanner)
 * applies directly, and so reported heights line up with storey elevations
 * parsed from IFCBUILDINGSTOREY (also Z-up mm, IFC-native).
 */
import type { Vec3 } from "./types";

/** Transform a local-space point (mm, Z-up, as returned by GetVertexArray)
 * into world Z-up millimetres using a web-ifc flatTransformation (col-major
 * 4x4, Y-up metres). */
export function toZUpMM(m: readonly number[], x: number, y: number, z: number): Vec3 {
  const xw = m[0]! * x + m[4]! * y + m[8]! * z + m[12]!;
  const yw = m[1]! * x + m[5]! * y + m[9]! * z + m[13]!;
  const zw = m[2]! * x + m[6]! * y + m[10]! * z + m[14]!;
  return [xw * 1000, -zw * 1000, yw * 1000];
}

/** Transform a local-space direction (normal) into world Z-up, normalized. */
export function transformNormalZUp(m: readonly number[], nx: number, ny: number, nz: number): Vec3 {
  const xw = m[0]! * nx + m[4]! * ny + m[8]! * nz;
  const yw = m[1]! * nx + m[5]! * ny + m[9]! * nz;
  const zw = m[2]! * nx + m[6]! * ny + m[10]! * nz;
  const ox = xw;
  const oy = -zw;
  const oz = yw;
  const len = Math.hypot(ox, oy, oz) || 1;
  return [ox / len, oy / len, oz / len];
}

export function sub(a: Vec3, b: Vec3): Vec3 {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

export function cross(a: Vec3, b: Vec3): Vec3 {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

export function dot(a: Vec3, b: Vec3): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

export function length(a: Vec3): number {
  return Math.hypot(a[0], a[1], a[2]);
}

export function normalize(a: Vec3): Vec3 {
  const l = length(a) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
}

export function add(a: Vec3, b: Vec3): Vec3 {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

export function scale(a: Vec3, s: number): Vec3 {
  return [a[0] * s, a[1] * s, a[2] * s];
}

/**
 * IMPORTANT convention note #2 (found while analysing space boundaries):
 * `api.GetWorldTransformMatrix(modelID, placementExpressId)` (used for
 * IfcLocalPlacement chains, e.g. an IfcSpace's ObjectPlacement) is a
 * DIFFERENT convention from `FlatMesh.flatTransformation` (used for
 * tessellated geometry, see toZUpMM above): it is a plain native-unit (mm),
 * Z-up, col-major 4x4 -- NO unit scaling and NO Y-up axis permutation.
 * Verified empirically: transforming an IfcRelSpaceBoundary's local
 * (space-relative) ConnectionGeometry.Location through this matrix with a
 * PLAIN apply lands it inside the corresponding space's own Z-up-mm bbox
 * (from toZUpMM); running it through toZUpMM's unit/axis conversion instead
 * produces nonsense (double-converted) coordinates.
 */
export function applyMat4Plain(m: readonly number[], p: Vec3): Vec3 {
  return [
    m[0]! * p[0] + m[4]! * p[1] + m[8]! * p[2] + m[12]!,
    m[1]! * p[0] + m[5]! * p[1] + m[9]! * p[2] + m[13]!,
    m[2]! * p[0] + m[6]! * p[1] + m[10]! * p[2] + m[14]!,
  ];
}

/** Same matrix, direction only (no translation), normalized. */
export function applyMat4LinearOnly(m: readonly number[], v: Vec3): Vec3 {
  const o: Vec3 = [
    m[0]! * v[0] + m[4]! * v[1] + m[8]! * v[2],
    m[1]! * v[0] + m[5]! * v[1] + m[9]! * v[2],
    m[2]! * v[0] + m[6]! * v[1] + m[10]! * v[2],
  ];
  return normalize(o);
}

/** Triangle area (mm^2 in, mm^2 out) via cross product / 2. */
export function triangleArea(p0: Vec3, p1: Vec3, p2: Vec3): number {
  return length(cross(sub(p1, p0), sub(p2, p0))) / 2;
}

export function triangleNormal(p0: Vec3, p1: Vec3, p2: Vec3): Vec3 {
  return normalize(cross(sub(p1, p0), sub(p2, p0)));
}

/**
 * Moeller-Trumbore ray-triangle intersection.
 * Returns distance t along ray (origin + t*dir), or null if no forward hit.
 */
export function rayTriangleIntersect(
  origin: Vec3,
  dir: Vec3,
  p0: Vec3,
  p1: Vec3,
  p2: Vec3,
  epsilon = 1e-6,
): number | null {
  const edge1 = sub(p1, p0);
  const edge2 = sub(p2, p0);
  const h = cross(dir, edge2);
  const a = dot(edge1, h);
  if (Math.abs(a) < epsilon) return null; // parallel
  const f = 1 / a;
  const s = sub(origin, p0);
  const u = f * dot(s, h);
  if (u < -1e-9 || u > 1 + 1e-9) return null;
  const q = cross(s, edge1);
  const v = f * dot(dir, q);
  if (v < -1e-9 || u + v > 1 + 1e-9) return null;
  const t = f * dot(edge2, q);
  if (t <= epsilon) return null;
  return t;
}

/** Convex hull (2D, monotone chain) of a point set [[x,y],...]. */
export function convexHull2D(points: readonly [number, number][]): [number, number][] {
  const seen = new Set<string>();
  const pts: [number, number][] = [];
  for (const p of points) {
    const key = `${p[0]},${p[1]}`;
    if (seen.has(key)) continue;
    seen.add(key);
    pts.push(p);
  }
  pts.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  if (pts.length < 3) return pts;

  const cross2 = (o: [number, number], a: [number, number], b: [number, number]) =>
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);

  const lower: [number, number][] = [];
  for (const p of pts) {
    while (
      lower.length >= 2 &&
      cross2(lower[lower.length - 2]!, lower[lower.length - 1]!, p) <= 0
    ) {
      lower.pop();
    }
    lower.push(p);
  }
  const upper: [number, number][] = [];
  for (let i = pts.length - 1; i >= 0; i--) {
    const p = pts[i]!;
    while (
      upper.length >= 2 &&
      cross2(upper[upper.length - 2]!, upper[upper.length - 1]!, p) <= 0
    ) {
      upper.pop();
    }
    upper.push(p);
  }
  lower.pop();
  upper.pop();
  return lower.concat(upper);
}

/** 2D polygon area (shoelace), points = [[u,v], ...]. Absolute value. */
export function polygon2DArea(points: readonly [number, number][]): number {
  let sum = 0;
  const n = points.length;
  for (let i = 0; i < n; i++) {
    const [x1, y1] = points[i]!;
    const [x2, y2] = points[(i + 1) % n]!;
    sum += x1 * y2 - x2 * y1;
  }
  return Math.abs(sum) / 2;
}

/**
 * Shoelace area of a polygon with holes: outer boundary area minus the sum
 * of each hole's area. Used for IfcRelSpaceBoundary faces, which report an
 * OuterBoundary + zero or more InnerBoundaries (e.g. a wall boundary with a
 * window/door punched out of it, or -- less commonly -- an actual hole in a
 * slab). Assumes outer and holes are each a simple (non-self-intersecting)
 * polygon in the same 2D basis; no containment/overlap validation is done.
 */
export function polygon2DAreaWithHoles(
  outer: readonly [number, number][],
  holes: readonly (readonly [number, number][])[],
): number {
  let area = polygon2DArea(outer);
  for (const hole of holes) {
    if (hole.length) area -= polygon2DArea(hole);
  }
  return area;
}
