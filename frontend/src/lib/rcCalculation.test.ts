/**
 * Unit tests voor rcCalculation.ts — specifiek de lambdaOverride fallback.
 *
 * Deze tests dekken het scenario waarbij de Revit thermal import layers
 * aanlevert zonder een geldige materialId (geen database-match) maar mét
 * een lambda-waarde. Zonder de fix retourneerde calculateRc() een R-waarde
 * van 0 voor zo'n laag, waardoor u_value 0 bleef en transmissieverlies op
 * álle geïmporteerde schillen op 0 W/K viel.
 *
 * Draaien:
 *   npx vitest run src/lib/rcCalculation.test.ts
 */

import { describe, expect, it } from "vitest";

import { calculateRc, type LayerInput } from "./rcCalculation.ts";

describe("calculateRc — lambdaOverride fallback", () => {
  // -------------------------------------------------------------------------
  // Test 1 — lambdaOverride zonder database-match levert correcte R-waarde
  // -------------------------------------------------------------------------

  /**
   * Scenario: één laag van 100 mm met een exotische Revit materiaal-naam
   * (`i1_hout_bamboe`) die NIET matcht in materialsDatabase, maar de exporter
   * heeft `lambda: 0.035` meegegeven.
   *
   * Verwachting:
   *   R_laag = 0.100 m / 0.035 W/mK = 2.857... m²K/W
   *   Rc = 2.857 (alleen één laag)
   *   R_totaal = Rsi(0.13) + Rc + Rse(0.04) = 3.027...
   *   U = 1 / 3.027 ≈ 0.330 W/m²K
   */
  it("lambdaOverride zonder material-match levert correcte R-waarde", () => {
    const layers: LayerInput[] = [
      {
        materialId: "i1_hout_bamboe", // raw Revit naam, niet in database
        thickness: 100,
        lambdaOverride: 0.035,
      },
    ];

    const result = calculateRc(layers, "wall");

    const expectedRLayer = 0.1 / 0.035;
    expect(result.layers[0]!.r).toBeCloseTo(expectedRLayer, 6);
    expect(result.rc).toBeCloseTo(expectedRLayer, 6);
    expect(result.rTotal).toBeCloseTo(0.13 + expectedRLayer + 0.04, 6);
    const expectedU = 1 / (0.13 + expectedRLayer + 0.04);
    expect(result.uValue).toBeCloseTo(expectedU, 6);
    expect(result.uValue).toBeGreaterThan(0);
  });

  // -------------------------------------------------------------------------
  // Test 2 — Wand conform opdracht: layers [{foo, 100mm, lambda 0.04}] → U~0.37
  // -------------------------------------------------------------------------

  /**
   * Exact het voorbeeld uit de delegatie-opdracht:
   *   CatalogEntry layers = [{material:"foo", thickness_mm:100, lambda:0.04}]
   *   → calculateRc → R_laag = 0.100/0.04 = 2.5 m²K/W
   *   → R_totaal = 0.13 + 2.5 + 0.04 = 2.67
   *   → U ≈ 0.3745 W/m²K
   */
  it("roundtrip wand 100mm lambda 0.04 → U ≈ 0.3745", () => {
    const layers: LayerInput[] = [
      {
        materialId: "foo", // onbekend materiaal
        thickness: 100,
        lambdaOverride: 0.04,
      },
    ];

    const result = calculateRc(layers, "wall");

    expect(result.layers[0]!.r).toBeCloseTo(2.5, 6);
    expect(result.rc).toBeCloseTo(2.5, 6);
    expect(result.rTotal).toBeCloseTo(2.67, 6);
    expect(result.uValue).toBeCloseTo(1 / 2.67, 4);
  });

  // -------------------------------------------------------------------------
  // Test 3 — lambdaOverride ontbreekt → R = 0 (graceful fallback)
  // -------------------------------------------------------------------------

  /**
   * Als zowel de material-match als de lambdaOverride ontbreken, moet de laag
   * 0 bijdragen (niet NaN of crashen) en de hele berekening stabiel blijven.
   */
  it("ontbrekende material én lambda geeft R = 0 zonder NaN", () => {
    const layers: LayerInput[] = [
      {
        materialId: "does-not-exist",
        thickness: 150,
        // geen lambdaOverride
      },
    ];

    const result = calculateRc(layers, "wall");

    expect(result.layers[0]!.r).toBeCloseTo(0, 9);
    expect(result.rc).toBeCloseTo(0, 9);
    // R_totaal = 0.13 + 0 + 0.04 = 0.17 → U ≈ 5.88
    expect(result.rTotal).toBeCloseTo(0.17, 9);
    expect(Number.isFinite(result.uValue)).toBe(true);
  });

  // -------------------------------------------------------------------------
  // Test 4 — Meerlaagse opbouw zonder enkele database-match
  // -------------------------------------------------------------------------

  /**
   * Realistischer scenario met drie lagen uit een Revit export — geen enkele
   * laag matcht in de database, maar de exporter heeft alle lambdas meegegeven.
   * We verifiëren dat Rc = som van R-lagen en dat U > 0 is.
   */
  it("meerlaagse opbouw volledig via fallback: Rc = som R-lagen, U > 0", () => {
    const layers: LayerInput[] = [
      { materialId: "revit-binnenblad", thickness: 100, lambdaOverride: 1.0 },
      { materialId: "revit-isolatie", thickness: 120, lambdaOverride: 0.035 },
      { materialId: "revit-buitenblad", thickness: 100, lambdaOverride: 0.9 },
    ];

    const result = calculateRc(layers, "wall");

    const rBinnen = 0.1 / 1.0;
    const rIso = 0.12 / 0.035;
    const rBuiten = 0.1 / 0.9;
    const expectedRc = rBinnen + rIso + rBuiten;

    expect(result.rc).toBeCloseTo(expectedRc, 6);
    expect(result.rTotal).toBeCloseTo(0.13 + expectedRc + 0.04, 6);
    expect(result.uValue).toBeGreaterThan(0);
    expect(result.uValue).toBeLessThan(1.0);
  });
});
