/**
 * Jaarlijkse vochtbalans volgens NEN-EN-ISO 13788 (Glaser-maandmethode).
 *
 * Berekent per maand de condensatie- of droogsnelheid op het kritieke
 * grensvlak in de constructie. Volgt de vochtophoping over het jaar
 * en signaleert schimmelrisico bij > 90 dagen vocht.
 */

import type { VerticalPosition } from "../types";

import { saturationPressure } from "./glaserCalculation";
import { getMaterialById } from "./materialsDatabase";

// ---------- Maandelijks buitenklimaat Nederland (KNMI De Bilt 1991–2020) ----------

export interface MonthlyClimate {
  month: string;
  thetaE: number;
  rhE: number;
  days: number;
}

export const MONTHLY_CLIMATE_NL: MonthlyClimate[] = [
  { month: "Jan", thetaE: 3.4, rhE: 87, days: 31 },
  { month: "Feb", thetaE: 3.6, rhE: 84, days: 28 },
  { month: "Mrt", thetaE: 6.2, rhE: 80, days: 31 },
  { month: "Apr", thetaE: 9.3, rhE: 74, days: 30 },
  { month: "Mei", thetaE: 13.1, rhE: 73, days: 31 },
  { month: "Jun", thetaE: 15.8, rhE: 74, days: 30 },
  { month: "Jul", thetaE: 17.9, rhE: 76, days: 31 },
  { month: "Aug", thetaE: 17.6, rhE: 77, days: 31 },
  { month: "Sep", thetaE: 14.6, rhE: 81, days: 30 },
  { month: "Okt", thetaE: 10.7, rhE: 84, days: 31 },
  { month: "Nov", thetaE: 6.8, rhE: 88, days: 30 },
  { month: "Dec", thetaE: 4.0, rhE: 88, days: 31 },
];

// ---------- Constanten ----------

/** Dampdoorlatendheid van stilstaande lucht [kg/(m·s·Pa)]. */
const DELTA_0 = 2.0e-10;

const R_SI: Record<VerticalPosition, number> = {
  wall: 0.13,
  ceiling: 0.10,
  floor: 0.17,
};
const R_SE = 0.04;

// ---------- Types ----------

export type MoistureStatus = "dry" | "condensation" | "drying";

export interface MonthlyMoistureResult {
  month: string;
  thetaE: number;
  rhE: number;
  /** Dampdruk buiten [Pa]. */
  pe: number;
  /** Dampdruk binnen [Pa]. */
  pi: number;
  /** Temperatuur op condensatievlak [°C]. */
  thetaC: number;
  /** Verzadigingsdruk op condensatievlak [Pa]. */
  pSatC: number;
  /** Condensatiesnelheid [g/(m²·maand)]. Positief = condensatie, negatief = droging. */
  gc: number;
  /** Opgebouwd vocht [g/m²]. */
  ma: number;
  /** Status van het condensatievlak. */
  status: MoistureStatus;
}

export interface YearlyMoistureResult {
  months: MonthlyMoistureResult[];
  /** Maximaal opgebouwd vocht [g/m²]. */
  maxMa: number;
  /** Aantal dagen met vocht aanwezig. */
  wetDays: number;
  /** Schimmelrisico (> 90 dagen vocht). */
  hasRisk: boolean;
  /** Droogt de constructie volledig uit binnen een jaar? */
  driesOut: boolean;
  /** Naam van de laag aan de binnenzijde van het condensatievlak. */
  planeInnerLayer: string;
  /** Naam van de laag aan de buitenzijde van het condensatievlak. */
  planeOuterLayer: string;
  /** Positie van het condensatievlak [mm vanaf binnen]. */
  planePosition: number;
}

// ---------- Interne laagdata ----------

interface LayerData {
  name: string;
  thickness: number;
  r: number;
  sd: number;
}

// ---------- Berekening ----------

export function calculateYearlyMoisture(
  layers: { materialId: string; thickness: number }[],
  position: VerticalPosition,
  thetaI: number,
  rhI: number,
  climate?: MonthlyClimate[],
): YearlyMoistureResult | null {
  const rSi = R_SI[position];
  const rSe = R_SE;

  // Maandklimaat: gekozen KNMI-data (station + selectie) of de forfaitaire
  // De Bilt 1991–2020-normaal als fallback. Een ongeldige/lege array valt
  // óók terug op de norm-tabel zodat de berekening niet stilvalt.
  const monthlyClimate =
    climate && climate.length === 12 ? climate : MONTHLY_CLIMATE_NL;

  // Bouw laagdata op
  const layerData: LayerData[] = [];
  for (const li of layers) {
    const mat = getMaterialById(li.materialId);
    if (!mat) continue;

    let r: number;
    if (mat.rdFixed !== null) {
      r = mat.rdFixed;
    } else if (mat.lambda && mat.lambda > 0) {
      r = (li.thickness / 1000) / mat.lambda;
    } else {
      r = 0;
    }

    const sd = mat.sdFixed !== null
      ? mat.sdFixed
      : mat.mu * (li.thickness / 1000);

    layerData.push({ name: mat.name, thickness: li.thickness, r, sd });
  }

  if (layerData.length === 0) return null;

  const rTotal = rSi + layerData.reduce((s, l) => s + l.r, 0) + rSe;
  const sdTotal = layerData.reduce((s, l) => s + l.sd, 0);

  if (sdTotal <= 0 || rTotal <= 0) return null;

  // Dampdruk binnen (constant over het jaar)
  const pi = (rhI / 100) * saturationPressure(thetaI);

  // Grensvlakken opbouwen: na Rsi, na elke laag
  // Interface i zit tussen laag i-1 en laag i (i=0 = binnenoppervlak)
  const interfaces: { rCum: number; sdCum: number; xCum: number }[] = [];
  let rCum = rSi;
  let sdCum = 0;
  let xCum = 0;

  interfaces.push({ rCum, sdCum, xCum });

  for (const layer of layerData) {
    rCum += layer.r;
    sdCum += layer.sd;
    xCum += layer.thickness;
    interfaces.push({ rCum, sdCum, xCum });
  }

  // Vind het kritieke condensatievlak (maximale condensatie in koudste maand)
  const coldest = monthlyClimate.reduce((prev, curr) =>
    curr.thetaE < prev.thetaE ? curr : prev,
  );
  const peCold = (coldest.rhE / 100) * saturationPressure(coldest.thetaE);

  let maxGc = -Infinity;
  let critIdx = 0;

  for (let i = 0; i < interfaces.length; i++) {
    const iface = interfaces[i]!;
    const theta = thetaI - (thetaI - coldest.thetaE) * iface.rCum / rTotal;
    const pSat = saturationPressure(theta);

    const sdIn = Math.max(iface.sdCum, 0.001);
    const sdOut = Math.max(sdTotal - iface.sdCum, 0.001);

    const gIn = DELTA_0 * (pi - pSat) / sdIn;
    const gOut = DELTA_0 * (pSat - peCold) / sdOut;
    const gc = (gIn - gOut) * coldest.days * 86400 * 1000;

    if (gc > maxGc) {
      maxGc = gc;
      critIdx = i;
    }
  }

  const crit = interfaces[critIdx]!;

  // Laagnamen rond het condensatievlak
  const planeInnerLayer = critIdx > 0
    ? layerData[critIdx - 1]?.name ?? "Rsi"
    : "Rsi";
  const planeOuterLayer = critIdx < layerData.length
    ? layerData[critIdx]?.name ?? "Rse"
    : "Rse";

  // Bereken gc per maand
  const monthlyGc: number[] = [];

  for (let m = 0; m < 12; m++) {
    const climate = monthlyClimate[m]!;
    const pe = (climate.rhE / 100) * saturationPressure(climate.thetaE);
    const thetaC = thetaI - (thetaI - climate.thetaE) * crit.rCum / rTotal;
    const pSatC = saturationPressure(thetaC);

    const sdIn = Math.max(crit.sdCum, 0.001);
    const sdOut = Math.max(sdTotal - crit.sdCum, 0.001);

    const gIn = DELTA_0 * (pi - pSatC) / sdIn;
    const gOut = DELTA_0 * (pSatC - pe) / sdOut;
    const gc = (gIn - gOut) * climate.days * 86400 * 1000;

    monthlyGc.push(gc);
  }

  // Start bij de maand met hoogste condensatie (ISO 13788)
  const startMonth = monthlyGc.indexOf(Math.max(...monthlyGc));

  // Itereer 2× door het jaar (24 maanden) zodat het stabiliseert
  let ma = 0;
  const settled: MonthlyMoistureResult[] = [];

  for (let iter = 0; iter < 24; iter++) {
    const m = (startMonth + iter) % 12;
    const climate = monthlyClimate[m]!;
    const pe = (climate.rhE / 100) * saturationPressure(climate.thetaE);
    const thetaC = thetaI - (thetaI - climate.thetaE) * crit.rCum / rTotal;
    const pSatC = saturationPressure(thetaC);

    let gc = monthlyGc[m]!;

    // Geen vocht → geen droging mogelijk
    if (ma <= 0 && gc < 0) gc = 0;
    // Droging kan niet meer vocht verwijderen dan aanwezig
    if (ma + gc < 0) gc = -ma;

    ma = Math.max(0, ma + gc);

    // Bewaar alleen de tweede iteratie (maand 12–23)
    if (iter >= 12) {
      let status: MoistureStatus;
      if (gc > 0.1) {
        status = "condensation";
      } else if (ma > 0.1) {
        status = "drying";
      } else {
        status = "dry";
      }

      settled.push({
        month: climate.month,
        thetaE: climate.thetaE,
        rhE: climate.rhE,
        pe,
        pi,
        thetaC,
        pSatC,
        gc,
        ma,
        status,
      });
    }
  }

  // Hersorteer naar kalendermaand (Jan–Dec)
  const months: MonthlyMoistureResult[] = [];
  for (let m = 0; m < 12; m++) {
    const target = monthlyClimate[m]!.month;
    const found = settled.find((r) => r.month === target);
    if (found) months.push(found);
  }

  const maxMa = Math.max(...months.map((r) => r.ma), 0);
  const wetMonths = months.filter((r) => r.ma > 0.1).length;
  const wetDays = wetMonths * 30;
  const driesOut = months[months.length - 1]!.ma < 0.1
    || months.every((r) => r.ma < 0.1);

  return {
    months,
    maxMa,
    wetDays,
    hasRisk: wetDays > 90,
    driesOut,
    planeInnerLayer,
    planeOuterLayer,
    planePosition: crit.xCum,
  };
}
