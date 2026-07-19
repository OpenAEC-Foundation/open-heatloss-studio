/**
 * Raycasting against tagged triangles + a uniform-grid spatial index, ported
 * from the fase-1 PoC (`lib/raycast.mjs` + `lib/spatialgrid.mjs`).
 */
import { add, rayTriangleIntersect, scale } from "./geom";
import type { Triangle, Vec3 } from "./types";

export type TriangleTag =
  | { kind: "element"; id: number; ifcType: string }
  | { kind: "space"; id: number };

export interface TaggedTriangle {
  tag: TriangleTag;
  tri: Triangle;
}

export interface RayHit {
  t: number;
  tag: TriangleTag;
  point: Vec3;
}

export interface RayClassification {
  host: number | null;
  hostDistance: number | null;
  thicknessMM: number | null;
  /** "exterior" or "space:<id>". */
  classification: string;
  ambiguous: boolean;
  hitCount: number;
}

export interface ClassifyRayOptions {
  hostMaxDist?: number;
  totalMaxDist?: number;
  maxHostSpan?: number;
}

/** Cast one ray against a flat list of tagged triangles, return ALL hits sorted by t. */
export function raycastAll(
  origin: Vec3,
  dir: Vec3,
  taggedTriangles: readonly TaggedTriangle[],
  maxDist: number,
): RayHit[] {
  const hits: RayHit[] = [];
  for (const tt of taggedTriangles) {
    const t = rayTriangleIntersect(origin, dir, tt.tri.p0, tt.tri.p1, tt.tri.p2);
    if (t !== null && t <= maxDist) {
      hits.push({ t, tag: tt.tag, point: add(origin, scale(dir, t)) });
    }
  }
  hits.sort((a, b) => a.t - b.t);
  return hits;
}

/**
 * Per-sample classification walk:
 *  1. First hit among element-tagged triangles within hostMaxDist -> host element.
 *  2. Continue past all consecutive hits belonging to that SAME host id (its
 *     near+far face, or stacked layers sharing the id) until either:
 *       - a space-tagged hit -> adjacency = that space
 *       - hits run out within totalMaxDist -> exterior
 *       - a DIFFERENT element id keeps appearing with no space in between,
 *         up to totalMaxDist -> exterior (best-effort; ambiguous case flagged)
 */
export function classifyRay(
  origin: Vec3,
  dir: Vec3,
  taggedTriangles: readonly TaggedTriangle[],
  { hostMaxDist = 1500, totalMaxDist = 5000, maxHostSpan = 500 }: ClassifyRayOptions = {},
): RayClassification {
  const hits = raycastAll(origin, dir, taggedTriangles, totalMaxDist);
  if (hits.length === 0) {
    return {
      host: null,
      hostDistance: null,
      thicknessMM: null,
      classification: "exterior",
      ambiguous: false,
      hitCount: 0,
    };
  }
  const firstElementHit = hits.find((h) => h.tag.kind === "element" && h.t <= hostMaxDist);
  if (!firstElementHit) {
    // Something was hit (maybe a space directly, meaning near-zero-thickness
    // boundary, or an element beyond hostMaxDist) but no host within range.
    const firstSpaceHit = hits.find((h) => h.tag.kind === "space");
    if (firstSpaceHit && firstSpaceHit.t <= hostMaxDist && firstSpaceHit.tag.kind === "space") {
      return {
        host: null,
        hostDistance: null,
        thicknessMM: null,
        classification: `space:${firstSpaceHit.tag.id}`,
        ambiguous: true,
        hitCount: hits.length,
      };
    }
    return {
      host: null,
      hostDistance: null,
      thicknessMM: null,
      classification: "exterior",
      ambiguous: true,
      hitCount: hits.length,
    };
  }
  if (firstElementHit.tag.kind !== "element") {
    throw new Error("unreachable: firstElementHit must be an element tag");
  }
  const hostId = firstElementHit.tag.id;

  // Real-world walls/corners often have OTHER elements' surfaces coincident
  // with (or interleaved right next to) the host's own near/far faces, e.g.
  // where two walls overlap at a corner join. So rather than breaking the
  // walk on the first non-host hit (fragile against ties), take the exit
  // point as the FURTHEST hit still tagged with the same host id, bounded to
  // a plausible single-element thickness (maxHostSpan) so we don't
  // accidentally grab an unrelated far-away hit on the same expressID.
  let exitT = firstElementHit.t;
  let sawSecondHostHit = false;
  for (const h of hits) {
    if (h.tag.kind === "element" && h.tag.id === hostId && h.t <= firstElementHit.t + maxHostSpan) {
      if (h.t > exitT) sawSecondHostHit = true;
      exitT = Math.max(exitT, h.t);
    }
  }
  // If we never found a distinct far-face hit for this host (edge/corner
  // graze, or an open/leaky mesh), thickness is unmeasurable here; report
  // null rather than a misleading "0 mm".
  const thicknessMM = sawSecondHostHit ? exitT - firstElementHit.t : null;

  let ambiguous = false;
  for (const h of hits) {
    if (h.t <= exitT + 1e-6) continue; // still within/at the host's own span
    if (h.tag.kind === "space") {
      return {
        host: hostId,
        hostDistance: firstElementHit.t,
        thicknessMM,
        classification: `space:${h.tag.id}`,
        ambiguous,
        hitCount: hits.length,
      };
    }
    if (h.tag.kind === "element" && h.tag.id !== hostId) {
      // A different element beyond the host (next construction layer /
      // neighbouring element) -- keep walking, but flag as no longer a
      // clean single-host read.
      ambiguous = true;
    }
  }
  return {
    host: hostId,
    hostDistance: firstElementHit.t,
    thicknessMM,
    classification: "exterior",
    ambiguous,
    hitCount: hits.length,
  };
}

/**
 * Uniform-grid spatial hash for raycast candidate triangles.
 *
 * Buckets tagged triangles into cells by their bounding box; a ray query
 * walks the ray in `step`-sized hops (matching the sample spacing)
 * collecting the union of the 3x3x3 cell neighbourhood at each hop -- cheap,
 * no external deps, good enough for architectural geometry where triangles
 * are small relative to the cell size. Measured 6.4x faster than brute force
 * on the PoC's acceptance model (see RAPPORT-fase1.md §1.5).
 */
export class SpatialGrid {
  private readonly cellSize: number;
  private readonly cells = new Map<string, TaggedTriangle[]>();

  constructor(taggedTriangles: readonly TaggedTriangle[], cellSizeMM = 500) {
    this.cellSize = cellSizeMM;
    for (const tt of taggedTriangles) this.insert(tt);
  }

  private insert(tt: TaggedTriangle): void {
    const { p0, p1, p2 } = tt.tri;
    const minX = Math.min(p0[0], p1[0], p2[0]);
    const maxX = Math.max(p0[0], p1[0], p2[0]);
    const minY = Math.min(p0[1], p1[1], p2[1]);
    const maxY = Math.max(p0[1], p1[1], p2[1]);
    const minZ = Math.min(p0[2], p1[2], p2[2]);
    const maxZ = Math.max(p0[2], p1[2], p2[2]);
    const cx0 = Math.floor(minX / this.cellSize);
    const cx1 = Math.floor(maxX / this.cellSize);
    const cy0 = Math.floor(minY / this.cellSize);
    const cy1 = Math.floor(maxY / this.cellSize);
    const cz0 = Math.floor(minZ / this.cellSize);
    const cz1 = Math.floor(maxZ / this.cellSize);
    for (let cx = cx0; cx <= cx1; cx++) {
      for (let cy = cy0; cy <= cy1; cy++) {
        for (let cz = cz0; cz <= cz1; cz++) {
          const key = `${cx},${cy},${cz}`;
          let arr = this.cells.get(key);
          if (!arr) {
            arr = [];
            this.cells.set(key, arr);
          }
          arr.push(tt);
        }
      }
    }
  }

  /**
   * Union of candidate triangles in cells the ray plausibly passes through,
   * up to maxDist, deduplicated. Not a strict guarantee against missing a
   * triangle that straddles a cell boundary far from any sampled hop point
   * -- acceptable given architectural triangles are small vs. a 500mm cell
   * and the 3x3x3 neighbourhood margin, but noted as an approximation.
   */
  queryRay(origin: Vec3, dir: Vec3, maxDist: number, step = 250): TaggedTriangle[] {
    const seen = new Set<TaggedTriangle>();
    const result: TaggedTriangle[] = [];
    const nSteps = Math.max(1, Math.ceil(maxDist / step));
    for (let i = 0; i <= nSteps; i++) {
      const t = Math.min(i * step, maxDist);
      const px = origin[0] + dir[0] * t;
      const py = origin[1] + dir[1] * t;
      const pz = origin[2] + dir[2] * t;
      const cx = Math.floor(px / this.cellSize);
      const cy = Math.floor(py / this.cellSize);
      const cz = Math.floor(pz / this.cellSize);
      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          for (let dz = -1; dz <= 1; dz++) {
            const arr = this.cells.get(`${cx + dx},${cy + dy},${cz + dz}`);
            if (!arr) continue;
            for (const tt of arr) {
              if (seen.has(tt)) continue;
              seen.add(tt);
              result.push(tt);
            }
          }
        }
      }
      if (t >= maxDist) break;
    }
    return result;
  }

  get cellCount(): number {
    return this.cells.size;
  }
}
