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
  getMonthlyClimate,
  listAvailableYears,
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
