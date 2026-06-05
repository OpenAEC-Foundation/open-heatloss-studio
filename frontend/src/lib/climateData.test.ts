/**
 * Unit tests voor climateData.ts — de KNMI-klimaatdatalaag (WP1).
 *
 * Kernborging: de default-selectie (260 / "1991-2020") reproduceert
 * bit-gelijk `MONTHLY_CLIMATE_NL` uit `yearlyMoistureCalculation.ts`, zodat de
 * Glaser-jaarbalans géén stille resultaatwijziging krijgt (backward-compat).
 *
 * Stijl: vitest `describe`/`it` (zoals de isso53-*-tests), zodat de suite door
 * `npx vitest run` wordt opgepakt en meetelt.
 */
import { describe, expect, it } from "vitest";

import {
  CLIMATE_DEFAULT_SELECTION,
  CLIMATE_DEFAULT_STATION,
  CLIMATE_FORFAITAIR,
  decodeClimateValue,
  encodeClimateValue,
  getMonthlyClimate,
  listAvailableYears,
  listClimateOptions,
  listStations,
} from "./climateData";
import { MONTHLY_CLIMATE_NL } from "./yearlyMoistureCalculation";

describe("climateData — KNMI-klimaatdatalaag (WP1)", () => {
  it("default-station + default-selectie geven 12 maanden", () => {
    const months = getMonthlyClimate(
      CLIMATE_DEFAULT_STATION,
      CLIMATE_DEFAULT_SELECTION,
    );
    expect(months).not.toBeNull();
    expect(months).toHaveLength(12);
  });

  it("De Bilt 1991-2020 matcht bit-gelijk MONTHLY_CLIMATE_NL", () => {
    const months = getMonthlyClimate("260", "1991-2020");
    expect(months).not.toBeNull();
    expect(months).toHaveLength(MONTHLY_CLIMATE_NL.length);

    // Vergelijk maand voor maand op month/thetaE/rhE/days.
    months!.forEach((m, i) => {
      const ref = MONTHLY_CLIMATE_NL[i]!;
      expect(m.month).toBe(ref.month);
      expect(m.thetaE).toBe(ref.thetaE);
      expect(m.rhE).toBe(ref.rhE);
      expect(m.days).toBe(ref.days);
    });
  });

  it("onbekend station → null", () => {
    expect(getMonthlyClimate("999", "1991-2020")).toBeNull();
  });

  it("onbekend jaar voor bekend station → null", () => {
    expect(getMonthlyClimate("260", 1899)).toBeNull();
  });

  it("NEN5060 is een placeholder → null tot de norm-tabel is ingevuld", () => {
    expect(getMonthlyClimate("260", "NEN5060")).toBeNull();
  });

  it("listStations bevat De Bilt (260) met coördinaten", () => {
    const stations = listStations();
    const deBilt = stations.find((s) => s.id === "260");
    expect(deBilt).toBeDefined();
    expect(deBilt!.name).toBe("De Bilt");
    expect(deBilt!.lat).toBeCloseTo(52.1, 2);
    expect(deBilt!.lon).toBeCloseTo(5.18, 2);
  });

  it("listAvailableYears(260) bevat 1991-2020 vóór NEN5060", () => {
    const years = listAvailableYears("260");
    expect(years).toContain("1991-2020");
    expect(years).toContain("NEN5060");
    expect(years.indexOf("1991-2020")).toBeLessThan(years.indexOf("NEN5060"));
  });

  it("listAvailableYears(onbekend) → lege lijst", () => {
    expect(listAvailableYears("999")).toEqual([]);
  });
});

describe("climateData — forfaitaire norm-optie + single-dropdown", () => {
  it("default-selectie is 'forfaitair'", () => {
    expect(CLIMATE_DEFAULT_SELECTION).toBe(CLIMATE_FORFAITAIR);
  });

  it("getMonthlyClimate(anyStation, 'forfaitair') geeft 12 maanden = MONTHLY_CLIMATE_NL", () => {
    // Station-agnostisch: óók een willekeurig ander station levert de norm-reeks.
    for (const stationId of ["260", "240", "999", "onbekend"]) {
      const months = getMonthlyClimate(stationId, CLIMATE_FORFAITAIR);
      expect(months).not.toBeNull();
      expect(months).toHaveLength(MONTHLY_CLIMATE_NL.length);
      months!.forEach((m, i) => {
        const ref = MONTHLY_CLIMATE_NL[i]!;
        expect(m.month).toBe(ref.month);
        expect(m.thetaE).toBe(ref.thetaE);
        expect(m.rhE).toBe(ref.rhE);
        expect(m.days).toBe(ref.days);
      });
    }
  });

  it("forfaitair is bit-gelijk aan het oude De Bilt 1991-2020-resultaat", () => {
    const forfaitair = getMonthlyClimate("999", CLIMATE_FORFAITAIR);
    const deBilt = getMonthlyClimate("260", "1991-2020");
    expect(forfaitair).toEqual(deBilt);
  });

  it("listClimateOptions()[0] is de forfaitaire optie", () => {
    const options = listClimateOptions();
    expect(options[0]!.value).toBe(CLIMATE_FORFAITAIR);
    expect(options[0]!.label).toBe("Forfaitair (norm)");
    expect(options[0]!.group).toBeUndefined();
  });

  it("listClimateOptions bevat geen dubbele De Bilt 1991-2020 of NEN5060 entry", () => {
    const options = listClimateOptions();
    expect(
      options.some((o) => o.value === encodeClimateValue("260", "1991-2020")),
    ).toBe(false);
    expect(
      options.some((o) => o.value === encodeClimateValue("260", "NEN5060")),
    ).toBe(false);
  });

  it("listClimateOptions bevat historische stationjaren met group + leesbaar label", () => {
    const options = listClimateOptions();
    const deBilt2023 = options.find(
      (o) => o.value === encodeClimateValue("260", 2023),
    );
    expect(deBilt2023).toBeDefined();
    expect(deBilt2023!.group).toBe("De Bilt");
    expect(deBilt2023!.label).toBe("De Bilt — 2023");
  });

  it("encode/decode roundtrip — forfaitair", () => {
    const value = encodeClimateValue(CLIMATE_DEFAULT_STATION, CLIMATE_FORFAITAIR);
    expect(value).toBe(CLIMATE_FORFAITAIR);
    const decoded = decodeClimateValue(value);
    expect(decoded.selection).toBe(CLIMATE_FORFAITAIR);
  });

  it("encode/decode roundtrip — stationjaar", () => {
    const value = encodeClimateValue("260", 2023);
    expect(value).toBe("260|2023");
    const decoded = decodeClimateValue(value);
    expect(decoded.stationId).toBe("260");
    expect(decoded.selection).toBe(2023);
  });

  it("decodeClimateValue mapt elke listClimateOptions-value terug naar valide klimaat", () => {
    for (const opt of listClimateOptions()) {
      const { stationId, selection } = decodeClimateValue(opt.value);
      const months = getMonthlyClimate(stationId, selection);
      expect(months).not.toBeNull();
      expect(months).toHaveLength(12);
    }
  });
});
