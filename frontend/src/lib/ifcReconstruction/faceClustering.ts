/**
 * Connectivity-aware planar face clustering, ported from the fase-1 PoC
 * (`lib/faces.mjs::groupPlanarFacesConnected`). This is the flood-fill
 * clustering the PoC settled on — the earlier greedy normal+distance-only
 * matcher (`groupPlanarFaces`) is intentionally NOT ported: the PoC's own
 * finding (RAPPORT-fase1.md §1.1) was that it risks bridging two physically
 * disjoint but coplanar mesh regions, and the flood-fill version fully
 * subsumes it (identical output when there's nothing to disconnect, strictly
 * better when there is).
 */
import { add, dot, normalize, scale } from "./geom";
import type { GeometricOrientation, Triangle, Vec3 } from "./types";

export interface Face {
  normal: Vec3;
  distance: number;
  area: number;
  centroid: Vec3;
  triangles: Triangle[];
}

interface GrowingFace {
  normal: Vec3;
  distance: number;
  area: number;
  normalAccum: Vec3;
  centroidAccum: Vec3;
  triangles: Triangle[];
}

function centroidOf(t: Triangle): Vec3 {
  return [
    (t.p0[0] + t.p1[0] + t.p2[0]) / 3,
    (t.p0[1] + t.p1[1] + t.p2[1]) / 3,
    (t.p0[2] + t.p1[2] + t.p2[2]) / 3,
  ];
}

/**
 * Connectivity-aware clustering (flood-fill on shared mesh edges/vertices).
 *
 * Root cause it fixes: a space's own boundary mesh is one continuous
 * manifold, but a "flat run" of that mesh can legitimately be interrupted by
 * a small jog/recess/step (e.g. where a facade changes construction) while
 * remaining coplanar within tolerance -- a normal+distance-only matcher
 * would bridge across that jog purely because two DISCONNECTED patches
 * happen to be coplanar, silently lumping multiple real host elements into
 * one "face". Flood-fill only ever grows a face through a chain of triangles
 * that actually share an edge in the tessellation, so a physical break (even
 * a tiny one) in the room's own boundary naturally ends a face.
 */
export function groupPlanarFacesConnected(
  triangles: Triangle[],
  angleTolDeg = 1,
  distTolMM = 10,
  vertexWeldMM = 0.5,
): Face[] {
  const cosTol = Math.cos((angleTolDeg * Math.PI) / 180);
  const n = triangles.length;
  if (n === 0) return [];

  // Weld vertices (round to vertexWeldMM) to a canonical id so shared edges
  // across triangles (which each carry their own copies of p0/p1/p2) are
  // detected.
  const vertexIds = new Map<string, number>();
  function vid(p: Vec3): number {
    const key = `${Math.round(p[0] / vertexWeldMM)},${Math.round(p[1] / vertexWeldMM)},${Math.round(p[2] / vertexWeldMM)}`;
    let id = vertexIds.get(key);
    if (id === undefined) {
      id = vertexIds.size;
      vertexIds.set(key, id);
    }
    return id;
  }

  const triVerts: [number, number, number][] = new Array(n);
  const edgeMap = new Map<string, number[]>();
  for (let i = 0; i < n; i++) {
    const t = triangles[i]!;
    const ids: [number, number, number] = [vid(t.p0), vid(t.p1), vid(t.p2)];
    triVerts[i] = ids;
    const edges: [number, number][] = [
      [ids[0], ids[1]],
      [ids[1], ids[2]],
      [ids[2], ids[0]],
    ];
    for (const [a, b] of edges) {
      const key = a < b ? `${a},${b}` : `${b},${a}`;
      let arr = edgeMap.get(key);
      if (!arr) {
        arr = [];
        edgeMap.set(key, arr);
      }
      arr.push(i);
    }
  }

  const visited: boolean[] = new Array(n).fill(false);
  const faces: Face[] = [];

  function compatible(face: GrowingFace, tri: Triangle): boolean {
    if (dot(face.normal, tri.normal) <= cosTol) return false;
    const c = centroidOf(tri);
    const d = dot(face.normal, c);
    return Math.abs(d - face.distance) < distTolMM;
  }

  for (let seed = 0; seed < n; seed++) {
    if (visited[seed]) continue;
    const seedTri = triangles[seed]!;
    const face: GrowingFace = {
      normal: seedTri.normal,
      distance: dot(seedTri.normal, centroidOf(seedTri)),
      area: 0,
      normalAccum: [0, 0, 0],
      centroidAccum: [0, 0, 0],
      triangles: [],
    };
    visited[seed] = true;
    const queue: number[] = [seed];
    while (queue.length) {
      const i = queue.pop()!;
      const t = triangles[i]!;
      const c = centroidOf(t);
      face.normalAccum = add(face.normalAccum, scale(t.normal, t.area));
      face.normal = normalize(face.normalAccum);
      face.centroidAccum = add(face.centroidAccum, scale(c, t.area));
      face.area += t.area;
      face.distance = dot(face.normal, scale(face.centroidAccum, 1 / face.area));
      face.triangles.push(t);

      const ids = triVerts[i]!;
      const edges: [number, number][] = [
        [ids[0], ids[1]],
        [ids[1], ids[2]],
        [ids[2], ids[0]],
      ];
      for (const [a, b] of edges) {
        const key = a < b ? `${a},${b}` : `${b},${a}`;
        const neighbours = edgeMap.get(key) ?? [];
        for (const j of neighbours) {
          if (visited[j]) continue;
          if (compatible(face, triangles[j]!)) {
            visited[j] = true;
            queue.push(j);
          }
        }
      }
    }
    faces.push({
      normal: face.normal,
      distance: face.distance,
      area: face.area,
      triangles: face.triangles,
      centroid: [0, 0, 0], // filled in below
    });
  }

  for (const f of faces) {
    f.centroid = f.triangles.reduce<Vec3>(
      (acc, t) => add(acc, scale(centroidOf(t), t.area / f.area)),
      [0, 0, 0],
    );
  }
  return faces;
}

/** pyrevit-style orientation classification from a (Z-up) unit normal. */
export function classifyOrientation(normal: Vec3): GeometricOrientation {
  if (normal[2] > 0.7) return "plafond";
  if (normal[2] < -0.7) return "vloer";
  return "wand";
}

export interface FaceSample {
  point: Vec3;
  triangle: Triangle;
}

/**
 * Sample points across a face's triangles, targeting ~spacingMM between
 * samples, with a floor of minSamples total.
 */
export function sampleFacePoints(face: Face, spacingMM = 250, minSamples = 5): FaceSample[] {
  const samples: FaceSample[] = [];
  for (const tri of face.triangles) {
    const { p0, p1, p2, area } = tri;
    const nSub = Math.max(1, Math.round(Math.sqrt(area) / spacingMM));
    for (let i = 0; i <= nSub; i++) {
      for (let j = 0; j <= nSub - i; j++) {
        const u = i / nSub;
        const v = j / nSub;
        const w = 1 - u - v;
        if (w < -1e-9) continue;
        samples.push({
          point: [
            w * p0[0] + u * p1[0] + v * p2[0],
            w * p0[1] + u * p1[1] + v * p2[1],
            w * p0[2] + u * p1[2] + v * p2[2],
          ],
          triangle: tri,
        });
      }
    }
  }
  if (samples.length < minSamples) {
    for (const tri of face.triangles) {
      samples.push({
        point: [
          (tri.p0[0] + tri.p1[0] + tri.p2[0]) / 3,
          (tri.p0[1] + tri.p1[1] + tri.p2[1]) / 3,
          (tri.p0[2] + tri.p1[2] + tri.p2[2]) / 3,
        ],
        triangle: tri,
      });
    }
  }
  if (samples.length === 0) {
    samples.push({ point: face.centroid, triangle: face.triangles[0]! });
  }
  return samples;
}
