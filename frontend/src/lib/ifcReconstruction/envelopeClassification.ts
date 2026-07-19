/**
 * Maaiveld (ground-level) split for an exterior-facing face, ported from the
 * fase-1 PoC (`run-fase1.mjs`, step 5e). Pulled into its own pure/no-web-ifc
 * module so it's directly unit-testable against synthetic triangles.
 *
 * A face straddling the maaiveld plane (e.g. a basement wall partly above,
 * partly below grade) is reported as "gemengd" with an area split rather
 * than forced into one bucket.
 */
import type { MixedSplit, Triangle } from "./types";

export interface GroundSplitResult {
  classification: "grond" | "exterieur" | "gemengd";
  mixedSplit: MixedSplit | null;
}

/**
 * Classify a face's triangles against the maaiveld (ground) plane.
 *
 * @param triangles The face's triangles, world Z-up mm.
 * @param maaiveldMM Ground level, world Z-up mm (see pipeline.ts maaiveld resolution).
 * @param shiftMM Additional downward test-point shift (mm) -- used for
 *   zone==="vloer" faces to test the floor construction's UNDERSIDE (peil
 *   minus build-up thickness) rather than its top (room-side) surface, which
 *   sits at/above peil by definition. Pass 0 for wand/dak/plafond faces.
 */
export function classifyGroundSplit(
  triangles: readonly Triangle[],
  maaiveldMM: number,
  shiftMM: number,
): GroundSplitResult {
  const threshold = maaiveldMM + shiftMM;
  const zs = triangles.flatMap((t) => [t.p0[2] - threshold, t.p1[2] - threshold, t.p2[2] - threshold]);
  const allBelow = zs.every((z) => z < 0);
  const allAbove = zs.every((z) => z >= 0);
  if (allBelow) return { classification: "grond", mixedSplit: null };
  if (allAbove) return { classification: "exterieur", mixedSplit: null };

  let belowArea = 0;
  let aboveArea = 0;
  for (const t of triangles) {
    const centroidZ = (t.p0[2] + t.p1[2] + t.p2[2]) / 3 - threshold;
    if (centroidZ < 0) belowArea += t.area;
    else aboveArea += t.area;
  }
  return {
    classification: "gemengd",
    mixedSplit: {
      groundM2: Number((belowArea / 1e6).toFixed(3)),
      exteriorM2: Number((aboveArea / 1e6).toFixed(3)),
    },
  };
}
