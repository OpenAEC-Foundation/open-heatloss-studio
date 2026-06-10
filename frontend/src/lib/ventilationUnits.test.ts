import { describe, expect, it } from "vitest";

import type {
  VentilationUnit,
  VentilationUnitAssignment,
} from "../types/ventilation";
import {
  checkUnitCapacity,
  combinedRequirementDm3s,
  findCatalogUnit,
  getCatalogUnits,
  preferredUnitType,
  resolveUnitAssignments,
  totalAssignedCapacityM3h,
} from "./ventilationUnits";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const wtwUnit: VentilationUnit = {
  id: "u-wtw",
  type: "wtw",
  fabrikant: "Testfabrikant",
  model: "WTW 300",
  capaciteitM3h: 300, // = 83,3 dm³/s
  rendement: 0.9,
  source: "custom",
};

const mvUnit: VentilationUnit = {
  id: "u-mv",
  type: "mv",
  fabrikant: "Testfabrikant",
  model: "MV 90",
  capaciteitM3h: 90, // = 25 dm³/s
  source: "custom",
};

const units = [wtwUnit, mvUnit];

// ---------------------------------------------------------------------------
// Catalogus
// ---------------------------------------------------------------------------

describe("getCatalogUnits", () => {
  it("levert WTW- en MV-units met source 'catalog' en capaciteit > 0", () => {
    const catalog = getCatalogUnits();
    expect(catalog.length).toBeGreaterThan(0);
    expect(catalog.some((u) => u.type === "wtw")).toBe(true);
    expect(catalog.some((u) => u.type === "mv")).toBe(true);
    for (const u of catalog) {
      expect(u.source).toBe("catalog");
      expect(u.capaciteitM3h).toBeGreaterThan(0);
      expect(u.id).toBeTruthy();
    }
  });

  it("rendement alleen op WTW-units, als fractie 0..1", () => {
    for (const u of getCatalogUnits()) {
      if (u.type === "mv") expect(u.rendement).toBeUndefined();
      if (u.rendement !== undefined) {
        expect(u.rendement).toBeGreaterThan(0);
        expect(u.rendement).toBeLessThanOrEqual(1);
      }
    }
  });

  it("findCatalogUnit vindt op id; onbekend id → undefined", () => {
    const first = getCatalogUnits()[0]!;
    expect(findCatalogUnit(first.id)).toEqual(first);
    expect(findCatalogUnit("bestaat-niet")).toBeUndefined();
  });
});

describe("preferredUnitType", () => {
  it("D → wtw, B/C → mv, A → null (geen units van toepassing)", () => {
    expect(preferredUnitType("D")).toBe("wtw");
    expect(preferredUnitType("C")).toBe("mv");
    expect(preferredUnitType("B")).toBe("mv");
    expect(preferredUnitType("A")).toBeNull();
  });

  it("default (undefined) volgt systeem C → mv", () => {
    expect(preferredUnitType(undefined)).toBe("mv");
  });
});

// ---------------------------------------------------------------------------
// Toewijzing-resolutie + totaalcapaciteit (plugin: Σ capaciteit × aantal)
// ---------------------------------------------------------------------------

describe("resolveUnitAssignments / totalAssignedCapacityM3h", () => {
  it("Σ capaciteit × aantal over alle geldige toewijzingen", () => {
    const assignments: VentilationUnitAssignment[] = [
      { unitId: "u-wtw", aantal: 2 },
      { unitId: "u-mv", aantal: 1 },
    ];
    const resolved = resolveUnitAssignments(units, assignments);
    expect(resolved).toHaveLength(2);
    expect(totalAssignedCapacityM3h(resolved)).toBe(2 * 300 + 90);
  });

  it("negeert toewijzingen naar onbekende/verwijderde units en aantal ≤ 0", () => {
    const assignments: VentilationUnitAssignment[] = [
      { unitId: "verwijderd", aantal: 3 },
      { unitId: "u-mv", aantal: 0 },
      { unitId: "u-wtw", aantal: 1 },
    ];
    const resolved = resolveUnitAssignments(units, assignments);
    expect(resolved).toHaveLength(1);
    expect(totalAssignedCapacityM3h(resolved)).toBe(300);
  });

  it("undefined units of assignments → leeg (oude bestanden)", () => {
    expect(resolveUnitAssignments(undefined, [{ unitId: "x", aantal: 1 }])).toEqual([]);
    expect(resolveUnitAssignments(units, undefined)).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// Gecombineerde eis — systeem-afhankelijke keuze
// (plugin `_get_gecombineerde_eis` r.622-630 = altijd max; web = systeem-bewust)
// ---------------------------------------------------------------------------

describe("combinedRequirementDm3s — eis-keuze per systeem", () => {
  // toevoer-eis 20, afvoer-eis 35
  it("systeem D (balans): max(toevoer, afvoer) — identiek aan de plugin", () => {
    expect(combinedRequirementDm3s(20, 35, "D")).toBe(35);
    expect(combinedRequirementDm3s(40, 35, "D")).toBe(40);
  });

  // BIJGEWERKT (audit-R5 finding 3): de oude verwachting (C = alleen
  // afvoer-eis) codeerde de oude fout — de MV-box moet in balans óók de via
  // gevelroosters toegevoerde lucht verwerken, dus eis_C = max(toevoer, afvoer)
  // (identiek aan de plugin `_get_gecombineerde_eis`).
  it("systeem C (MV): max(toevoer, afvoer) — afvoer moet de toevoer verwerken (balans)", () => {
    expect(combinedRequirementDm3s(20, 35, "C")).toBe(35);
    expect(combinedRequirementDm3s(40, 35, "C")).toBe(40);
  });

  it("systeem B: alleen de toevoer-eis (afvoer natuurlijk)", () => {
    expect(combinedRequirementDm3s(40, 35, "B")).toBe(40);
  });

  it("systeem A: 0 (volledig natuurlijk, geen units)", () => {
    expect(combinedRequirementDm3s(40, 35, "A")).toBe(0);
  });

  it("default (undefined) volgt systeem C (max-gedrag)", () => {
    expect(combinedRequirementDm3s(40, 35, undefined)).toBe(40);
    expect(combinedRequirementDm3s(20, 35, undefined)).toBe(35);
  });
});

// ---------------------------------------------------------------------------
// Capaciteitstoets
// ---------------------------------------------------------------------------

describe("checkUnitCapacity", () => {
  it("voldoet: capaciteit ≥ eis, met marge% (systeem D)", () => {
    // 1× WTW 300 m³/h = 83,33 dm³/s; eis max(20, 35) = 35 dm³/s.
    const check = checkUnitCapacity(
      units,
      [{ unitId: "u-wtw", aantal: 1 }],
      20,
      35,
      "D",
    );
    expect(check.applicable).toBe(true);
    expect(check.assignedCount).toBe(1);
    expect(check.totalCapacityM3h).toBe(300);
    expect(check.totalCapacityDm3s).toBeCloseTo(83.33, 1);
    expect(check.requiredDm3s).toBe(35);
    expect(check.satisfied).toBe(true);
    expect(check.shortfallDm3s).toBe(0);
    expect(check.marginPct).toBeCloseTo(((300 / 3.6 - 35) / 35) * 100, 1);
  });

  it("tekort: capaciteit < eis (systeem C, MV-box te klein)", () => {
    // 1× MV 90 m³/h = 25 dm³/s; eis = afvoer 35 dm³/s → tekort 10.
    const check = checkUnitCapacity(
      units,
      [{ unitId: "u-mv", aantal: 1 }],
      20,
      35,
      "C",
    );
    expect(check.satisfied).toBe(false);
    expect(check.shortfallDm3s).toBeCloseTo(10, 5);
    expect(check.marginPct).toBeLessThan(0);
  });

  it("systeem C met toevoer > afvoer: eis volgt de toevoer (max-gedrag, audit-R5)", () => {
    // 1× MV 90 m³/h = 25 dm³/s; toevoer-eis 40 > afvoer-eis 20 → eis 40
    // (de box moet de toegevoerde lucht verwerken) → tekort 15.
    const check = checkUnitCapacity(
      units,
      [{ unitId: "u-mv", aantal: 1 }],
      40,
      20,
      "C",
    );
    expect(check.requiredDm3s).toBe(40);
    expect(check.satisfied).toBe(false);
    expect(check.shortfallDm3s).toBeCloseTo(15, 5);
  });

  it("aantal vermenigvuldigt: 2× MV 90 dekt de eis wél", () => {
    const check = checkUnitCapacity(
      units,
      [{ unitId: "u-mv", aantal: 2 }],
      20,
      35,
      "C",
    );
    expect(check.totalCapacityM3h).toBe(180);
    expect(check.satisfied).toBe(true);
  });

  it("systeem A: niet van toepassing (applicable false, eis 0)", () => {
    const check = checkUnitCapacity(units, [], 20, 35, "A");
    expect(check.applicable).toBe(false);
    expect(check.requiredDm3s).toBe(0);
  });

  it("eis 0 → satisfied true en marge 0 (geen zinvolle marge)", () => {
    const check = checkUnitCapacity(units, [], 0, 0, "D");
    expect(check.satisfied).toBe(true);
    expect(check.marginPct).toBe(0);
  });
});
