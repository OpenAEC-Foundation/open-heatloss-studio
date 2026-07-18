/**
 * Tests voor de uitzetting-rekenkern (`lib/uitzettingCalculation.ts`).
 *
 * Dekt: de drie thermische rekenblad-ankers (staal, zink, beton) exact, het
 * vochtzwelling-anker (OSB O2, EN 318), randgevallen (ΔT=0, α=null,
 * negatieve/inverse RV-invoer) en een paar checks op de bibliotheek-data
 * zelf (`getMaterialById(...).alpha`).
 */
import { describe, expect, it } from "vitest";

import { getMaterialById, MATERIALS_DATABASE } from "./materialsDatabase";
import {
  DEFAULT_SWELLING_MM_PER_M_PER_PERCENT,
  calculateMoistureSwelling,
  calculateThermalExpansion,
  shouldShowWoodGrainNote,
} from "./uitzettingCalculation";

describe("thermische uitzetting — rekenblad-ankers", () => {
  it("staal (α=12), l0=1m, ref 20/min -10/max 60 -> 0,36 mm krimp / 0,48 mm vergroting", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: 1,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.krimpMm).toBeCloseTo(0.36, 6);
    expect(result.vergrotingMm).toBeCloseTo(0.48, 6);
  });

  it("zink (α=36), l0=1m, ref 20/min -10/max 60 -> 1,08 mm krimp / 1,44 mm vergroting", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 36,
      lengthM: 1,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.krimpMm).toBeCloseTo(1.08, 6);
    expect(result.vergrotingMm).toBeCloseTo(1.44, 6);
  });

  it("beton (α=12), l0=1m, ref 20/min 17/max 27 -> 0,036 mm krimp / 0,084 mm vergroting", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: 1,
      refTempC: 20,
      minTempC: 17,
      maxTempC: 27,
    });
    expect(result.krimpMm).toBeCloseTo(0.036, 6);
    expect(result.vergrotingMm).toBeCloseTo(0.084, 6);
  });

  it("mm/m is onafhankelijk van l0 (staal-anker op 2,5 m)", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: 2.5,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.krimpMmPerM).toBeCloseTo(0.36, 6);
    expect(result.vergrotingMmPerM).toBeCloseTo(0.48, 6);
    expect(result.krimpMm).toBeCloseTo(0.36 * 2.5, 6);
    expect(result.vergrotingMm).toBeCloseTo(0.48 * 2.5, 6);
  });
});

describe("thermische uitzetting — waarschuwingen", () => {
  it("waarschuwt bij zink/aluminium-achtige uitzetting ≥ 0,5 mm op de ingevoerde lengte", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 36,
      lengthM: 1,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.vergrotingMm).toBeGreaterThanOrEqual(0.5);
    expect(result.warnings.some((w) => w.includes("dilatatie"))).toBe(true);
  });

  it("geeft GEEN dilatatie-waarschuwing bij het kleine beton-anker", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: 1,
      refTempC: 20,
      minTempC: 17,
      maxTempC: 27,
    });
    expect(result.warnings).toHaveLength(0);
  });
});

describe("thermische uitzetting — randgevallen", () => {
  it("ΔT=0 (min = max = ref) geeft 0 mm krimp en vergroting, geen fout", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: 1,
      refTempC: 20,
      minTempC: 20,
      maxTempC: 20,
    });
    expect(result.krimpMm).toBe(0);
    expect(result.vergrotingMm).toBe(0);
    expect(result.warnings).toHaveLength(0);
  });

  it("α=null (bv. isolatiemateriaal) geeft een nette waarschuwing en 0 mm, geen throw", () => {
    expect(() =>
      calculateThermalExpansion({
        alphaPer1e6PerK: null,
        lengthM: 1,
        refTempC: 20,
        minTempC: -10,
        maxTempC: 60,
      }),
    ).not.toThrow();

    const result = calculateThermalExpansion({
      alphaPer1e6PerK: null,
      lengthM: 1,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.krimpMm).toBe(0);
    expect(result.vergrotingMm).toBe(0);
    expect(result.warnings.length).toBeGreaterThan(0);
    expect(result.warnings[0]).toMatch(/geen α bekend/);
  });

  it("negatieve lengte wordt als 0 behandeld met waarschuwing", () => {
    const result = calculateThermalExpansion({
      alphaPer1e6PerK: 12,
      lengthM: -1,
      refTempC: 20,
      minTempC: -10,
      maxTempC: 60,
    });
    expect(result.krimpMm).toBe(0);
    expect(result.vergrotingMm).toBe(0);
    expect(result.warnings.some((w) => w.includes("negatief"))).toBe(true);
  });
});

describe("vochtzwelling plaatmateriaal (EN 318) — rekenblad-anker", () => {
  it("0,8 m, 50->65% RV -> 0,654 mm toename; 50->35% -> 0,654 mm krimp", () => {
    const result = calculateMoistureSwelling({
      lengthM: 0.8,
      rvInstallPercent: 50,
      rvMaxPercent: 65,
      rvMinPercent: 35,
      swellingMmPerMPerPercent: DEFAULT_SWELLING_MM_PER_M_PER_PERCENT.value,
    });
    expect(result.toenameMm).toBeCloseTo(0.654, 3);
    expect(result.krimpMm).toBeCloseTo(0.654, 3);
  });

  it("default-zwellingscoëfficiënt is de OSB-O2/EN-318-waarde 0,0545", () => {
    expect(DEFAULT_SWELLING_MM_PER_M_PER_PERCENT.value).toBe(0.0545);
    expect(DEFAULT_SWELLING_MM_PER_M_PER_PERCENT.source).toBe("EN 318");
  });
});

describe("vochtzwelling plaatmateriaal — randgevallen", () => {
  it("RV max onder RV installatie clamped naar 0 toename, met waarschuwing", () => {
    const result = calculateMoistureSwelling({
      lengthM: 1,
      rvInstallPercent: 50,
      rvMaxPercent: 40,
      rvMinPercent: 35,
      swellingMmPerMPerPercent: 0.0545,
    });
    expect(result.toenameMm).toBe(0);
    expect(result.warnings.length).toBeGreaterThan(0);
  });

  it("RV min boven RV installatie clamped naar 0 krimp, met waarschuwing", () => {
    const result = calculateMoistureSwelling({
      lengthM: 1,
      rvInstallPercent: 50,
      rvMaxPercent: 65,
      rvMinPercent: 55,
      swellingMmPerMPerPercent: 0.0545,
    });
    expect(result.krimpMm).toBe(0);
    expect(result.warnings.length).toBeGreaterThan(0);
  });
});

describe("materialenbibliotheek — alpha-veld", () => {
  it("elk metaal in de bibliotheek heeft een niet-null alpha", () => {
    const metalen = MATERIALS_DATABASE.filter((m) => m.category === "metaal");
    expect(metalen.length).toBeGreaterThan(0);
    for (const m of metalen) {
      expect(m.alpha).not.toBeNull();
    }
  });

  it("staal heeft alpha 12 en zink heeft alpha 36 (rekenblad-eigenaar)", () => {
    const staal = MATERIALS_DATABASE.find((m) => m.name === "Staal");
    const zink = MATERIALS_DATABASE.find((m) => m.name === "Zink");
    expect(staal?.alpha).toBe(12);
    expect(zink?.alpha).toBe(36);
  });

  it("dampremmende folies (Miofol/Pro Clima/PE-folie) hebben allemaal alpha null", () => {
    // Echte damprem-/dampopen-membranen: sdFixed gezet én geen dichtheid
    // (rho null) — dat sluit "Aluminium (pure folie)" uit, dat wél een
    // dichtheidloos folie-item is maar fysiek aluminium (alpha=24, zie
    // Metaal-categorie) en dus een bewuste uitzondering is.
    const folies = MATERIALS_DATABASE.filter(
      (m) =>
        m.category === "folie" &&
        m.sdFixed !== null &&
        m.rdFixed === 0 &&
        m.rho === null &&
        m.name !== "Aluminium (pure folie)",
    );
    expect(folies.length).toBeGreaterThan(0);
    for (const f of folies) {
      expect(f.alpha).toBeNull();
    }
  });

  it("getMaterialById voor staal retourneert alpha=12", () => {
    const staal = MATERIALS_DATABASE.find((m) => m.name === "Staal");
    expect(staal).toBeDefined();
    const found = getMaterialById(staal!.id);
    expect(found?.alpha).toBe(12);
  });
});

describe("shouldShowWoodGrainNote", () => {
  it("hout en plaatmateriaal -> true, staal (metaal) -> false, geen categorie -> false", () => {
    expect(shouldShowWoodGrainNote("hout")).toBe(true);
    expect(shouldShowWoodGrainNote("plaatmateriaal")).toBe(true);
    expect(shouldShowWoodGrainNote("metaal")).toBe(false);
    expect(shouldShowWoodGrainNote(null)).toBe(false);
  });
});
