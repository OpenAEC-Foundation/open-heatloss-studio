import { describe, expect, it } from "vitest";

import {
  bblMinimumDm3s,
  bezettingMinimumDm3s,
  requirement,
  requirementByDescription,
} from "./isso53Ventilation";

describe("isso53Ventilation — tabel 4.10 spiegel van de Rust-bron", () => {
  it("Kantoorruimte → 6.5 dm³/s·pp & 0.05 pers/m²", () => {
    const r = requirementByDescription("Kantoorruimte");
    expect(r).not.toBeNull();
    expect(r?.nieuwbouwDm3sPp).toBe(6.5);
    expect(r?.personenPerM2).toBe(0.05);
    expect(r?.bestaandDm3sPp).toBe(3.44);
  });

  it("Lesruimte → 8.5 dm³/s·pp & 0.125 pers/m²", () => {
    const r = requirementByDescription("Lesruimte");
    expect(r?.nieuwbouwDm3sPp).toBe(8.5);
    expect(r?.personenPerM2).toBe(0.125);
    expect(r?.bestaandDm3sPp).toBe(3.44);
  });

  it("Patiëntenkamer → 12.0 dm³/s·pp & 0.125 pers/m²", () => {
    const r = requirementByDescription("Patiëntenkamer");
    expect(r?.nieuwbouwDm3sPp).toBe(12.0);
    expect(r?.personenPerM2).toBe(0.125);
    expect(r?.bestaandDm3sPp).toBe(3.44);
  });
});

describe("requirement(functie, ruimte) — match-arms", () => {
  it("kantoor × kantoorruimte → Kantoorruimte (6.5)", () => {
    expect(requirement("kantoor", "kantoorruimte")?.nieuwbouwDm3sPp).toBe(6.5);
  });

  it("kantoor × verblijfsruimte → Kantoorruimte (6.5)", () => {
    expect(requirement("kantoor", "verblijfsruimte")?.nieuwbouwDm3sPp).toBe(6.5);
  });

  it("onderwijs × lesruimte → Lesruimte (8.5)", () => {
    expect(requirement("onderwijs", "lesruimte")?.nieuwbouwDm3sPp).toBe(8.5);
  });

  it("Vergaderruimte catch-all geldt onder elke functie (6.5 & 0.05)", () => {
    for (const f of [
      "kantoor",
      "onderwijs",
      "gezondheidszorg",
      "bijeenkomst",
      "logies",
      "sport",
      "winkel",
      "cel",
      "industrie",
    ] as const) {
      const r = requirement(f, "vergaderruimte");
      expect(r, `vergaderruimte onder ${f}`).not.toBeNull();
      expect(r?.nieuwbouwDm3sPp).toBe(6.5);
      expect(r?.personenPerM2).toBe(0.05);
    }
  });

  it("cel × verblijfsruimte → Cel voor dag- en nachtverblijf (12.0)", () => {
    expect(requirement("cel", "verblijfsruimte")?.nieuwbouwDm3sPp).toBe(12.0);
  });

  it("kantoor × technischeRuimte → null (geen eis, dekkings-gat netjes)", () => {
    expect(requirement("kantoor", "technischeRuimte")).toBeNull();
  });

  it("kantoor × bergruimte → null", () => {
    expect(requirement("kantoor", "bergruimte")).toBeNull();
  });
});

describe("bblMinimumDm3s — oppervlakte × dichtheid × tarief", () => {
  it("Kantoor 100 m² → 100 × 0.05 × 6.5 = 32.5 dm³/s", () => {
    expect(bblMinimumDm3s("kantoor", "kantoorruimte", 100)).toBe(32.5);
  });

  it("Lesruimte 50 m² → 50 × 0.125 × 8.5 = 53.125 → 53.1 (1 dec)", () => {
    expect(bblMinimumDm3s("onderwijs", "lesruimte", 50)).toBe(53.1);
  });

  it("combinatie zonder eis → null", () => {
    expect(bblMinimumDm3s("kantoor", "technischeRuimte", 100)).toBeNull();
  });

  it("eis zonder bezettingsdichtheid (Sportzaal, personenPerM2 null) → null", () => {
    expect(bblMinimumDm3s("sport", "sportzaal", 200)).toBeNull();
  });
});

describe("bezettingMinimumDm3s — personen × tarief", () => {
  it("Kantoor 4 pers → 4 × 6.5 = 26.0 dm³/s", () => {
    expect(bezettingMinimumDm3s("kantoor", "kantoorruimte", 4)).toBe(26.0);
  });

  it("Lesruimte 25 pers → 25 × 8.5 = 212.5 dm³/s", () => {
    expect(bezettingMinimumDm3s("onderwijs", "lesruimte", 25)).toBe(212.5);
  });

  it("personen niet ingevuld → null", () => {
    expect(bezettingMinimumDm3s("kantoor", "kantoorruimte", null)).toBeNull();
    expect(bezettingMinimumDm3s("kantoor", "kantoorruimte", undefined)).toBeNull();
  });

  it("combinatie zonder eis → null", () => {
    expect(bezettingMinimumDm3s("kantoor", "technischeRuimte", 4)).toBeNull();
  });
});
