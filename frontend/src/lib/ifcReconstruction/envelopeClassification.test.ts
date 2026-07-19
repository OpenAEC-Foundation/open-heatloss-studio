/**
 * Tests voor `envelopeClassification.ts` — de maaiveld-splitsing die in de
 * fase-1 PoC twee coordinatenstelsel-valkuilen opleverde (zie
 * RAPPORT-fase1.md §1.4): een vlak dat het maaiveld snijdt moet als
 * "gemengd" met een juiste m²-verdeling worden gerapporteerd, niet stilzwijgend
 * naar één kant worden afgerond.
 */
import { describe, expect, it } from "vitest";

import { classifyGroundSplit } from "./envelopeClassification";
import { triangleArea, triangleNormal } from "./geom";
import type { Triangle, Vec3 } from "./types";

function tri(p0: Vec3, p1: Vec3, p2: Vec3): Triangle {
  return { p0, p1, p2, normal: triangleNormal(p0, p1, p2), area: triangleArea(p0, p1, p2), sourceId: 1 };
}

const MAAIVELD_MM = 1000;

describe("classifyGroundSplit — maaiveld-splitsing", () => {
  it("een vlak volledig onder het maaiveld -> grond, geen mixedSplit", () => {
    const t = tri([0, 0, 900], [1000, 0, 900], [0, 1000, 900]);
    const result = classifyGroundSplit([t], MAAIVELD_MM, 0);
    expect(result.classification).toBe("grond");
    expect(result.mixedSplit).toBeNull();
  });

  it("een vlak volledig boven het maaiveld -> exterieur, geen mixedSplit", () => {
    const t = tri([0, 0, 1200], [1000, 0, 1200], [0, 1000, 1200]);
    const result = classifyGroundSplit([t], MAAIVELD_MM, 0);
    expect(result.classification).toBe("exterieur");
    expect(result.mixedSplit).toBeNull();
  });

  it("een vlak dat het maaiveld snijdt (twee driehoeken, boven+onder) -> gemengd met juiste m2-verdeling", () => {
    const below = tri([0, 0, 900], [1000, 0, 900], [0, 1000, 900]); // 500,000 mm^2
    const above = tri([0, 0, 1200], [1000, 0, 1200], [0, 1000, 1200]); // 500,000 mm^2
    const result = classifyGroundSplit([below, above], MAAIVELD_MM, 0);
    expect(result.classification).toBe("gemengd");
    expect(result.mixedSplit).not.toBeNull();
    expect(result.mixedSplit!.groundM2).toBeCloseTo(below.area / 1e6, 6);
    expect(result.mixedSplit!.exteriorM2).toBeCloseTo(above.area / 1e6, 6);
  });

  it("shiftMM verschuift de toets (vloer-onderkant): een vlak tussen peil en peil+shift wordt 'grond'", () => {
    // z=1100 ligt BOVEN het rauwe maaiveld (1000) maar de vloer-onderkant-shift
    // (200mm) verlegt de vergelijking naar 1200 -- 1100 < 1200 dus "grond".
    const t = tri([0, 0, 1100], [1000, 0, 1100], [0, 1000, 1100]);
    const withoutShift = classifyGroundSplit([t], MAAIVELD_MM, 0);
    const withShift = classifyGroundSplit([t], MAAIVELD_MM, 200);
    expect(withoutShift.classification).toBe("exterieur");
    expect(withShift.classification).toBe("grond");
  });
});
