/**
 * Unit-tests voor de reken-evaluator (`expressionInput.ts`).
 *
 * De repo heeft (nog) geen testrunner als devDependency. Om de build schoon te
 * houden — `tsc -b` betrekt alles onder `src/` — gebruikt dit bestand een
 * zelfstandige mini-assertharness in plaats van een `vitest`/`jest`-import.
 *
 * Draaien:
 *   npx tsx src/lib/expressionInput.test.ts
 *
 * Mocht er later een echte runner landen, dan vervangt `it()` triviaal door
 * de runner-equivalent.
 */

import { evaluateNumericInput } from "./expressionInput";

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

/** Gelijkheid binnen een tolerantie (default ±1e-9 voor floating point). */
function assertClose(actual: number | null, expected: number, tol = 1e-9, label = ""): void {
  assert(actual !== null, `${label || "waarde"}: verwacht ${expected}, kreeg null`);
  const diff = Math.abs((actual as number) - expected);
  assert(
    diff <= tol,
    `${label || "waarde"}: verwacht ${expected} ±${tol}, kreeg ${actual} (Δ=${diff})`,
  );
}

function assertNull(actual: number | null, label = ""): void {
  assert(actual === null, `${label || "waarde"}: verwacht null, kreeg ${actual}`);
}

// ---------- Kale getallen ----------

it("kaal geheel getal", () => {
  assertClose(evaluateNumericInput("12"), 12, 1e-9, "12");
});

it("kaal kommagetal", () => {
  assertClose(evaluateNumericInput("12,5"), 12.5, 1e-9, "12,5");
});

it("kaal puntgetal", () => {
  assertClose(evaluateNumericInput("12.5"), 12.5, 1e-9, "12.5");
});

it("getal met omringende whitespace", () => {
  assertClose(evaluateNumericInput("  3,9  "), 3.9, 1e-9, "  3,9  ");
});

// ---------- Expressies ----------

it("vermenigvuldiging met komma-decimalen", () => {
  assertClose(evaluateNumericInput("1,5*2,6"), 3.9, 1e-9, "1,5*2,6");
});

it("Excel-stijl leidend = wordt gestript", () => {
  assertClose(evaluateNumericInput("=1,5*2,6"), 3.9, 1e-9, "=1,5*2,6");
});

it("optellen en aftrekken", () => {
  assertClose(evaluateNumericInput("10+2-3"), 9, 1e-9, "10+2-3");
});

it("operator-prioriteit: * vóór +", () => {
  assertClose(evaluateNumericInput("2+3*4"), 14, 1e-9, "2+3*4");
});

it("haakjes overrulen prioriteit", () => {
  assertClose(evaluateNumericInput("(2+3)*4"), 20, 1e-9, "(2+3)*4");
});

it("geneste haakjes", () => {
  assertClose(evaluateNumericInput("((1+2)*(3+4))"), 21, 1e-9, "((1+2)*(3+4))");
});

it("deling met decimaal resultaat", () => {
  assertClose(evaluateNumericInput("10/4"), 2.5, 1e-9, "10/4");
});

it("unaire min", () => {
  assertClose(evaluateNumericInput("-5"), -5, 1e-9, "-5");
});

it("unaire min in expressie", () => {
  assertClose(evaluateNumericInput("3*-2"), -6, 1e-9, "3*-2");
});

it("leidend unair plus", () => {
  assertClose(evaluateNumericInput("+2+3"), 5, 1e-9, "+2+3");
});

it("dubbele unaire min", () => {
  assertClose(evaluateNumericInput("--5"), 5, 1e-9, "--5");
});

it("whitespace binnen expressie", () => {
  assertClose(evaluateNumericInput(" 1,5 * 2,6 "), 3.9, 1e-9, "spaties in expressie");
});

it("= met whitespace na strippen", () => {
  assertClose(evaluateNumericInput("= 4 * 5"), 20, 1e-9, "= 4 * 5");
});

// ---------- Ongeldige invoer → null ----------

it("lege string geeft null", () => {
  assertNull(evaluateNumericInput(""), "lege string");
});

it("alleen whitespace geeft null", () => {
  assertNull(evaluateNumericInput("   "), "whitespace");
});

it("alleen = geeft null", () => {
  assertNull(evaluateNumericInput("="), "alleen =");
});

it("onvolledige expressie geeft null", () => {
  assertNull(evaluateNumericInput("2,5+"), "2,5+");
});

it("niet-numerieke tekst geeft null", () => {
  assertNull(evaluateNumericInput("abc"), "abc");
});

it("deling door nul geeft null", () => {
  assertNull(evaluateNumericInput("1/0"), "1/0");
});

it("onbalans haakjes geeft null", () => {
  assertNull(evaluateNumericInput("(2+3"), "(2+3");
});

it("dubbel decimaalteken geeft null", () => {
  assertNull(evaluateNumericInput("1,2,3"), "1,2,3");
});

it("losse operator geeft null", () => {
  assertNull(evaluateNumericInput("*"), "*");
});

it("pathologisch lange invoer geeft null (DoS-guard)", () => {
  // Ruim boven de 100-teken limiet: zou anders de parser-stack kunnen laten
  // overlopen via geneste haakjes.
  const longInput = "(".repeat(2000) + "1" + ")".repeat(2000);
  assertNull(evaluateNumericInput(longInput), "lange geneste haakjes");
});

// ---------- Rapportage ----------

const summary = `\nexpressionInput tests: ${passed} geslaagd, ${failed} gefaald.`;
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
