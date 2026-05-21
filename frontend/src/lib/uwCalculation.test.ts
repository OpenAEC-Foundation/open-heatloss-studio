/**
 * Unit-tests voor de U_w-calculator (`uwCalculation.ts` + `spacerTable.ts`).
 *
 * De repo heeft (nog) geen testrunner als devDependency. Om de build schoon te
 * houden — `tsc -b` betrekt alles onder `src/` — gebruikt dit bestand een
 * zelfstandige mini-assertharness in plaats van een `vitest`/`jest`-import.
 *
 * Draaien:
 *   npx tsx src/lib/uwCalculation.test.ts
 *
 * Mocht er later een echte runner landen, dan vervangt `it()` triviaal door
 * de runner-equivalent.
 */

import {
  calculateUw,
  computeGeometry,
  resolvePsiG,
  validateUwInput,
  type UwInput,
} from "./uwCalculation";
import { SPACER_PSI_G, spacerPsiG } from "./spacerTable";

// ---------- Mini-assertharness ----------

let passed = 0;
let failed = 0;
const failures: string[] = [];

function it(name: string, fn: () => void): void {
  try {
    fn();
    passed += 1;
  } catch (err) {
    failed += 1;
    const message = err instanceof Error ? err.message : String(err);
    failures.push(`✗ ${name}\n    ${message}`);
  }
}

function assert(condition: boolean, message: string): void {
  if (!condition) throw new Error(message);
}

/** Gelijkheid binnen een tolerantie (default ±0,01 — werkpak-acceptatie). */
function assertClose(actual: number, expected: number, tol = 0.01, label = ""): void {
  const diff = Math.abs(actual - expected);
  assert(
    diff <= tol,
    `${label || "waarde"}: verwacht ${expected} ±${tol}, kreeg ${actual} (Δ=${diff.toFixed(5)})`,
  );
}

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

it("spacertabel bevat de 4 norm-Ψ_g-waarden", () => {
  assertClose(SPACER_PSI_G.aluminium, 0.08, 0, "aluminium");
  assertClose(SPACER_PSI_G.stainless, 0.06, 0, "stainless");
  assertClose(SPACER_PSI_G.warm_edge_polymer, 0.04, 0, "warm_edge_polymer");
  assertClose(SPACER_PSI_G.warm_edge_foam, 0.02, 0, "warm_edge_foam");
});

it("spacerPsiG geeft undefined bij null/undefined", () => {
  assert(spacerPsiG(null) === undefined, "null moet undefined geven");
  assert(spacerPsiG(undefined) === undefined, "undefined moet undefined geven");
  assertClose(spacerPsiG("stainless") ?? -1, 0.06, 0, "stainless lookup");
});

// ---------- Geometrie ----------

it("geometrie — worked example 1 (1×1 ruit)", () => {
  const g = computeGeometry(baseInput());
  assertClose(g.a_w_m2, 1.8, 0.0001, "A_w");
  assertClose(g.a_g_m2, 1.3936, 0.0001, "A_g");
  assertClose(g.a_f_m2, 0.4064, 0.0001, "A_f");
  assertClose(g.l_g_m, 4.76, 0.0001, "l_g");
});

it("geometrie — worked example 2 (2×1 ruiten)", () => {
  const g = computeGeometry({ ...baseInput(), pane_columns: 2 });
  assertClose(g.a_w_m2, 1.8, 0.0001, "A_w");
  assertClose(g.a_g_m2, 1.2864, 0.0001, "A_g");
  assertClose(g.a_f_m2, 0.5136, 0.0001, "A_f");
  assertClose(g.l_g_m, 7.28, 0.0001, "l_g");
});

it("geometrie — A_g + A_f telt op tot A_w", () => {
  const g = computeGeometry({ ...baseInput(), pane_columns: 3, pane_rows: 2 });
  assertClose(g.a_g_m2 + g.a_f_m2, g.a_w_m2, 0.0001, "A_g + A_f = A_w");
});

// ---------- U_w-formule: worked examples ----------

it("U_w — worked example 1: U_w ≈ 1,379", () => {
  const r = calculateUw(baseInput());
  assertClose(r.u_w, 1.379, 0.01, "U_w example 1");
});

it("U_w — worked example 2: U_w ≈ 1,509", () => {
  const r = calculateUw({ ...baseInput(), pane_columns: 2 });
  assertClose(r.u_w, 1.509, 0.01, "U_w example 2");
});

// ---------- Ψ_g-resolutie ----------

it("Ψ_g — spacer-tabel wint wanneer niet handmatig", () => {
  const input: UwInput = { ...baseInput(), spacer: "warm_edge_foam", psi_g: 0.99 };
  assertClose(resolvePsiG(input), 0.02, 0, "tabel-Ψ_g warm_edge_foam");
});

it("Ψ_g — handmatige override wint van de tabelwaarde", () => {
  const input: UwInput = {
    ...baseInput(),
    spacer: "aluminium",
    psi_g: 0.035,
    psi_g_is_manual: true,
  };
  assertClose(resolvePsiG(input), 0.035, 0, "override-Ψ_g");
});

it("Ψ_g — geen spacer + niet handmatig valt terug op psi_g", () => {
  const input: UwInput = {
    ...baseInput(),
    spacer: null,
    psi_g: 0.05,
    psi_g_is_manual: false,
  };
  assertClose(resolvePsiG(input), 0.05, 0, "fallback-Ψ_g");
});

it("U_w — handmatige Ψ_g beïnvloedt het resultaat", () => {
  const base = calculateUw(baseInput());
  const manual = calculateUw({
    ...baseInput(),
    psi_g: 0.02,
    psi_g_is_manual: true,
  });
  // Lagere Ψ_g → lagere U_w.
  assert(manual.u_w < base.u_w, `verwacht ${manual.u_w} < ${base.u_w}`);
});

// ---------- Validatie ----------

it("validatie — geldige invoer geeft geen fouten", () => {
  assert(validateUwInput(baseInput()).length === 0, "basis-invoer moet geldig zijn");
});

it("validatie — profiel te breed levert een fout op", () => {
  // (c+1)·f = 2·700 = 1400 > W=1200 → glas verdwijnt.
  const errors = validateUwInput({ ...baseInput(), frame_width_mm: 700 });
  assert(
    errors.some((e) => e.message.includes("Profiel te breed")),
    "verwacht een 'Profiel te breed'-fout",
  );
});

it("validatie — niet-gehele ruit-indeling levert een fout op", () => {
  const errors = validateUwInput({ ...baseInput(), pane_columns: 1.5 });
  assert(
    errors.some((e) => e.field === "pane_columns"),
    "verwacht een fout op pane_columns",
  );
});

it("validatie — niet-positieve afmeting levert een fout op", () => {
  const errors = validateUwInput({ ...baseInput(), width_mm: 0 });
  assert(
    errors.some((e) => e.field === "width_mm"),
    "verwacht een fout op width_mm",
  );
});

it("calculateUw gooit bij ongeldige invoer", () => {
  let threw = false;
  try {
    calculateUw({ ...baseInput(), frame_width_mm: 700 });
  } catch {
    threw = true;
  }
  assert(threw, "calculateUw moet gooien bij ongeldige invoer");
});

// ---------- Rapportage ----------

const summary = `\nU_w-calculator tests: ${passed} geslaagd, ${failed} gefaald.`;
if (failed > 0) {
  console.error(failures.join("\n"));
  console.error(summary);
  // Exit-code 1 zodat een CI-stap faalt. `process` is een Node-global; de
  // repo heeft geen @types/node, dus dynamisch via globalThis benaderen
  // zonder een type-afhankelijkheid te introduceren. In een browser-bundle
  // is dit afwezig → no-op.
  const g = globalThis as { process?: { exitCode?: number } };
  if (g.process) g.process.exitCode = 1;
} else {
  console.log(summary);
}

export { passed, failed };
