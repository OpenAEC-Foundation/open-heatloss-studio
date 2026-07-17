/**
 * Tests voor de HWA-rekenkern (`lib/hwaCalculation.ts`).
 *
 * Dekt: de (na bronverificatie herziene) norm-conforme referentiecase —
 * een plat dak is ALTIJD gereduceerd (grind 0,6 / overig plat 0,75), zie de
 * module-doc-comment in `hwaCalculation.ts` — het advies-alternatief-pad
 * (n+1 afvoeren), de hellingreductie-banden (trapsgewijs, GEEN interpolatie
 * meer), de platdakfactoren (grind/plat/ontbrekende afwerking + warning),
 * de gevelbijdrage (β = 0,3), meerdere afvoeren, de UV-systeemtoets
 * (pass/fail + ontbrekende capaciteit) en alle edge-cases (0 vlakken,
 * oppervlak 0, ongeldig aantal afvoeren, hellingshoek buiten bereik,
 * capaciteitstabel die niet volstaat).
 */
import { describe, expect, it } from "vitest";

import type { HwaInput, HwaRoofSurface } from "../types/hwa";
import {
  adviesDiameterMm,
  calculateHwa,
  calculateSurface,
  DEFAULT_RAIN_INTENSITY_LP_MIN_M2,
  DESIGN_SLOPE_MM_PER_M,
  DOWNPIPE_CAPACITY_TABLE,
  EMERGENCY_OVERFLOW_WARNING,
  FACADE_CONTRIBUTION_FACTOR,
  FLAT_ROOF_FACTORS,
  pitchReductionFactor,
  surfaceReductionFactor,
} from "./hwaCalculation";

/** Bouwt een minimaal geldig dakvlak, met overrides per test. */
function makeSurface(overrides: Partial<HwaRoofSurface> = {}): HwaRoofSurface {
  return {
    id: "vlak-1",
    name: "Testvlak",
    areaInputMode: "vrij",
    areaM2: 0,
    pitchDeg: 0,
    flatRoofFinish: null,
    facadeContributionM2: 0,
    downpipeCount: 1,
    ...overrides,
  };
}

describe("referentiecase (norm-conform na bronverificatie)", () => {
  it("plat dak 5×8 m zonder grind → 54 l/min (40 × 0,75 × 1,8), Ø75 volstaat", () => {
    // Norm-conform: een plat dak is ALTIJD gereduceerd, "zonder grind" ≡
    // flatRoofFinish "plat" (0,75) — het ongereduceerde (1,0) voorbeeldblok
    // uit het bronrekenblad is vervallen, zie hwaCalculation.ts
    // module-doc-comment. Zie de aparte cases hieronder voor grind (0,6) en
    // een niet-opgegeven afwerking (fallback naar 0,75 + warning).
    const surface = makeSurface({
      areaInputMode: "lxb",
      lengthM: 8,
      widthM: 5,
      pitchDeg: 0,
      flatRoofFinish: "plat",
      downpipeCount: 1,
      afschotMmPerM: 20, // boven de ontwerpdrempel — geen afschot-warning, referentiecase blijft warning-vrij
    });
    const result = calculateSurface(surface, DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value);

    expect(result.effectiveAreaM2).toBeCloseTo(30, 6); // 40 × 0,75
    expect(result.flowLpMin).toBeCloseTo(54, 6);
    expect(result.flowPerPipeLpMin).toBeCloseTo(54, 6);
    expect(result.adviesdiameterMm).toBe(75);
    // 2 afvoeren zou ook op Ø75 uitkomen (27 ≤ 75) → geen kleiner alternatief.
    expect(result.alternatief).toBeNull();
    expect(result.warnings).toEqual([]);
  });

  it("plat dak met grind → 40 × 0,6 × 1,8 = 43,2 l/min", () => {
    const surface = makeSurface({
      areaInputMode: "lxb",
      lengthM: 8,
      widthM: 5,
      pitchDeg: 0,
      flatRoofFinish: "grind",
      downpipeCount: 1,
      afschotMmPerM: 20, // boven de ontwerpdrempel — geen afschot-warning, referentiecase blijft warning-vrij
    });
    const result = calculateSurface(surface, DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value);

    expect(result.effectiveAreaM2).toBeCloseTo(24, 6); // 40 × 0,6
    expect(result.flowLpMin).toBeCloseTo(43.2, 6);
    expect(result.warnings).toEqual([]);
  });

  it("plat dak zonder opgegeven afwerking (null) → valt terug op 0,75 mét warning", () => {
    const surface = makeSurface({
      areaInputMode: "lxb",
      lengthM: 8,
      widthM: 5,
      pitchDeg: 0,
      flatRoofFinish: null,
      downpipeCount: 1,
    });
    const result = calculateSurface(surface, DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value);

    expect(result.effectiveAreaM2).toBeCloseTo(30, 6); // 40 × 0,75, zelfde als expliciet "plat"
    expect(result.flowLpMin).toBeCloseTo(54, 6);
    expect(result.warnings).toContain(
      "platdak-afwerking niet opgegeven, 0,75 (overig plat dak) aangenomen",
    );
  });

  it("100 l/min met 1 afvoer → advies Ø80; alternatief-pad met 2 afvoeren geeft Ø75", () => {
    // Hellend dak (30°, binnen de ≤45°-band → β = 1,0, geen reductie) zodat
    // dit specifiek het advies-alternatief-pad demonstreert, los van de
    // platdakfactor: 50 m² × 2,0 l/(min·m²) = 100 l/min; 1 afvoer →
    // 100 l/min per afvoer, Ø75 (capaciteit 75) volstaat niet, Ø80
    // (capaciteit 117) wel.
    const surface = makeSurface({
      areaM2: 50,
      pitchDeg: 30,
      flatRoofFinish: null,
      downpipeCount: 1,
    });
    const result = calculateSurface(surface, 2.0);

    expect(result.flowLpMin).toBeCloseTo(100, 6);
    expect(result.adviesdiameterMm).toBe(80);
    // Alternatief: 2 afvoeren → 50 l/min per afvoer → Ø75 volstaat (75 ≥ 50).
    expect(result.alternatief).not.toBeNull();
    expect(result.alternatief).toEqual({
      downpipeCount: 2,
      diameterMm: 75,
      flowPerPipeLpMin: 50,
    });
  });
});

describe("adviesDiameterMm — capaciteitstabel", () => {
  it("kiest de kleinste diameter met capaciteit ≥ debiet", () => {
    expect(adviesDiameterMm(0)).toBe(75);
    expect(adviesDiameterMm(75)).toBe(75);
    expect(adviesDiameterMm(75.1)).toBe(80);
    expect(adviesDiameterMm(117)).toBe(80);
    expect(adviesDiameterMm(163)).toBe(90);
    expect(adviesDiameterMm(210)).toBe(100);
    expect(adviesDiameterMm(338)).toBe(120);
    expect(adviesDiameterMm(870)).toBe(200);
    expect(adviesDiameterMm(2150)).toBe(315);
    expect(adviesDiameterMm(3420)).toBe(400);
  });

  it("geen 125/160 mm in de tabel (bewust, zoals het bronrekenblad)", () => {
    const diameters = DOWNPIPE_CAPACITY_TABLE.value.map((r) => r.diameterMm);
    expect(diameters).not.toContain(125);
    expect(diameters).not.toContain(160);
  });

  it("boven Ø400-capaciteit → null (volstaat niet)", () => {
    expect(adviesDiameterMm(3420.1)).toBeNull();
    expect(adviesDiameterMm(9000)).toBeNull();
  });
});

describe("pitchReductionFactor — trapsgewijze banden (GEEN interpolatie)", () => {
  it("exacte bandgrenzen", () => {
    expect(pitchReductionFactor(45)).toBe(1.0);
    expect(pitchReductionFactor(60)).toBe(0.8);
    expect(pitchReductionFactor(85)).toBe(0.6);
    expect(pitchReductionFactor(90)).toBe(0.3);
  });

  it("≤ 45° → vlakke factor 1,0 (geen reductie)", () => {
    expect(pitchReductionFactor(0)).toBe(1.0);
    expect(pitchReductionFactor(30)).toBe(1.0);
  });

  it("52,5° (in de >45–60°-band) → 0,8", () => {
    expect(pitchReductionFactor(52.5)).toBe(0.8);
  });

  it("61° (in de >60–85°-band) → 0,6", () => {
    expect(pitchReductionFactor(61)).toBe(0.6);
  });

  it("85° (bovengrens van de >60–85°-band) → 0,6", () => {
    expect(pitchReductionFactor(85)).toBe(0.6);
  });

  it("86° (in de >85–90°-band) → 0,3", () => {
    expect(pitchReductionFactor(86)).toBe(0.3);
  });

  it("geen interpolatie: begin en einde van een band geven exact dezelfde factor", () => {
    // Bv. 46° en 59° zitten allebei in de >45–60°-band → beide 0,8, geen
    // lineair verloop ertussen (in tegenstelling tot de oude implementatie).
    expect(pitchReductionFactor(46)).toBe(0.8);
    expect(pitchReductionFactor(59)).toBe(0.8);
    expect(pitchReductionFactor(46)).toBe(pitchReductionFactor(59));
  });
});

describe("surfaceReductionFactor — platdak vs. helling", () => {
  it("grind (0,6) en plat zonder grind (0,75) bij pitchDeg 0", () => {
    expect(FLAT_ROOF_FACTORS.value.grind).toBe(0.6);
    expect(FLAT_ROOF_FACTORS.value.plat).toBe(0.75);
    expect(surfaceReductionFactor(0, "grind")).toBe(0.6);
    expect(surfaceReductionFactor(0, "plat")).toBe(0.75);
  });

  it("null (geen afwerking opgegeven) valt terug op 0,75 — een plat dak is ALTIJD gereduceerd, NIET meer 1,0", () => {
    expect(surfaceReductionFactor(0, null)).toBe(0.75);
    expect(surfaceReductionFactor(0, null)).toBe(FLAT_ROOF_FACTORS.value.plat);
  });

  it("hellend dak (pitchDeg > 0) gebruikt de hellingreductie, niet de platdakfactor", () => {
    expect(surfaceReductionFactor(60, "grind")).toBe(pitchReductionFactor(60));
    expect(surfaceReductionFactor(60, null)).toBe(pitchReductionFactor(60));
  });
});

describe("calculateSurface — platdak-afwerking-warning", () => {
  it("null-afwerking op een plat dak geeft de expliciete warning, expliciete afwerking niet", () => {
    const zonderAfwerking = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: null }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(zonderAfwerking.warnings).toContain(
      "platdak-afwerking niet opgegeven, 0,75 (overig plat dak) aangenomen",
    );

    const metGrind = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "grind" }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(
      metGrind.warnings.some((w) => w.includes("platdak-afwerking niet opgegeven")),
    ).toBe(false);

    const metPlat = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat" }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(metPlat.warnings.some((w) => w.includes("platdak-afwerking niet opgegeven"))).toBe(
      false,
    );
  });

  it("hellend dak (pitchDeg > 0) geeft nooit de platdak-afwerking-warning, ongeacht flatRoofFinish", () => {
    const result = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 60, flatRoofFinish: null }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.warnings.some((w) => w.includes("platdak-afwerking"))).toBe(false);
  });
});

describe("meerdere afvoeren", () => {
  it("debiet per afvoer schaalt met downpipeCount", () => {
    const oneDownpipe = calculateSurface(
      makeSurface({ areaM2: 100, downpipeCount: 1 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    const twoDownpipes = calculateSurface(
      makeSurface({ areaM2: 100, downpipeCount: 2 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(twoDownpipes.flowLpMin).toBeCloseTo(oneDownpipe.flowLpMin, 6);
    expect(twoDownpipes.flowPerPipeLpMin).toBeCloseTo(oneDownpipe.flowPerPipeLpMin / 2, 6);
  });
});

describe("gevelbijdrage", () => {
  it("facadeContributionM2 telt op bij factor 0,3 (muren ≡ verticaal vlak, zelfde β als de 85–90°-band)", () => {
    expect(FACADE_CONTRIBUTION_FACTOR.value).toBe(0.3);
    const result = calculateSurface(
      makeSurface({ areaM2: 0, facadeContributionM2: 10, pitchDeg: 0, flatRoofFinish: "plat" }),
      2.0,
    );
    // Basisoppervlak 0 → alleen de gevelbijdrage telt: 10 × 0,3 = 3.
    expect(result.effectiveAreaM2).toBeCloseTo(3, 6);
    expect(result.flowLpMin).toBeCloseTo(6, 6);
  });
});

describe("UV-systeem — pass/fail", () => {
  const surfaces: HwaRoofSurface[] = [
    makeSurface({ id: "v1", areaM2: 40, flatRoofFinish: "plat", pitchDeg: 0 }), // 40*0,75*1,8 = 54
    makeSurface({ id: "v2", areaM2: 50, flatRoofFinish: "grind", pitchDeg: 0 }), // 50*0,6*1,8 = 54
  ];

  it("totaal 108 l/min, capaciteit 120 → pass", () => {
    const input: HwaInput = {
      surfaces,
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "uv",
      uvSystemCapacityLpMin: 120,
    };
    const result = calculateHwa(input);
    expect(result.totaalFlowLpMin).toBeCloseTo(108, 6);
    expect(result.uvToets).toEqual({
      pass: true,
      totaalFlowLpMin: result.totaalFlowLpMin,
      capaciteitLpMin: 120,
    });
    expect(result.warnings).toContain(EMERGENCY_OVERFLOW_WARNING);
  });

  it("totaal 108 l/min, capaciteit 100 → fail", () => {
    const input: HwaInput = {
      surfaces,
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "uv",
      uvSystemCapacityLpMin: 100,
    };
    const result = calculateHwa(input);
    expect(result.uvToets?.pass).toBe(false);
  });

  it("ontbrekende UV-capaciteit → geen toets, wel warning", () => {
    const input: HwaInput = {
      surfaces,
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "uv",
    };
    const result = calculateHwa(input);
    expect(result.uvToets).toBeNull();
    expect(result.warnings).toContain("UV-systeemcapaciteit ontbreekt, toets niet uitgevoerd");
    expect(result.warnings).toContain(EMERGENCY_OVERFLOW_WARNING);
  });
});

describe("noodafvoer-waarschuwing bij platte daken (traditioneel)", () => {
  it("platte daken in traditionele modus → altijd de noodafvoer-warning", () => {
    const input: HwaInput = {
      surfaces: [makeSurface({ pitchDeg: 0, areaM2: 20 })],
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "traditioneel",
    };
    const result = calculateHwa(input);
    expect(result.warnings).toContain(EMERGENCY_OVERFLOW_WARNING);
  });

  it("alleen hellende daken in traditionele modus → geen noodafvoer-warning", () => {
    const input: HwaInput = {
      surfaces: [makeSurface({ pitchDeg: 60, areaM2: 20, flatRoofFinish: null })],
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "traditioneel",
    };
    const result = calculateHwa(input);
    expect(result.warnings).not.toContain(EMERGENCY_OVERFLOW_WARNING);
  });

  it("negatieve hellingshoek (clampt naar 0° = plat dak) → toch de noodafvoer-warning", () => {
    // pitchDeg: -5 wordt in calculateSurface naar 0° geclampt en als plat
    // dak doorgerekend; calculateHwa moet dezelfde geclampte waarde gebruiken
    // om hasFlatRoof te bepalen, niet de ruwe (ongeclampte) invoer.
    const input: HwaInput = {
      surfaces: [makeSurface({ pitchDeg: -5, areaM2: 20, flatRoofFinish: "plat" })],
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "traditioneel",
    };
    const result = calculateHwa(input);
    expect(result.warnings).toContain(EMERGENCY_OVERFLOW_WARNING);
  });
});

describe("afschot — controle bij platte daken (beïnvloedt de berekening NIET)", () => {
  it("plat dak zonder afschot (ontbrekend) → plasvorming-warning, effectiveAreaM2 ongewijzigd", () => {
    const zonderAfschot = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat" }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(zonderAfschot.warnings).toContain(
      "afschot niet ingevuld of 0 bij plat dak, risico op plasvorming/waterophoping — afschot naar de afvoerpunten aanbevolen, controleer noodafvoer",
    );
    expect(zonderAfschot.afschotMmPerM).toBeNull();

    const metAfschotNul = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat", afschotMmPerM: 0 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(metAfschotNul.warnings).toContain(
      "afschot niet ingevuld of 0 bij plat dak, risico op plasvorming/waterophoping — afschot naar de afvoerpunten aanbevolen, controleer noodafvoer",
    );
    // Afschot beïnvloedt de reductiefactor niet — zelfde effectief oppervlak als zonder afschot.
    expect(metAfschotNul.effectiveAreaM2).toBeCloseTo(zonderAfschot.effectiveAreaM2, 6);
  });

  it("afschot 10 mm/m (> 0 maar < 16 mm/m drempel) → onder-drempel-warning, geen plasvorming-warning", () => {
    const result = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat", afschotMmPerM: 10 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.warnings).toContain(
      `afschot (10 mm/m) ligt onder het aanbevolen ontwerpafschot van ${DESIGN_SLOPE_MM_PER_M.value} mm/m`,
    );
    expect(
      result.warnings.some((w) => w.includes("risico op plasvorming")),
    ).toBe(false);
    expect(result.afschotMmPerM).toBe(10);
  });

  it("afschot ≥ 16 mm/m → geen afschot-warning, wel doorgegeven in het resultaat", () => {
    const opDrempel = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat", afschotMmPerM: 16 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(opDrempel.warnings.some((w) => w.includes("afschot"))).toBe(false);
    expect(opDrempel.afschotMmPerM).toBe(16);

    const bovenDrempel = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 0, flatRoofFinish: "plat", afschotMmPerM: 20 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(bovenDrempel.warnings.some((w) => w.includes("afschot"))).toBe(false);
    expect(bovenDrempel.afschotMmPerM).toBe(20);
  });

  it("hellend dak (pitchDeg > 0) geeft nooit een afschot-warning, ongeacht afschotMmPerM", () => {
    const result = calculateSurface(
      makeSurface({ areaM2: 40, pitchDeg: 60, flatRoofFinish: null, afschotMmPerM: undefined }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.warnings.some((w) => w.includes("afschot"))).toBe(false);
  });
});

describe("edge-cases", () => {
  it("0 vlakken → totalen 0 en een warning, geen crash", () => {
    const input: HwaInput = {
      surfaces: [],
      rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      systemMode: "traditioneel",
    };
    const result = calculateHwa(input);
    expect(result.surfaceResults).toEqual([]);
    expect(result.totaalEffectiveAreaM2).toBe(0);
    expect(result.totaalFlowLpMin).toBe(0);
    expect(result.uvToets).toBeNull();
    expect(result.warnings).toContain("geen dakvlakken ingevoerd");
  });

  it("oppervlak 0 → geen crash, warning, adviesdiameter voor debiet 0", () => {
    const result = calculateSurface(
      makeSurface({ areaM2: 0, facadeContributionM2: 0 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.effectiveAreaM2).toBe(0);
    expect(result.flowLpMin).toBe(0);
    expect(result.warnings).toContain("dakvlak heeft een effectief oppervlak van 0 m²");
    expect(result.adviesdiameterMm).toBe(75);
  });

  it("downpipeCount 0 of negatief → clamp naar 1 + warning", () => {
    for (const count of [0, -3, Number.NaN]) {
      const result = calculateSurface(
        makeSurface({ areaM2: 40, downpipeCount: count }),
        DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
      );
      expect(result.flowPerPipeLpMin).toBeCloseTo(result.flowLpMin, 6);
      expect(result.warnings.some((w) => w.includes("aantal afvoeren"))).toBe(true);
    }
  });

  it("downpipeCount met decimalen → naar beneden afgerond", () => {
    const result = calculateSurface(
      makeSurface({ areaM2: 40, downpipeCount: 2.7 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.flowPerPipeLpMin).toBeCloseTo(result.flowLpMin / 2, 6);
  });

  it("pitchDeg buiten 0–90 → clamp + warning", () => {
    const teLaag = calculateSurface(
      makeSurface({ pitchDeg: -10, areaM2: 40, flatRoofFinish: "plat" }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(teLaag.warnings.some((w) => w.includes("hellingshoek"))).toBe(true);
    // Geclampt naar 0° → platdakfactor (plat = 0,75) van toepassing.
    expect(teLaag.effectiveAreaM2).toBeCloseTo(40 * 0.75, 6);

    const teHoog = calculateSurface(
      makeSurface({ pitchDeg: 120, areaM2: 40, flatRoofFinish: null }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(teHoog.warnings.some((w) => w.includes("hellingshoek"))).toBe(true);
    // Geclampt naar 90° → factor 0,3.
    expect(teHoog.effectiveAreaM2).toBeCloseTo(40 * 0.3, 6);
  });

  it("pitchDeg niet-finite (NaN) → clamp naar 0° + warning (zelfde patroon als downpipeCount)", () => {
    const result = calculateSurface(
      makeSurface({ pitchDeg: Number.NaN, areaM2: 40, flatRoofFinish: "grind" }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.warnings.some((w) => w.includes("hellingshoek"))).toBe(true);
    // Geclampt naar 0° → platdakfactor (grind = 0,6) van toepassing, geen NaN-doorsijpeling.
    expect(result.effectiveAreaM2).toBeCloseTo(40 * 0.6, 6);
  });

  it("'vrij'-modus met ontbrekende areaM2 → warning, oppervlak op 0 (analoog aan 'lxb')", () => {
    const result = calculateSurface(
      makeSurface({ areaInputMode: "vrij", areaM2: undefined }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.effectiveAreaM2).toBe(0);
    expect(
      result.warnings.some((w) => w.includes("oppervlak ontbreekt bij invoermodus 'vrij'")),
    ).toBe(true);
  });

  it("debiet boven Ø400-capaciteit → adviesdiameter null + warning, geen alternatief", () => {
    // 10.000 m² zodat zowel het primaire debiet als het 2-afvoeren-
    // alternatief boven de Ø400-capaciteit (3420 l/min) blijven, ook na de
    // platdakreductie (0,75 bij flatRoofFinish: null).
    const result = calculateSurface(
      makeSurface({ areaM2: 10000, flatRoofFinish: null, downpipeCount: 1 }),
      DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
    );
    expect(result.flowPerPipeLpMin).toBeGreaterThan(3420);
    expect(result.flowPerPipeLpMin / 2).toBeGreaterThan(3420);
    expect(result.adviesdiameterMm).toBeNull();
    expect(result.alternatief).toBeNull();
    expect(result.warnings.some((w) => w.includes("volstaat niet"))).toBe(true);
  });

  it("ongeldige regenintensiteit (≤ 0) valt terug op de default + warning", () => {
    // Hellend dak (30°, ≤45°-band → β = 1,0) zodat de platdakfactor niet
    // meetelt en de test zuiver de regenintensiteit-fallback controleert.
    const input: HwaInput = {
      surfaces: [makeSurface({ areaM2: 40, pitchDeg: 30, flatRoofFinish: null })],
      rainIntensityLpMinM2: 0,
      systemMode: "traditioneel",
    };
    const result = calculateHwa(input);
    expect(result.totaalFlowLpMin).toBeCloseTo(40 * DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value, 6);
    expect(result.warnings.some((w) => w.includes("regenintensiteit"))).toBe(true);
  });
});
