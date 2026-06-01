import { describe, expect, it } from "vitest";
import { matchIfcMaterial } from "./ifcMaterialMatcher";

/**
 * Regression suite for the IFC/Revit material-name matcher.
 *
 * Core bug (fixed): the pyRevit exporter emits names like `n7_isolatie_PIR`
 * (no λ — the tool resolves λ here). The generic keyword "isolatie" on
 * "Isolerende mortel" (λ 0.12) used to out-score the specific "PIR" keyword
 * (λ 0.023), giving a ~5× too high λ. The fix removes that generic keyword and
 * weights exact token==keyword matches over substring hits.
 */
describe("matchIfcMaterial — pyRevit n7_isolatie_* names", () => {
  it("maps n7_isolatie_PIR to PIR (λ 0.023), not Isolerende mortel", () => {
    const match = matchIfcMaterial("n7_isolatie_PIR");
    expect(match.material?.name).toBe("PIR");
    expect(match.material?.lambda).toBe(0.023);
    expect(match.confidence).toBe("keyword");
  });

  it("maps n7_isolatie_resol to Resolschuim (phenol) (λ 0.020)", () => {
    const match = matchIfcMaterial("n7_isolatie_resol");
    expect(match.material?.name).toBe("Resolschuim (phenol)");
    expect(match.material?.lambda).toBe(0.02);
  });

  it("does NOT map n7_isolatie_PIR to Isolerende mortel", () => {
    const match = matchIfcMaterial("n7_isolatie_PIR");
    expect(match.material?.name).not.toBe("Isolerende mortel");
  });
});

describe("matchIfcMaterial — regressions on common insulation/structure", () => {
  const cases: ReadonlyArray<readonly [string, string]> = [
    ["n7_isolatie_EPS", "EPS"],
    ["n7_isolatie_XPS", "XPS"],
    ["n7_isolatie_PUR", "PUR"],
    ["EPS", "EPS"],
    ["XPS", "XPS"],
    ["PIR", "PIR"],
    ["glaswol", "Glaswol"],
    ["steenwol", "Steenwol hoge dichtheid"],
    ["beton gewapend", "Beton gewapend"],
    ["naaldhout", "Naaldhout"],
    ["OSB", "OSB"],
    ["kalkzandsteen", "Kalkzandsteen"],
  ];

  for (const [ifcName, expected] of cases) {
    it(`matches "${ifcName}" → "${expected}"`, () => {
      const match = matchIfcMaterial(ifcName);
      expect(match.material?.name).toBe(expected);
    });
  }

  it("still matches a genuine isolerende mortel name", () => {
    const match = matchIfcMaterial("isolerende mortel");
    expect(match.material?.name).toBe("Isolerende mortel");
  });

  it("returns confidence 'none' for an unknown material", () => {
    const match = matchIfcMaterial("zomaar_iets_onbekends_xyzzy");
    expect(match.material).toBeNull();
    expect(match.confidence).toBe("none");
  });
});
