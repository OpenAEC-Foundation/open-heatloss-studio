/**
 * Tests voor het deurspleet-rekenmodel (`lib/doorGap.ts`).
 *
 * Dekt: consistentie met de bestaande spleetformule
 * (`estimateDoorGapAreaCm2`, NEN 1087:2001 §5.1.3.2), de vuistregel-
 * reconciliatie (12 vs. 12,9 cm² per dm³/s), de mm-afronding naar boven,
 * de advies-drempel (20 mm) + het geluidswerend-pad, het indicatieve
 * deurrooster-voorstel (seed + conservatieve netto-fracties) en de
 * eenheden-conventie: het rekenmodel is dm³/s-only — m³/h-conversie blijft
 * aan de UI-rand (`flowFromDisplay`, `types/ventilation.ts`).
 */
import { describe, expect, it } from "vitest";

import { flowFromDisplay } from "../types/ventilation";
import {
  DOOR_GAP_DELTA_P_OFFICE_PA,
  DOOR_GAP_DELTA_P_PA,
  DOOR_GAP_GRILLE_THRESHOLD_MM,
  DOOR_GRILLE_SEED,
  doorGapAdvice,
  gapHeightMm,
  GRILLE_NET_AREA_FRACTION,
  GRILLE_NET_AREA_FRACTION_ACOUSTIC,
  grilleNetAreaCm2,
  proposeDoorGrille,
  requiredGapAreaCm2,
  RULE_OF_THUMB_CM2_PER_DM3S,
  ruleOfThumbAreaCm2,
} from "./doorGap";
import { estimateDoorGapAreaCm2 } from "./ventilationBalance";

describe("requiredGapAreaCm2", () => {
  it("1 dm³/s bij Δp = 1 Pa (woonfunctie) → 12,9 cm²", () => {
    // Exact: 1/(0,6·√(2/1,2)) × 10 = 12,9099… cm² per dm³/s.
    expect(requiredGapAreaCm2(1)).toBeCloseTo(12.91, 2);
    expect(requiredGapAreaCm2(1, DOOR_GAP_DELTA_P_PA)).toBeCloseTo(12.91, 2);
  });

  it("is exact dezelfde formule als estimateDoorGapAreaCm2 (geen duplicatie)", () => {
    for (const flow of [0, 1, 7, 12.5, 21, 25]) {
      expect(requiredGapAreaCm2(flow)).toBe(estimateDoorGapAreaCm2(flow));
      expect(requiredGapAreaCm2(flow, DOOR_GAP_DELTA_P_OFFICE_PA)).toBe(
        estimateDoorGapAreaCm2(flow, DOOR_GAP_DELTA_P_OFFICE_PA),
      );
    }
  });

  it("kantoor-Δp (2 Pa) geeft een factor √2 kleinere doorlaat", () => {
    expect(requiredGapAreaCm2(1, DOOR_GAP_DELTA_P_OFFICE_PA)).toBeCloseTo(
      12.9099 / Math.SQRT2,
      3,
    );
  });

  it("debiet ≤ 0 → 0 cm²", () => {
    expect(requiredGapAreaCm2(0)).toBe(0);
    expect(requiredGapAreaCm2(-5)).toBe(0);
  });
});

describe("vuistregel 12 cm² per dm³/s", () => {
  it("constante is 12 en de exacte formule (12,9) is iets ruimer", () => {
    expect(RULE_OF_THUMB_CM2_PER_DM3S).toBe(12);
    // De vuistregel is krapper afgerond dan de exacte orifice-uitkomst.
    expect(ruleOfThumbAreaCm2(1)).toBeLessThan(requiredGapAreaCm2(1));
    expect(ruleOfThumbAreaCm2(7)).toBe(84);
    expect(ruleOfThumbAreaCm2(0)).toBe(0);
    expect(ruleOfThumbAreaCm2(-1)).toBe(0);
  });
});

describe("gapHeightMm", () => {
  it("25 dm³/s door een 880 mm-deur → 322,7 cm² en 37 mm (naar boven afgerond)", () => {
    const r = gapHeightMm({ flowDm3s: 25, doorWidthMm: 880 });
    expect(r.areaCm2).toBeCloseTo(322.75, 1);
    // Onafgerond: 32274,9 mm² / 880 mm = 36,68 mm → ceil = 37.
    expect(r.heightMm).toBe(37);
  });

  it("rondt altijd naar boven af op hele mm (nooit te krap)", () => {
    for (const flow of [1, 7, 12.5, 21]) {
      const r = gapHeightMm({ flowDm3s: flow, doorWidthMm: 880 });
      const rawMm = (r.areaCm2 * 100) / 880;
      expect(Number.isInteger(r.heightMm)).toBe(true);
      expect(r.heightMm).toBeGreaterThanOrEqual(rawMm);
      expect(r.heightMm - rawMm).toBeLessThan(1);
    }
  });

  it("respecteert het Δp-criterium (2 Pa kantoor → lagere spleet)", () => {
    const woon = gapHeightMm({ flowDm3s: 10, doorWidthMm: 880 });
    const kantoor = gapHeightMm({
      flowDm3s: 10,
      doorWidthMm: 880,
      deltaPPa: DOOR_GAP_DELTA_P_OFFICE_PA,
    });
    expect(kantoor.areaCm2).toBeLessThan(woon.areaCm2);
    expect(kantoor.heightMm).toBeLessThanOrEqual(woon.heightMm);
  });

  it("freeAreaReductionPct vergroot de geometrische doorlaat (50% → ×2)", () => {
    const basis = gapHeightMm({ flowDm3s: 7, doorWidthMm: 880 });
    const gereduceerd = gapHeightMm({
      flowDm3s: 7,
      doorWidthMm: 880,
      freeAreaReductionPct: 50,
    });
    expect(gereduceerd.areaCm2).toBeCloseTo(basis.areaCm2 * 2, 6);
    expect(gereduceerd.heightMm).toBeGreaterThan(basis.heightMm);
  });

  it("negeert ongeldige reducties (≤ 0 of ≥ 100)", () => {
    const basis = gapHeightMm({ flowDm3s: 7, doorWidthMm: 880 });
    for (const pct of [0, -10, 100, 250]) {
      expect(
        gapHeightMm({ flowDm3s: 7, doorWidthMm: 880, freeAreaReductionPct: pct }),
      ).toEqual(basis);
    }
  });

  it("debiet ≤ 0 of ongeldige deurbreedte → hoogte 0", () => {
    expect(gapHeightMm({ flowDm3s: 0, doorWidthMm: 880 })).toEqual({
      areaCm2: 0,
      heightMm: 0,
    });
    expect(gapHeightMm({ flowDm3s: 7, doorWidthMm: 0 }).heightMm).toBe(0);
    expect(gapHeightMm({ flowDm3s: 7, doorWidthMm: -880 }).heightMm).toBe(0);
  });
});

describe("doorGapAdvice", () => {
  it("≤ 20 mm → ok, > 20 mm → deurrooster", () => {
    expect(DOOR_GAP_GRILLE_THRESHOLD_MM).toBe(20);
    expect(doorGapAdvice(0)).toBe("ok");
    expect(doorGapAdvice(11)).toBe("ok");
    expect(doorGapAdvice(20)).toBe("ok");
    expect(doorGapAdvice(21)).toBe("grille");
    expect(doorGapAdvice(37)).toBe("grille");
  });

  it("geluidswerend → altijd rooster, ongeacht spleethoogte", () => {
    // Een open spleet is akoestisch ongewenst — ook bij een lage spleet.
    expect(doorGapAdvice(0, true)).toBe("grille");
    expect(doorGapAdvice(11, true)).toBe("grille");
    expect(doorGapAdvice(37, true)).toBe("grille");
    // Expliciet false ≡ default-gedrag.
    expect(doorGapAdvice(11, false)).toBe("ok");
  });
});

describe("grilleNetAreaCm2 — indicatieve netto doorlaat", () => {
  it("rekent met de conservatieve fracties (standaard 40%, geluidswerend 25%)", () => {
    expect(GRILLE_NET_AREA_FRACTION).toBe(0.4);
    expect(GRILLE_NET_AREA_FRACTION_ACOUSTIC).toBe(0.25);
    // 455×90 mm = 409,5 cm² dagmaat → 163,8 cm² netto (standaard).
    expect(grilleNetAreaCm2({ widthMm: 455, heightMm: 90 })).toBeCloseTo(163.8, 6);
    // Geluidswerend: 409,5 × 0,25 = 102,375 cm².
    expect(grilleNetAreaCm2({ widthMm: 455, heightMm: 90 }, true)).toBeCloseTo(
      102.375,
      6,
    );
  });

  it("seed is oplopend gesorteerd op dagmaat-oppervlak (kleinste-eerst-selectie)", () => {
    const areas = DOOR_GRILLE_SEED.map((s) => s.widthMm * s.heightMm);
    expect(areas).toEqual([...areas].sort((a, b) => a - b));
    expect(DOOR_GRILLE_SEED.length).toBeGreaterThanOrEqual(4);
  });
});

describe("proposeDoorGrille — kleinste passende rooster", () => {
  it("7 dm³/s (90,4 cm²) → 1× 345×90 (netto 124,2 cm²)", () => {
    const p = proposeDoorGrille(requiredGapAreaCm2(7));
    expect(p).not.toBeNull();
    expect(p!.size).toEqual({ widthMm: 345, heightMm: 90 });
    expect(p!.count).toBe(1);
    expect(p!.netAreaCm2PerGrille).toBeCloseTo(124.2, 6);
    expect(p!.totalNetAreaCm2).toBeGreaterThanOrEqual(requiredGapAreaCm2(7));
    expect(p!.acoustic).toBe(false);
  });

  it("geluidswerend vergt een groter rooster voor hetzelfde debiet", () => {
    // Zelfde 90,4 cm², maar met 25%-fractie → 425×90 (netto 95,6 cm²).
    const p = proposeDoorGrille(requiredGapAreaCm2(7), true);
    expect(p!.size).toEqual({ widthMm: 425, heightMm: 90 });
    expect(p!.count).toBe(1);
    expect(p!.acoustic).toBe(true);
    expect(p!.totalNetAreaCm2).toBeGreaterThanOrEqual(requiredGapAreaCm2(7));
  });

  it("past het niet in 1 rooster → kleinste maat die het in 2× haalt", () => {
    // 25 dm³/s → 322,7 cm²; grootste enkele maat (455×150) levert maar
    // 273 cm² netto → 2× 455×90 (2 × 163,8 = 327,6 cm²).
    const p = proposeDoorGrille(requiredGapAreaCm2(25));
    expect(p!.count).toBe(2);
    expect(p!.size).toEqual({ widthMm: 455, heightMm: 90 });
    expect(p!.totalNetAreaCm2).toBeGreaterThanOrEqual(requiredGapAreaCm2(25));
  });

  it("fallback: zelfs 2× de grootste te klein → grootste maat × benodigd aantal", () => {
    // 600 cm² > 2 × 273 cm² → 3× 455×150.
    const p = proposeDoorGrille(600);
    expect(p!.size).toEqual({ widthMm: 455, heightMm: 150 });
    expect(p!.count).toBe(3);
    expect(p!.totalNetAreaCm2).toBeGreaterThanOrEqual(600);
  });

  it("voorstel dekt de behoefte altijd (totaal ≥ benodigd, over een bereik)", () => {
    for (const req of [1, 50, 90.4, 163.8, 200, 322.7, 500, 1000]) {
      for (const acoustic of [false, true]) {
        const p = proposeDoorGrille(req, acoustic);
        expect(p).not.toBeNull();
        expect(p!.totalNetAreaCm2).toBeGreaterThanOrEqual(req);
        expect(p!.totalNetAreaCm2).toBeCloseTo(
          p!.netAreaCm2PerGrille * p!.count,
          9,
        );
      }
    }
  });

  it("doorlaat ≤ 0 → geen rooster nodig (null)", () => {
    expect(proposeDoorGrille(0)).toBeNull();
    expect(proposeDoorGrille(-10)).toBeNull();
    expect(proposeDoorGrille(0, true)).toBeNull();
  });
});

describe("eenheden-conventie — m³/h alleen aan de UI-rand", () => {
  it("90 m³/h via flowFromDisplay aan de rand ≡ 25 dm³/s in het rekenmodel", () => {
    // Het rekenmodel kent géén m³/h: de UI converteert vóór de aanroep.
    const dm3s = flowFromDisplay(90, "m3h");
    expect(dm3s).toBe(25);
    expect(requiredGapAreaCm2(dm3s)).toBe(requiredGapAreaCm2(25));
    expect(gapHeightMm({ flowDm3s: dm3s, doorWidthMm: 880 })).toEqual(
      gapHeightMm({ flowDm3s: 25, doorWidthMm: 880 }),
    );
  });
});
