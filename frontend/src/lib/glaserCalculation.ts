/**
 * Dampspanningsberekening volgens de Glaser-methode (NEN-EN-ISO 13788).
 *
 * Berekent temperatuur- en dampdrukverloop door een constructie-opbouw
 * om condensatierisico te beoordelen.
 */

import type { VerticalPosition } from "../types";

import { getMaterialById } from "./materialsDatabase";

// ---------- Oppervlakteweerstanden ----------

const R_SI: Record<VerticalPosition, number> = {
  wall: 0.13,
  ceiling: 0.10,
  floor: 0.17,
};
const R_SE = 0.04;

// ---------- Standaard klimaatcondities ----------

export const GLASER_DEFAULTS = {
  thetaI: 20,
  thetaE: -10,
  rhI: 50,
  rhE: 90,
} as const;

// ---------- Types ----------

export interface GlaserPoint {
  /** Positie vanaf binnenoppervlak [mm]. */
  x: number;
  /** Temperatuur [°C]. */
  temperature: number;
  /** Verzadigingsdampdruk [Pa]. */
  pSat: number;
  /** Werkelijke dampdruk [Pa]. */
  pActual: number;
}

export interface GlaserResult {
  /** Punten op laag-grensvlakken (voor pActual lijnstukken). */
  interfacePoints: GlaserPoint[];
  /** Dichte punten voor vloeiende pSat-curve. */
  curvePoints: GlaserPoint[];
  /** Laagnamen in volgorde. */
  layerNames: string[];
  /** Laagdiktes [mm] in volgorde. */
  layerThicknesses: number[];
  /** Totale dikte [mm]. */
  totalThickness: number;
  /** Condensatierisico aanwezig. */
  hasCondensation: boolean;
  /** Dampdruk binnenlucht [Pa]. */
  pI: number;
  /** Dampdruk buitenlucht [Pa]. */
  pE: number;
}

export interface GlaserInput {
  layers: { materialId: string; thickness: number }[];
  position: VerticalPosition;
  thetaI: number;
  thetaE: number;
  /** Relatieve luchtvochtigheid binnen [%]. */
  rhI: number;
  /** Relatieve luchtvochtigheid buiten [%]. */
  rhE: number;
}

// ---------- Verzadigingsdampdruk (Magnus/Tetens, NEN-EN-ISO 13788) ----------

/** Verzadigingsdampdruk [Pa] bij temperatuur theta [°C]. */
export function saturationPressure(theta: number): number {
  if (theta >= 0) {
    return 610.5 * Math.exp((17.269 * theta) / (237.3 + theta));
  }
  return 610.5 * Math.exp((21.875 * theta) / (265.5 + theta));
}

// ---------- Interne laagdata ----------

interface LayerData {
  name: string;
  thickness: number;
  r: number;
  sd: number;
}

// ---------- Berekening ----------

/** Aantal tussenpunten per laag voor vloeiende pSat-curve. */
const CURVE_SUBDIVISIONS = 8;

export function calculateGlaser(input: GlaserInput): GlaserResult {
  const { layers: layerInputs, position, thetaI, thetaE, rhI, rhE } = input;

  const rSi = R_SI[position];
  const rSe = R_SE;

  // Bouw laagdata op
  const layerData: LayerData[] = [];
  for (const li of layerInputs) {
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

    const sd = mat.mu * (li.thickness / 1000);

    layerData.push({ name: mat.name, thickness: li.thickness, r, sd });
  }

  const rcTotal = layerData.reduce((s, l) => s + l.r, 0);
  const rTotal = rSi + rcTotal + rSe;
  const sdTotal = layerData.reduce((s, l) => s + l.sd, 0);
  const totalThickness = layerData.reduce((s, l) => s + l.thickness, 0);

  const pI = (rhI / 100) * saturationPressure(thetaI);
  const pE = (rhE / 100) * saturationPressure(thetaE);

  // Helper: temperatuur bij cumulatieve R (inclusief Rsi)
  const tempAt = (rCum: number): number =>
    thetaI - (thetaI - thetaE) * rCum / rTotal;

  // Helper: dampdruk bij cumulatieve sd
  const pressureAt = (sdCum: number): number =>
    sdTotal > 0 ? pI - (pI - pE) * sdCum / sdTotal : pI;

  // Interface-punten (laag-grensvlakken)
  const interfacePoints: GlaserPoint[] = [];
  let rCum = rSi;
  let sdCum = 0;
  let xCum = 0;

  // Binnenoppervlak
  const tInside = tempAt(rCum);
  interfacePoints.push({
    x: 0,
    temperature: tInside,
    pSat: saturationPressure(tInside),
    pActual: pI,
  });

  for (const layer of layerData) {
    rCum += layer.r;
    sdCum += layer.sd;
    xCum += layer.thickness;

    const t = tempAt(rCum);
    interfacePoints.push({
      x: xCum,
      temperature: t,
      pSat: saturationPressure(t),
      pActual: pressureAt(sdCum),
    });
  }

  // Dichte curve-punten voor vloeiende pSat-lijn
  const curvePoints: GlaserPoint[] = [];
  rCum = rSi;
  sdCum = 0;
  xCum = 0;

  // Startpunt
  curvePoints.push({ ...interfacePoints[0]! });

  for (const layer of layerData) {
    const rStep = layer.r / CURVE_SUBDIVISIONS;
    const sdStep = layer.sd / CURVE_SUBDIVISIONS;
    const xStep = layer.thickness / CURVE_SUBDIVISIONS;

    for (let j = 1; j <= CURVE_SUBDIVISIONS; j++) {
      const rAt = rCum + rStep * j;
      const sdAt = sdCum + sdStep * j;
      const xAt = xCum + xStep * j;

      const t = tempAt(rAt);
      curvePoints.push({
        x: xAt,
        temperature: t,
        pSat: saturationPressure(t),
        pActual: pressureAt(sdAt),
      });
    }

    rCum += layer.r;
    sdCum += layer.sd;
    xCum += layer.thickness;
  }

  // Condensatiecheck: ergens pActual > pSat?
  const hasCondensation = curvePoints.some((p) => p.pActual > p.pSat + 0.5);

  return {
    interfacePoints,
    curvePoints,
    layerNames: layerData.map((l) => l.name),
    layerThicknesses: layerData.map((l) => l.thickness),
    totalThickness,
    hasCondensation,
    pI,
    pE,
  };
}
