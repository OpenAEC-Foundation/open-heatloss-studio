import { describe, expect, it } from "vitest";

import {
  isso53BblMinimumDm3s,
  isso53BezettingMinimumDm3s,
} from "./isso53Ventilation";

describe("isso53BblMinimumDm3s — 0,9 dm³/s per m²", () => {
  it("10,27 m² → 9,243 dm³/s", () => {
    expect(isso53BblMinimumDm3s(10.27)).toBeCloseTo(9.243, 12);
  });

  it("100 m² → 90 dm³/s", () => {
    expect(isso53BblMinimumDm3s(100)).toBe(90);
  });

  it("0 m² → 0 dm³/s", () => {
    expect(isso53BblMinimumDm3s(0)).toBe(0);
  });
});

describe("isso53BezettingMinimumDm3s — personen × tabel 4.10-tarief", () => {
  it("Kantoorruimte: 10 pers × 6,5 → 65 dm³/s", () => {
    expect(isso53BezettingMinimumDm3s("kantoor", "kantoorruimte", 10)).toBe(65);
  });

  it("Kantoor verblijfsruimte mapt naar Kantoorruimte: 4 pers × 6,5 → 26", () => {
    expect(isso53BezettingMinimumDm3s("kantoor", "verblijfsruimte", 4)).toBe(26);
  });

  it("Vergaderruimte (catch-all) onder kantoor: 8 pers × 6,5 → 52", () => {
    expect(isso53BezettingMinimumDm3s("kantoor", "vergaderruimte", 8)).toBe(52);
  });

  it("Vergaderruimte (catch-all) onder industrie: 2 pers × 6,5 → 13", () => {
    expect(isso53BezettingMinimumDm3s("industrie", "vergaderruimte", 2)).toBe(13);
  });

  it("Lesruimte: 25 pers × 8,5 → 212,5 dm³/s", () => {
    expect(isso53BezettingMinimumDm3s("onderwijs", "lesruimte", 25)).toBeCloseTo(
      212.5,
      12,
    );
  });

  it("combinatie zonder tarief (kantoor × bergruimte) → null", () => {
    expect(isso53BezettingMinimumDm3s("kantoor", "bergruimte", 5)).toBeNull();
  });

  it("combinatie zonder tarief (kantoor × technischeRuimte) → null", () => {
    expect(
      isso53BezettingMinimumDm3s("kantoor", "technischeRuimte", 5),
    ).toBeNull();
  });

  it("personen null → null (ook bij geldige combi)", () => {
    expect(
      isso53BezettingMinimumDm3s("kantoor", "kantoorruimte", null),
    ).toBeNull();
  });

  it("personen undefined → null", () => {
    expect(
      isso53BezettingMinimumDm3s("kantoor", "kantoorruimte", undefined),
    ).toBeNull();
  });
});
