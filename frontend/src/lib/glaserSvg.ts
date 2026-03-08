/**
 * Standalone SVG string generator voor het Glaser dampspanningsdiagram.
 *
 * Produceert een SVG string (geen React) die als base64 image in het
 * rapport kan worden geëmbed. Logica is equivalent aan GlaserDiagram.tsx.
 */

import type { GlaserResult } from "./glaserCalculation";
import {
  MATERIAL_CATEGORY_VISUALS,
  type MaterialCategory,
} from "./materialsDatabase";

// ---------- Layout constanten ----------

const WIDTH = 720;
const HEIGHT = 380;
const MARGIN = { top: 20, right: 25, bottom: 60, left: 58 };
const PLOT_W = WIDTH - MARGIN.left - MARGIN.right;
const PLOT_H = HEIGHT - MARGIN.top - MARGIN.bottom;

// ---------- Helpers ----------

function niceMax(value: number): number {
  if (value <= 0) return 100;
  const magnitude = Math.pow(10, Math.floor(Math.log10(value)));
  const normalized = value / magnitude;
  if (normalized <= 1) return magnitude;
  if (normalized <= 2) return 2 * magnitude;
  if (normalized <= 5) return 5 * magnitude;
  return 10 * magnitude;
}

function esc(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen - 1) + "\u2026";
}

// ---------- SVG hatching patterns ----------

const HATCH_PATTERNS = `
<defs>
  <pattern id="hatch-masonry" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <line x1="0" y1="0" x2="0" y2="8" stroke="rgba(0,0,0,0.18)" stroke-width="1"/>
    <line x1="4" y1="0" x2="4" y2="8" stroke="rgba(0,0,0,0.08)" stroke-width="0.5"/>
  </pattern>
  <pattern id="hatch-concrete" width="6" height="6" patternUnits="userSpaceOnUse">
    <circle cx="1.5" cy="1.5" r="0.7" fill="rgba(0,0,0,0.15)"/>
    <circle cx="4.5" cy="4.5" r="0.7" fill="rgba(0,0,0,0.15)"/>
  </pattern>
  <pattern id="hatch-insulation" width="10" height="8" patternUnits="userSpaceOnUse">
    <polyline points="0,6 2.5,2 5,6 7.5,2 10,6" fill="none" stroke="rgba(0,0,0,0.15)" stroke-width="0.8"/>
  </pattern>
  <pattern id="hatch-wood" width="12" height="6" patternUnits="userSpaceOnUse">
    <line x1="0" y1="2" x2="12" y2="2" stroke="rgba(0,0,0,0.12)" stroke-width="0.6"/>
    <line x1="0" y1="5" x2="12" y2="5" stroke="rgba(0,0,0,0.08)" stroke-width="0.4"/>
  </pattern>
  <pattern id="hatch-foil" width="4" height="3" patternUnits="userSpaceOnUse">
    <line x1="0" y1="1.5" x2="4" y2="1.5" stroke="rgba(0,0,0,0.2)" stroke-width="1"/>
  </pattern>
  <pattern id="hatch-metal" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <line x1="0" y1="0" x2="0" y2="4" stroke="rgba(0,0,0,0.2)" stroke-width="1"/>
  </pattern>
</defs>`;

// ---------- Generator ----------

export function generateGlaserSvg(
  result: GlaserResult,
  thetaI: number,
  thetaE: number,
): string {
  const {
    curvePoints,
    interfacePoints,
    layerThicknesses,
    layerNames,
    layerCategories,
    totalThickness,
  } = result;

  if (curvePoints.length < 2 || totalThickness <= 0) {
    return "";
  }

  // Scales
  const allP = [
    ...curvePoints.map((p) => p.pSat),
    ...interfacePoints.map((p) => p.pActual),
  ];
  const rawMax = Math.max(...allP, 100);
  const yMax = niceMax(rawMax * 1.1);

  const toX = (xMm: number) => MARGIN.left + (xMm / totalThickness) * PLOT_W;
  const toY = (pPa: number) => MARGIN.top + PLOT_H - (pPa / yMax) * PLOT_H;

  // Y-axis ticks
  const yTickCount = 5;
  const yStep = yMax / yTickCount;
  const yTicks: number[] = [];
  for (let i = 0; i <= yTickCount; i++) {
    yTicks.push(Math.round(yStep * i));
  }

  const parts: string[] = [];

  // SVG open
  parts.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${HEIGHT}" width="${WIDTH}" height="${HEIGHT}" style="font-family: system-ui, -apple-system, sans-serif;">`);
  parts.push(HATCH_PATTERNS);

  // Background
  parts.push(`<rect x="${MARGIN.left}" y="${MARGIN.top}" width="${PLOT_W}" height="${PLOT_H}" fill="white" stroke="#e7e5e4" stroke-width="1"/>`);

  // Layer bands
  let xCum = 0;
  for (let i = 0; i < layerThicknesses.length; i++) {
    const d = layerThicknesses[i]!;
    const cat = layerCategories[i] as MaterialCategory | undefined;
    const x = toX(xCum);
    const w = (d / totalThickness) * PLOT_W;
    const color = cat ? (MATERIAL_CATEGORY_VISUALS[cat]?.color ?? "#e5e7eb") : "#e5e7eb";
    const pattern = cat ? MATERIAL_CATEGORY_VISUALS[cat]?.patternId : undefined;

    parts.push(`<rect x="${x.toFixed(1)}" y="${MARGIN.top}" width="${Math.max(w, 1).toFixed(1)}" height="${PLOT_H}" fill="${color}" fill-opacity="0.55"/>`);
    if (pattern) {
      parts.push(`<rect x="${x.toFixed(1)}" y="${MARGIN.top}" width="${Math.max(w, 1).toFixed(1)}" height="${PLOT_H}" fill="url(#${pattern})"/>`);
    }
    if (i > 0) {
      parts.push(`<line x1="${x.toFixed(1)}" y1="${MARGIN.top}" x2="${x.toFixed(1)}" y2="${MARGIN.top + PLOT_H}" stroke="#78716c" stroke-width="0.5"/>`);
    }
    if (w > 14) {
      const name = esc(truncate(layerNames[i] ?? "", Math.max(4, Math.floor(w / 5.5))));
      parts.push(`<text x="${(x + w / 2).toFixed(1)}" y="${MARGIN.top + PLOT_H + 14}" text-anchor="middle" font-size="9" fill="#57534e" font-weight="500">${name}</text>`);
    }
    xCum += d;
  }

  // Y-axis gridlines and labels
  for (const tick of yTicks) {
    const y = toY(tick);
    if (tick > 0) {
      parts.push(`<line x1="${MARGIN.left}" y1="${y.toFixed(1)}" x2="${MARGIN.left + PLOT_W}" y2="${y.toFixed(1)}" stroke="#d6d3d1" stroke-width="0.5" stroke-dasharray="3,3"/>`);
    }
    parts.push(`<text x="${MARGIN.left - 6}" y="${(y + 3).toFixed(1)}" text-anchor="end" font-size="9" fill="#a8a29e">${tick}</text>`);
  }

  // Y-axis label
  parts.push(`<text x="12" y="${MARGIN.top + PLOT_H / 2}" text-anchor="middle" font-size="10" fill="#78716c" transform="rotate(-90, 12, ${MARGIN.top + PLOT_H / 2})">Dampdruk [Pa]</text>`);

  // Condensation zone
  const condensZones = buildCondensationPath(curvePoints, toX, toY);
  if (condensZones) {
    parts.push(`<path d="${condensZones}" fill="#fca5a5" fill-opacity="0.45"/>`);
  }

  // pSat curve (blue)
  const pSatPath = curvePoints
    .map((p, i) => `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pSat).toFixed(1)}`)
    .join(" ");
  parts.push(`<path d="${pSatPath}" fill="none" stroke="#2563eb" stroke-width="2.5" stroke-linejoin="round"/>`);

  // pActual line (amber dashed)
  const pActualPath = interfacePoints
    .map((p, i) => `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pActual).toFixed(1)}`)
    .join(" ");
  parts.push(`<path d="${pActualPath}" fill="none" stroke="#d97706" stroke-width="2.5" stroke-linejoin="round" stroke-dasharray="6,3"/>`);

  // Interface points
  for (const p of interfacePoints) {
    parts.push(`<circle cx="${toX(p.x).toFixed(1)}" cy="${toY(p.pSat).toFixed(1)}" r="3" fill="#2563eb"/>`);
    parts.push(`<circle cx="${toX(p.x).toFixed(1)}" cy="${toY(p.pActual).toFixed(1)}" r="3" fill="#d97706"/>`);
  }

  // Temperature labels at first and last interface
  for (let i = 0; i < interfacePoints.length; i++) {
    const p = interfacePoints[i]!;
    if (interfacePoints.length > 4 && i !== 0 && i !== interfacePoints.length - 1) continue;
    parts.push(`<text x="${toX(p.x).toFixed(1)}" y="${MARGIN.top + PLOT_H + 28}" text-anchor="middle" font-size="9" fill="#78716c">${p.temperature.toFixed(1)}°C</text>`);
  }

  // Inside/outside labels
  parts.push(`<text x="${MARGIN.left + 4}" y="${MARGIN.top + PLOT_H + 46}" font-size="10" font-weight="600" fill="#57534e">Binnen (${thetaI}°C)</text>`);
  parts.push(`<text x="${MARGIN.left + PLOT_W - 4}" y="${MARGIN.top + PLOT_H + 46}" text-anchor="end" font-size="10" font-weight="600" fill="#57534e">Buiten (${thetaE}°C)</text>`);

  // Legend
  const lx = MARGIN.left + PLOT_W - 200;
  const ly = MARGIN.top + 8;
  parts.push(`<g transform="translate(${lx}, ${ly})">`);
  parts.push(`<rect x="-6" y="-4" width="206" height="42" rx="4" fill="white" fill-opacity="0.92" stroke="#d6d3d1" stroke-width="0.5"/>`);
  parts.push(`<line x1="0" y1="8" x2="18" y2="8" stroke="#2563eb" stroke-width="2.5"/>`);
  parts.push(`<circle cx="9" cy="8" r="3" fill="#2563eb"/>`);
  parts.push(`<text x="24" y="11" font-size="10" fill="#57534e">Verzadigingsdruk (p_sat)</text>`);
  parts.push(`<line x1="0" y1="26" x2="18" y2="26" stroke="#d97706" stroke-width="2.5" stroke-dasharray="6,3"/>`);
  parts.push(`<circle cx="9" cy="26" r="3" fill="#d97706"/>`);
  parts.push(`<text x="24" y="29" font-size="10" fill="#57534e">Werkelijke dampdruk (p)</text>`);
  parts.push(`</g>`);

  parts.push(`</svg>`);

  return parts.join("\n");
}

// ---------- Condensation zone helper ----------

function buildCondensationPath(
  curvePoints: GlaserResult["curvePoints"],
  toX: (x: number) => number,
  toY: (p: number) => number,
): string {
  const zones: string[] = [];
  let inZone = false;
  let zoneStart = "";

  for (let i = 0; i < curvePoints.length; i++) {
    const p = curvePoints[i]!;
    const x = toX(p.x).toFixed(1);
    const yActual = toY(p.pActual).toFixed(1);
    const ySat = toY(p.pSat).toFixed(1);

    if (p.pActual > p.pSat + 0.5) {
      if (!inZone) {
        zoneStart = `M${x},${ySat} L${x},${yActual}`;
        inZone = true;
      } else {
        zoneStart += ` L${x},${yActual}`;
      }
    } else if (inZone) {
      const backPath = curvePoints
        .slice(0, i)
        .reverse()
        .filter((bp) => bp.pActual > bp.pSat + 0.5)
        .map((bp) => `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`)
        .join(" ");
      zones.push(`${zoneStart} ${backPath} Z`);
      inZone = false;
    }
  }

  if (inZone) {
    const lastInZone = curvePoints.filter((p) => p.pActual > p.pSat + 0.5);
    const backPath = [...lastInZone]
      .reverse()
      .map((bp) => `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`)
      .join(" ");
    zones.push(`${zoneStart} ${backPath} Z`);
  }

  return zones.join(" ");
}

/** Encode SVG string to base64 for embedding in report JSON. */
export function svgToBase64(svg: string): string {
  // TextEncoder for reliable UTF-8 → binary → base64
  const bytes = new TextEncoder().encode(svg);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}
