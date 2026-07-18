/**
 * Tests voor de hoeken-omrekentool (`lib/hoekenCalculation.ts`).
 *
 * Dekt: de drie rekenblad-ankers (1:12, 8%, 45°) exact op tolerantie 1e-3,
 * round-trips tussen alle drie de eenheden, en de randgevallen (0, 89,9°,
 * verhouding 1:∞-gedrag/0%, negatieve invoer, ≥90°).
 */
import { describe, expect, it } from "vitest";

import {
  gradenNaarProcent,
  gradenNaarVerhouding,
  hoekWaardenVanGraden,
  hoekWaardenVanProcent,
  hoekWaardenVanVerhouding,
  procentNaarGraden,
  procentNaarVerhouding,
  verhoudingNaarGraden,
  verhoudingNaarProcent,
} from "./hoekenCalculation";

describe("rekenblad-ankers", () => {
  it("1:12 = 8,333% ≈ 4,7636°", () => {
    const procent = verhoudingNaarProcent(12);
    expect(procent).toBeCloseTo(8.333, 3);

    const graden = verhoudingNaarGraden(12);
    expect(graden).toBeCloseTo(4.7636, 3);
  });

  it("8% ≈ 4,5739°", () => {
    expect(procentNaarGraden(8)).toBeCloseTo(4.5739, 3);
  });

  it("45° = 100% = 1:1", () => {
    expect(gradenNaarProcent(45)).toBeCloseTo(100, 3);
    expect(gradenNaarVerhouding(45)).toBeCloseTo(1, 3);
  });
});

describe("round-trips", () => {
  it("graden -> procent -> graden reconstrueert de oorspronkelijke hoek", () => {
    for (const graden of [0, 1.5, 4.7636, 15, 30, 60, 85, 89.9]) {
      const procent = gradenNaarProcent(graden);
      expect(procentNaarGraden(procent)).toBeCloseTo(graden, 6);
    }
  });

  it("procent -> verhouding -> procent reconstrueert het oorspronkelijke percentage", () => {
    for (const procent of [0.1, 1.6, 5, 8.333, 24, 100, 173.2051]) {
      const n = procentNaarVerhouding(procent);
      expect(verhoudingNaarProcent(n)).toBeCloseTo(procent, 6);
    }
  });

  it("hoekWaardenVanGraden/-Procent/-Verhouding leveren onderling consistente sets op", () => {
    const viaGraden = hoekWaardenVanGraden(30);
    const viaProcent = hoekWaardenVanProcent(viaGraden.procent);
    const viaVerhouding = hoekWaardenVanVerhouding(viaGraden.verhoudingN);

    expect(viaProcent.graden).toBeCloseTo(30, 6);
    expect(viaVerhouding.graden).toBeCloseTo(30, 6);
    expect(viaProcent.verhoudingN).toBeCloseTo(viaGraden.verhoudingN, 6);
    expect(viaVerhouding.procent).toBeCloseTo(viaGraden.procent, 6);
  });
});

describe("randgevallen", () => {
  it("0° = 0% = verhouding 1:∞ (Infinity, geen fout)", () => {
    expect(gradenNaarProcent(0)).toBe(0);
    expect(procentNaarVerhouding(0)).toBe(Infinity);
    expect(gradenNaarVerhouding(0)).toBe(Infinity);
  });

  it("89,9° geeft een geldig (zeer steil) percentage, net onder de 90°-grens", () => {
    const procent = gradenNaarProcent(89.9);
    expect(procent).toBeGreaterThan(50000);
    expect(Number.isFinite(procent)).toBe(true);
  });

  it("verhouding 1:0 (n=0, verticaal) → percentage Infinity, geen fout", () => {
    expect(verhoudingNaarProcent(0)).toBe(Infinity);
  });

  it("percentage Infinity (verticaal) → exact 90°", () => {
    expect(procentNaarGraden(Infinity)).toBe(90);
  });

  it("90° en hoger gooien een RangeError (verticaal, tan ongedefinieerd)", () => {
    expect(() => gradenNaarProcent(90)).toThrow(RangeError);
    expect(() => gradenNaarProcent(120)).toThrow(RangeError);
  });

  it("negatieve invoer gooit een RangeError voor alle drie de conversies", () => {
    expect(() => gradenNaarProcent(-1)).toThrow(RangeError);
    expect(() => procentNaarGraden(-5)).toThrow(RangeError);
    expect(() => verhoudingNaarProcent(-12)).toThrow(RangeError);
  });
});
