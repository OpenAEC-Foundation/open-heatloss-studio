/**
 * Unit-tests voor de reken-evaluator (`expressionInput.ts`).
 *
 * Draaien:
 *   npx vitest run src/lib/expressionInput.test.ts
 */

import { describe, expect, it } from "vitest";

import { evaluateNumericInput } from "./expressionInput";

describe("evaluateNumericInput — kale getallen", () => {
  it("kaal geheel getal", () => {
    expect(evaluateNumericInput("12")).toBeCloseTo(12, 9);
  });

  it("kaal kommagetal", () => {
    expect(evaluateNumericInput("12,5")).toBeCloseTo(12.5, 9);
  });

  it("kaal puntgetal", () => {
    expect(evaluateNumericInput("12.5")).toBeCloseTo(12.5, 9);
  });

  it("getal met omringende whitespace", () => {
    expect(evaluateNumericInput("  3,9  ")).toBeCloseTo(3.9, 9);
  });
});

describe("evaluateNumericInput — expressies", () => {
  it("vermenigvuldiging met komma-decimalen", () => {
    expect(evaluateNumericInput("1,5*2,6")).toBeCloseTo(3.9, 9);
  });

  it("Excel-stijl leidend = wordt gestript", () => {
    expect(evaluateNumericInput("=1,5*2,6")).toBeCloseTo(3.9, 9);
  });

  it("optellen en aftrekken", () => {
    expect(evaluateNumericInput("10+2-3")).toBeCloseTo(9, 9);
  });

  it("operator-prioriteit: * vóór +", () => {
    expect(evaluateNumericInput("2+3*4")).toBeCloseTo(14, 9);
  });

  it("haakjes overrulen prioriteit", () => {
    expect(evaluateNumericInput("(2+3)*4")).toBeCloseTo(20, 9);
  });

  it("geneste haakjes", () => {
    expect(evaluateNumericInput("((1+2)*(3+4))")).toBeCloseTo(21, 9);
  });

  it("deling met decimaal resultaat", () => {
    expect(evaluateNumericInput("10/4")).toBeCloseTo(2.5, 9);
  });

  it("unaire min", () => {
    expect(evaluateNumericInput("-5")).toBeCloseTo(-5, 9);
  });

  it("unaire min in expressie", () => {
    expect(evaluateNumericInput("3*-2")).toBeCloseTo(-6, 9);
  });

  it("leidend unair plus", () => {
    expect(evaluateNumericInput("+2+3")).toBeCloseTo(5, 9);
  });

  it("dubbele unaire min", () => {
    expect(evaluateNumericInput("--5")).toBeCloseTo(5, 9);
  });

  it("whitespace binnen expressie", () => {
    expect(evaluateNumericInput(" 1,5 * 2,6 ")).toBeCloseTo(3.9, 9);
  });

  it("= met whitespace na strippen", () => {
    expect(evaluateNumericInput("= 4 * 5")).toBeCloseTo(20, 9);
  });
});

describe("evaluateNumericInput — ongeldige invoer geeft null", () => {
  it("lege string geeft null", () => {
    expect(evaluateNumericInput("")).toBeNull();
  });

  it("alleen whitespace geeft null", () => {
    expect(evaluateNumericInput("   ")).toBeNull();
  });

  it("alleen = geeft null", () => {
    expect(evaluateNumericInput("=")).toBeNull();
  });

  it("onvolledige expressie geeft null", () => {
    expect(evaluateNumericInput("2,5+")).toBeNull();
  });

  it("niet-numerieke tekst geeft null", () => {
    expect(evaluateNumericInput("abc")).toBeNull();
  });

  it("deling door nul geeft null", () => {
    expect(evaluateNumericInput("1/0")).toBeNull();
  });

  it("onbalans haakjes geeft null", () => {
    expect(evaluateNumericInput("(2+3")).toBeNull();
  });

  it("dubbel decimaalteken geeft null", () => {
    expect(evaluateNumericInput("1,2,3")).toBeNull();
  });

  it("losse operator geeft null", () => {
    expect(evaluateNumericInput("*")).toBeNull();
  });

  it("pathologisch lange invoer geeft null (DoS-guard)", () => {
    // Ruim boven de 100-teken limiet: zou anders de parser-stack kunnen laten
    // overlopen via geneste haakjes.
    const longInput = "(".repeat(2000) + "1" + ")".repeat(2000);
    expect(evaluateNumericInput(longInput)).toBeNull();
  });
});
