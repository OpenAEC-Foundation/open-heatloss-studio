import { describe, expect, it } from "vitest";

import {
  TEMPERATURE_IS_EXTERIOR,
  design_indoor_temperature,
} from "./isso53Temperature";

describe("design_indoor_temperature — ISSO 53 tabel 2.2 (TS-port)", () => {
  it("kantoor + verblijfsruimte → 20 °C", () => {
    expect(design_indoor_temperature("kantoor", "verblijfsruimte")).toBe(20);
  });

  it("kantoor + kantoorruimte → 20 °C", () => {
    expect(design_indoor_temperature("kantoor", "kantoorruimte")).toBe(20);
  });

  it("gezondheidszorg + verblijfsruimte → 22 °C (zorg)", () => {
    expect(design_indoor_temperature("gezondheidszorg", "verblijfsruimte")).toBe(
      22,
    );
  });

  it("onderwijs + badruimte → 22 °C (overig)", () => {
    expect(design_indoor_temperature("onderwijs", "badruimte")).toBe(22);
  });

  it("gezondheidszorg + badruimte → 24 °C (zorg)", () => {
    expect(design_indoor_temperature("gezondheidszorg", "badruimte")).toBe(24);
  });

  it("cel + verkeersruimte → 18 °C", () => {
    expect(design_indoor_temperature("cel", "verkeersruimte")).toBe(18);
  });

  it("kantoor + toiletruimte → 18 °C", () => {
    expect(design_indoor_temperature("kantoor", "toiletruimte")).toBe(18);
  });

  it("kantoor + technischeRuimte → 10 °C", () => {
    expect(design_indoor_temperature("kantoor", "technischeRuimte")).toBe(10);
  });

  it("kantoor + bergruimte → 10 °C", () => {
    expect(design_indoor_temperature("kantoor", "bergruimte")).toBe(10);
  });

  it("gezondheidszorg + stallingsruimte → 5 °C (forfaitair)", () => {
    expect(
      design_indoor_temperature("gezondheidszorg", "stallingsruimte"),
    ).toBe(5);
  });

  it("industrie + garage → TEMPERATURE_IS_EXTERIOR (θ_e marker)", () => {
    expect(design_indoor_temperature("industrie", "garage")).toBe(
      TEMPERATURE_IS_EXTERIOR,
    );
  });
});
