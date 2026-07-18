/**
 * Tests voor de hellingbaan-rekenkern (`lib/hellingbaanCalculation.ts`).
 *
 * Dekt: de referentiecase (3600 mm, stalling, MET overgangen, override
 * 16%) inclusief de gedocumenteerde 2770/1385-vs-2720/1360-discrepantie,
 * de drie zones (kort/midden/lang) van de kwadratische optimalisatie, de
 * variant zonder overgangshellingen, en de twee warning-paden (override
 * buiten de norm-bandbreedte, breedte onder het minimum).
 */
import { describe, expect, it } from "vitest";

import type { HellingbaanInput } from "../types/hellingbaan";
import {
  calculateHellingbaan,
  calculateHellingbaanReferentie,
  getGarageType,
  hellingPercentToDegrees,
  isReferentieNormConform,
  LEN_MAX_MM,
  LEN_MIN_MM,
  OVERGANG_TOP_MM,
  OVERGANG_VOET_MM,
} from "./hellingbaanCalculation";

function makeInput(overrides: Partial<HellingbaanInput> = {}): HellingbaanInput {
  return {
    hoogteMm: 3600,
    garageTypeId: "stalling",
    metOvergang: true,
    breedteMm: 2750,
    ...overrides,
  };
}

describe("referentiecase (rekenblad-eigenaar, 3600 mm stalling)", () => {
  it("MET overgangen + override 16% → 24577,5 mm (pyRevit-waarden 2770/1385, wijkt af van excel 24540 mm)", () => {
    // Handmatige controle met de pyRevit-waarden (overgangHelling = 8%):
    //   overgangOnderHoogte = 2770 × 0,08        = 221,6 mm
    //   overgangBovenHoogte = 1385 × 0,08        = 110,8 mm
    //   hoofdHoogte         = 3600 − 221,6 − 110,8 = 3267,6 mm
    //   hoofdLengte         = 3267,6 / 0,16 × 100 = 20422,5 mm
    //   totaal              = 2770 + 20422,5 + 1385 = 24577,5 mm
    //
    // Met de rekenblad-waarden van de eigenaar (2720/1360) komt de excel op
    // 24540 mm uit: overgangOnderHoogte 217,6 + overgangBovenHoogte 108,8 →
    // hoofdHoogte 3273,6 → hoofdLengte 20460 → totaal 2720+20460+1360=24540 mm.
    // Verschil: 37,5 mm, uitsluitend toe te schrijven aan de overgangslengtes
    // (2770/1385 vs. 2720/1360) — zie module-doc-comment in
    // hellingbaanCalculation.ts.
    const result = calculateHellingbaan(
      makeInput({ hellingOverridePercent: 16 }),
    );

    expect(result.isOverride).toBe(true);
    expect(result.hellingPercent).toBe(16);
    expect(result.lengteTotaalMm).toBeCloseTo(24577.5, 6);
    expect(result.isOverrideBuitenZone).toBe(false); // 16% ligt binnen [14%, 24%] van stalling
    expect(result.warnings).toEqual([]);

    const [voet, hoofd, top] = result.segments;
    expect(voet!.lengteMm).toBe(OVERGANG_VOET_MM.value);
    expect(top!.lengteMm).toBe(OVERGANG_TOP_MM.value);
    expect(hoofd!.hellingPercent).toBe(16);
    expect(hoofd!.lengteMm).toBeCloseTo(20422.5, 6);
  });
});

describe("zone-indeling (kwadratische optimalisatie)", () => {
  const stalling = getGarageType("stalling"); // max 24%, min 14%, breedteMin 2750 mm

  it("kort-zone: klein hoogteverschil → helling = max-helling van het type", () => {
    // hoogteMin voor stalling = LEN_MIN × max / 100 = 10000 × 24 / 100 = 2400 mm
    const hoogteMin = (LEN_MIN_MM.value * stalling.maxHellingPercent) / 100;
    const result = calculateHellingbaan(makeInput({ hoogteMm: hoogteMin - 400 }));

    expect(result.zone).toBe("kort");
    expect(result.hellingBerekendPercent).toBe(stalling.maxHellingPercent);
    expect(result.hellingPercent).toBe(stalling.maxHellingPercent);
  });

  it("lang-zone: groot hoogteverschil → helling = min-helling van het type", () => {
    // hoogteMax voor stalling = LEN_MAX × min / 100 = 40000 × 14 / 100 = 5600 mm
    const hoogteMax = (LEN_MAX_MM.value * stalling.minHellingPercent) / 100;
    const result = calculateHellingbaan(makeInput({ hoogteMm: hoogteMax + 400 }));

    expect(result.zone).toBe("lang");
    expect(result.hellingBerekendPercent).toBe(stalling.minHellingPercent);
    expect(result.hellingPercent).toBe(stalling.minHellingPercent);
  });

  it("midden-zone: berekende helling ligt binnen [min, max] en de round-trip klopt met het hoogteverschil", () => {
    const hoogteMm = 3600; // tussen hoogteMin (2400) en hoogteMax (5600) voor stalling
    const result = calculateHellingbaan(makeInput({ hoogteMm }));

    expect(result.zone).toBe("midden");
    expect(result.isOverride).toBe(false);
    expect(result.hellingBerekendPercent).toBeGreaterThanOrEqual(stalling.minHellingPercent);
    expect(result.hellingBerekendPercent).toBeLessThanOrEqual(stalling.maxHellingPercent);

    // Round-trip: de som van de hoogtes van alle segmenten moet het
    // ingevoerde hoogteverschil reconstrueren.
    const hoogteTerug = result.segments.reduce((sum, s) => sum + s.hoogteMm, 0);
    expect(hoogteTerug).toBeCloseTo(hoogteMm, 6);
  });

  it("vast-type (openbaar, min === max): zone 'vast', helling altijd 14%", () => {
    const result = calculateHellingbaan(
      makeInput({ garageTypeId: "openbaar", hoogteMm: 3600, breedteMm: 3000 }),
    );

    expect(result.zone).toBe("vast");
    expect(result.hellingBerekendPercent).toBe(14);
  });
});

describe("isReferentieNormConform (norm-context van de 'zonder optimalisatie'-vergelijking)", () => {
  it("kort/vast/simpel: de vaste max-helling IS norm-conform (dat is letterlijk wat de norm-berekening daar zelf gebruikt)", () => {
    expect(isReferentieNormConform("kort")).toBe(true);
    expect(isReferentieNormConform("vast")).toBe(true);
    expect(isReferentieNormConform("simpel")).toBe(true);
  });

  it("midden/lang: de vaste max-helling is NIET norm-conform (de norm verlangt daar juist een minder steile helling)", () => {
    expect(isReferentieNormConform("midden")).toBe(false);
    expect(isReferentieNormConform("lang")).toBe(false);
  });
});

describe("zonder overgangshellingen", () => {
  it("totale lengte = hoogte / helling × 100, geen overgang-segmenten", () => {
    const result = calculateHellingbaan(
      makeInput({ metOvergang: false, hellingOverridePercent: 20, hoogteMm: 3600 }),
    );

    expect(result.segments).toHaveLength(1);
    expect(result.segments[0]!.type).toBe("enkel");
    expect(result.lengteTotaalMm).toBeCloseTo((3600 / 20) * 100, 6); // 18000 mm
  });

  it("calculateHellingbaanReferentie gebruikt altijd de max-helling van het type, zonder overgang", () => {
    const ref = calculateHellingbaanReferentie({
      hoogteMm: 3600,
      garageTypeId: "stalling",
      metOvergang: false,
    });

    expect(ref.hellingPercent).toBe(24);
    expect(ref.lengteTotaalMm).toBeCloseTo((3600 / 24) * 100, 6); // 15000 mm
  });
});

describe("warnings", () => {
  it("override buiten de norm-bandbreedte van het type → warning + isOverrideBuitenZone", () => {
    // niet_openbaar: min 14%, max 20% — override 10% ligt daaronder.
    const result = calculateHellingbaan(
      makeInput({ garageTypeId: "niet_openbaar", hellingOverridePercent: 10, breedteMm: 2750 }),
    );

    expect(result.isOverride).toBe(true);
    expect(result.isOverrideBuitenZone).toBe(true);
    expect(result.warnings.some((w) => w.includes("bandbreedte"))).toBe(true);
  });

  it("breedte onder de minimumbreedte van het type → warning + isBreedteOnderMinimum", () => {
    const result = calculateHellingbaan(makeInput({ breedteMm: 2000 })); // stalling min 2750 mm

    expect(result.isBreedteOnderMinimum).toBe(true);
    expect(result.warnings.some((w) => w.includes("minimumbreedte"))).toBe(true);
  });

  it("ongeldige override (> 30% of ≤ 0) valt terug op de norm-berekende helling, met warning", () => {
    const result = calculateHellingbaan(makeInput({ hellingOverridePercent: 45 }));

    expect(result.isOverride).toBe(false);
    expect(result.hellingPercent).toBe(result.hellingBerekendPercent);
    expect(result.warnings.some((w) => w.includes("ongeldig"))).toBe(true);
  });

  it("negatief hoogteverschil wordt gecorrigeerd naar 0 mm, met warning", () => {
    const result = calculateHellingbaan(makeInput({ hoogteMm: -100 }));

    expect(result.warnings.some((w) => w.includes("hoogteverschil") && w.includes("ongeldig"))).toBe(true);
    expect(result.lengteTotaalMm).toBeGreaterThanOrEqual(0);
  });
});

describe("hellingPercentToDegrees", () => {
  it("16% helling ≈ 9,09°", () => {
    expect(hellingPercentToDegrees(16)).toBeCloseTo(9.09, 1);
  });

  it("0% helling = 0°", () => {
    expect(hellingPercentToDegrees(0)).toBe(0);
  });
});
