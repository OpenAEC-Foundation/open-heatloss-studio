/**
 * Unit-tests voor de U_w-calculator (`uwCalculation.ts` + `spacerTable.ts`).
 *
 * Draaien:
 *   npx vitest run src/lib/uwCalculation.test.ts
 */

import { describe, expect, it } from "vitest";

import {
  calculateUw,
  computeGeometry,
  resolvePsiG,
  validateUwInput,
  type UwInput,
} from "./uwCalculation";
import { SPACER_PSI_G, spacerPsiG } from "./spacerTable";

// ---------- Fixtures ----------

/** Basis-kozijn van worked example 1 (1×1 ruit). */
function baseInput(): UwInput {
  return {
    width_mm: 1200,
    height_mm: 1500,
    frame_width_mm: 80,
    pane_columns: 1,
    pane_rows: 1,
    u_g: 1.1,
    u_f: 1.4,
    spacer: "aluminium",
    psi_g: 0.08,
    psi_g_is_manual: false,
  };
}

// ---------- spacerTable ----------

describe("spacerTable", () => {
  it("bevat de 4 norm-Ψ_g-waarden", () => {
    expect(SPACER_PSI_G.aluminium).toBe(0.08);
    expect(SPACER_PSI_G.stainless).toBe(0.06);
    expect(SPACER_PSI_G.warm_edge_polymer).toBe(0.04);
    expect(SPACER_PSI_G.warm_edge_foam).toBe(0.02);
  });

  it("spacerPsiG geeft undefined bij null/undefined", () => {
    expect(spacerPsiG(null)).toBeUndefined();
    expect(spacerPsiG(undefined)).toBeUndefined();
    expect(spacerPsiG("stainless")).toBe(0.06);
  });
});

// ---------- Geometrie ----------

describe("computeGeometry", () => {
  it("worked example 1 (1×1 ruit)", () => {
    const g = computeGeometry(baseInput());
    expect(g.a_w_m2).toBeCloseTo(1.8, 4);
    expect(g.a_g_m2).toBeCloseTo(1.3936, 4);
    expect(g.a_f_m2).toBeCloseTo(0.4064, 4);
    expect(g.l_g_m).toBeCloseTo(4.76, 4);
  });

  it("worked example 2 (2×1 ruiten)", () => {
    const g = computeGeometry({ ...baseInput(), pane_columns: 2 });
    expect(g.a_w_m2).toBeCloseTo(1.8, 4);
    expect(g.a_g_m2).toBeCloseTo(1.2864, 4);
    expect(g.a_f_m2).toBeCloseTo(0.5136, 4);
    expect(g.l_g_m).toBeCloseTo(7.28, 4);
  });

  it("A_g + A_f telt op tot A_w", () => {
    const g = computeGeometry({ ...baseInput(), pane_columns: 3, pane_rows: 2 });
    expect(g.a_g_m2 + g.a_f_m2).toBeCloseTo(g.a_w_m2, 4);
  });
});

// ---------- U_w-formule: worked examples ----------

describe("calculateUw — worked examples", () => {
  it("worked example 1: U_w ≈ 1,379", () => {
    const r = calculateUw(baseInput());
    expect(r.u_w).toBeCloseTo(1.379, 2);
  });

  it("worked example 2: U_w ≈ 1,509", () => {
    const r = calculateUw({ ...baseInput(), pane_columns: 2 });
    expect(r.u_w).toBeCloseTo(1.509, 2);
  });
});

// ---------- Ψ_g-resolutie ----------

describe("resolvePsiG", () => {
  it("spacer-tabel wint wanneer niet handmatig", () => {
    const input: UwInput = { ...baseInput(), spacer: "warm_edge_foam", psi_g: 0.99 };
    expect(resolvePsiG(input)).toBe(0.02);
  });

  it("handmatige override wint van de tabelwaarde", () => {
    const input: UwInput = {
      ...baseInput(),
      spacer: "aluminium",
      psi_g: 0.035,
      psi_g_is_manual: true,
    };
    expect(resolvePsiG(input)).toBe(0.035);
  });

  it("geen spacer + niet handmatig valt terug op psi_g", () => {
    const input: UwInput = {
      ...baseInput(),
      spacer: null,
      psi_g: 0.05,
      psi_g_is_manual: false,
    };
    expect(resolvePsiG(input)).toBe(0.05);
  });

  it("handmatige Ψ_g beïnvloedt het U_w-resultaat", () => {
    const base = calculateUw(baseInput());
    const manual = calculateUw({
      ...baseInput(),
      psi_g: 0.02,
      psi_g_is_manual: true,
    });
    // Lagere Ψ_g → lagere U_w.
    expect(manual.u_w).toBeLessThan(base.u_w);
  });
});

// ---------- Validatie ----------

describe("validateUwInput / calculateUw", () => {
  it("geldige invoer geeft geen fouten", () => {
    expect(validateUwInput(baseInput())).toHaveLength(0);
  });

  it("profiel te breed levert een fout op", () => {
    // (c+1)·f = 2·700 = 1400 > W=1200 → glas verdwijnt.
    const errors = validateUwInput({ ...baseInput(), frame_width_mm: 700 });
    expect(errors.some((e) => e.message.includes("Profiel te breed"))).toBe(true);
  });

  it("niet-gehele ruit-indeling levert een fout op", () => {
    const errors = validateUwInput({ ...baseInput(), pane_columns: 1.5 });
    expect(errors.some((e) => e.field === "pane_columns")).toBe(true);
  });

  it("niet-positieve afmeting levert een fout op", () => {
    const errors = validateUwInput({ ...baseInput(), width_mm: 0 });
    expect(errors.some((e) => e.field === "width_mm")).toBe(true);
  });

  it("calculateUw gooit bij ongeldige invoer", () => {
    expect(() => calculateUw({ ...baseInput(), frame_width_mm: 700 })).toThrow();
  });
});
