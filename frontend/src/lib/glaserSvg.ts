/**
 * Standalone SVG string generator voor het Glaser dampspanningsdiagram.
 *
 * Produceert een SVG string (geen React) die als base64 image in het
 * rapport kan worden geëmbed. Logica is equivalent aan GlaserDiagram.tsx.
 *
 * Hatch patterns komen uit hatchPatterns.ts (gedeeld met GlaserDiagram.tsx).
 */

import type { GlaserResult, LayerStudInfo } from "./glaserCalculation";
import {
  generateHatchPatternDefs,
  resolvePatternId,
} from "./hatchPatterns";
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

/** Minimum aantal zichtbare studs per laag. */
const MIN_VISIBLE_STUDS = 2;
/** Maximum aantal zichtbare studs per laag. */
const MAX_VISIBLE_STUDS = 5;

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

function categoryColor(cat: MaterialCategory): string {
  return MATERIAL_CATEGORY_VISUALS[cat]?.color ?? "#e5e7eb";
}

/** Isolatie-categorieën die een directe zigzag-arcering krijgen (geen tiled pattern). */
const INSULATION_CATEGORIES: ReadonlySet<MaterialCategory> = new Set([
  "isolatie_mineraal",
  "isolatie_kunststof",
  "isolatie_natuurlijk",
]);

/** Hoek van de zigzag-lijnen ten opzichte van horizontaal (60°). */
const ZIGZAG_ANGLE_DEG = 60;
const ZIGZAG_TAN = Math.tan((ZIGZAG_ANGLE_DEG * Math.PI) / 180); // ~1.732

/** Zigzag styling per isolatietype. */
const ZIGZAG_STYLE: Record<string, { strokeWidth: number; offset?: number }> = {
  isolatie_mineraal: { strokeWidth: 0.8 },
  isolatie_kunststof: { strokeWidth: 0.8, offset: 1.5 },
  isolatie_natuurlijk: { strokeWidth: 0.8 },
};

/**
 * Genereer een SVG path (M/L) voor een doorlopend zigzag-patroon binnen een band.
 */
function generateZigzagPath(
  bandX: number,
  bandY: number,
  bandW: number,
  bandH: number,
  periodScale = 1.0,
): string {
  const halfPeriod = (bandH / ZIGZAG_TAN) * periodScale;
  if (halfPeriod < 0.5 || bandW < 1) return "";

  const bottom = bandY + bandH;
  const top = bandY;

  const points: string[] = [];
  let x = bandX;
  let goingUp = true;

  points.push(`M${x.toFixed(1)},${bottom.toFixed(1)}`);

  while (x < bandX + bandW + halfPeriod) {
    x += halfPeriod;
    const y = goingUp ? top : bottom;
    points.push(`L${x.toFixed(1)},${y.toFixed(1)}`);
    goingUp = !goingUp;
  }

  return points.join(" ");
}

// ---------- Stud computation ----------

interface StudBand {
  x: number;
  w: number;
}

function computeStudBands(
  bandX: number,
  bandW: number,
  stud: LayerStudInfo,
): StudBand[] {
  if (bandW < 8) return [];

  const fraction = stud.width / stud.spacing;
  const studPixelWidth = Math.max(bandW * fraction, 2);

  const realCount = Math.floor(bandW / (stud.spacing / stud.width * studPixelWidth));
  const count = Math.min(
    Math.max(realCount, MIN_VISIBLE_STUDS),
    MAX_VISIBLE_STUDS,
  );

  const totalStudWidth = count * studPixelWidth;
  const totalGapWidth = bandW - totalStudWidth;
  const gap = totalGapWidth / (count + 1);

  const bands: StudBand[] = [];
  for (let i = 0; i < count; i++) {
    bands.push({
      x: bandX + gap * (i + 1) + studPixelWidth * i,
      w: studPixelWidth,
    });
  }
  return bands;
}

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
    layerHatchPatterns,
    layerStuds,
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
  parts.push(generateHatchPatternDefs());

  // Background
  parts.push(`<rect x="${MARGIN.left}" y="${MARGIN.top}" width="${PLOT_W}" height="${PLOT_H}" fill="white" stroke="#e7e5e4" stroke-width="1"/>`);

  // Layer bands with NEN 47 patterns and stud visualization
  let xCum = 0;
  for (let i = 0; i < layerThicknesses.length; i++) {
    const d = layerThicknesses[i]!;
    const cat = layerCategories[i] as MaterialCategory | undefined;
    const hatchOverride = layerHatchPatterns?.[i];
    const stud = layerStuds?.[i];
    const x = toX(xCum);
    const w = (d / totalThickness) * PLOT_W;
    const color = cat ? categoryColor(cat) : "#e5e7eb";

    // Isolatie-categorieën krijgen directe zigzag-lijnen, geen tiled pattern
    const isInsulation = cat ? INSULATION_CATEGORIES.has(cat) : false;
    const patternId = cat && !isInsulation ? resolvePatternId(cat, hatchOverride) : undefined;

    // Color fill
    parts.push(`<rect x="${x.toFixed(1)}" y="${MARGIN.top}" width="${Math.max(w, 1).toFixed(1)}" height="${PLOT_H}" fill="${color}" fill-opacity="0.55"/>`);

    // Pattern overlay (niet voor isolatie)
    if (patternId) {
      parts.push(`<rect x="${x.toFixed(1)}" y="${MARGIN.top}" width="${Math.max(w, 1).toFixed(1)}" height="${PLOT_H}" fill="url(#${patternId})"/>`);
    }

    // Directe zigzag-arcering voor isolatie-lagen (NEN 47)
    if (cat && isInsulation && w > 2) {
      const clipId = `insulation-clip-${i}`;
      const style = ZIGZAG_STYLE[cat] ?? { strokeWidth: 0.8 };
      const periodScale = cat === "isolatie_natuurlijk" ? 1.3 : 1.0;
      const extraWidth = PLOT_H / ZIGZAG_TAN;
      const mainPath = generateZigzagPath(
        x - extraWidth,
        MARGIN.top,
        w + 2 * extraWidth,
        PLOT_H,
        periodScale,
      );
      parts.push(`<clipPath id="${clipId}"><rect x="${x.toFixed(1)}" y="${MARGIN.top}" width="${w.toFixed(1)}" height="${PLOT_H}"/></clipPath>`);
      parts.push(`<g clip-path="url(#${clipId})">`);
      parts.push(`<path d="${mainPath}" fill="none" stroke="#555" stroke-width="${style.strokeWidth}" opacity="0.6" stroke-linejoin="round"/>`);
      if (style.offset) {
        const offsetPath = generateZigzagPath(
          x - extraWidth + style.offset,
          MARGIN.top,
          w + 2 * extraWidth,
          PLOT_H,
          periodScale,
        );
        parts.push(`<path d="${offsetPath}" fill="none" stroke="#555" stroke-width="${style.strokeWidth}" opacity="0.6" stroke-linejoin="round"/>`);
      }
      parts.push(`</g>`);
    }

    // Studs: verticale banden met stijl-patroon
    if (stud) {
      const studPatternId = resolvePatternId(stud.studCategory, stud.studHatchPattern);
      const studColor = categoryColor(stud.studCategory);
      const studBands = computeStudBands(x, w, stud);

      for (const sb of studBands) {
        // Stijl-kleur
        parts.push(`<rect x="${sb.x.toFixed(1)}" y="${MARGIN.top}" width="${sb.w.toFixed(1)}" height="${PLOT_H}" fill="${studColor}" fill-opacity="0.65"/>`);
        // Stijl-arcering
        if (studPatternId) {
          parts.push(`<rect x="${sb.x.toFixed(1)}" y="${MARGIN.top}" width="${sb.w.toFixed(1)}" height="${PLOT_H}" fill="url(#${studPatternId})"/>`);
        }
        // Scheidingslijnen
        parts.push(`<line x1="${sb.x.toFixed(1)}" y1="${MARGIN.top}" x2="${sb.x.toFixed(1)}" y2="${MARGIN.top + PLOT_H}" stroke="#78716c" stroke-width="0.3" stroke-opacity="0.5"/>`);
        parts.push(`<line x1="${(sb.x + sb.w).toFixed(1)}" y1="${MARGIN.top}" x2="${(sb.x + sb.w).toFixed(1)}" y2="${MARGIN.top + PLOT_H}" stroke="#78716c" stroke-width="0.3" stroke-opacity="0.5"/>`);
      }
    }

    // Layer separator
    if (i > 0) {
      parts.push(`<line x1="${x.toFixed(1)}" y1="${MARGIN.top}" x2="${x.toFixed(1)}" y2="${MARGIN.top + PLOT_H}" stroke="#78716c" stroke-width="0.5"/>`);
    }

    // Layer name
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
  // TextEncoder for reliable UTF-8 -> binary -> base64
  const bytes = new TextEncoder().encode(svg);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

/**
 * Render SVG to PNG base64 via offscreen canvas.
 * PyMuPDF can't handle SVG -- this converts to a raster image first.
 * Returns base64 PNG string (without data URI prefix).
 */
export async function svgToPngBase64(svg: string, scale = 3): Promise<string> {
  // Parse SVG dimensions from viewBox or width/height attributes
  const widthMatch = svg.match(/width="(\d+)"/);
  const heightMatch = svg.match(/height="(\d+)"/);
  const width = widthMatch?.[1] ? parseInt(widthMatch[1]) : 720;
  const height = heightMatch?.[1] ? parseInt(heightMatch[1]) : 380;

  const canvas = document.createElement("canvas");
  canvas.width = width * scale;
  canvas.height = height * scale;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Canvas 2D context unavailable");

  const blob = new Blob([svg], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);

  try {
    const img = new Image();
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = () => reject(new Error("SVG rendering failed"));
      img.src = url;
    });
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    // canvas.toDataURL returns "data:image/png;base64,..." -- strip prefix
    const dataUrl = canvas.toDataURL("image/png");
    return dataUrl.replace(/^data:image\/png;base64,/, "");
  } finally {
    URL.revokeObjectURL(url);
  }
}
