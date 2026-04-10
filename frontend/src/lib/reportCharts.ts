/**
 * SVG string builders voor PDF-rapport diagrammen.
 *
 * Rendert dezelfde charts als de screen-componenten in components/charts/,
 * maar als standalone SVG met inline hex-kleuren en report-specifieke
 * styling (donkere tekst op wit papier, geen Tailwind, geen CSS vars).
 *
 * Output wordt base64-encoded en meegegeven aan BM Reports API via
 * { data, media_type: "image/svg+xml", filename } in een block_image.
 */

import type {
  BoundaryType,
  BuildingSummary,
  Room,
  RoomResult,
} from "../types";
import { DEFAULT_THETA_WATER } from "./constants";
import { buildRoomLookup, computeDeltaT } from "../components/charts/deltaT";

// ---------------------------------------------------------------------------
// Report-specifieke styling: donker op wit, hex inline
// ---------------------------------------------------------------------------

const REPORT_COLORS = {
  text: "#111827",
  textMuted: "#6b7280",
  textSecondary: "#374151",
  grid: "#e5e7eb",
  gridStrong: "#9ca3af",
  background: "#ffffff",
  // Chart palette — identiek aan themes.css --domain-chart-* hex waarden.
  transmission: "#3b82f6",
  ventilation: "#22c55e",
  infiltration: "#8b5cf6",
  heatingUp: "#f59e0b",
  system: "#78716c",
  neighbor: "#8b5cf6",
  walls: "#ef4444",
  glazing: "#3b82f6",
  roofs: "#f59e0b",
  floors: "#22c55e",
  internalWalls: "#8b5cf6",
  other: "#78716c",
} as const;

const FONT_FAMILY = "Helvetica, Arial, sans-serif";

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/** Wrap SVG inhoud in een standalone <svg> element met xmlns + witte achtergrond. */
function wrapSvg(width: number, height: number, inner: string): string {
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" ` +
    `width="${width}" height="${height}">` +
    `<rect width="${width}" height="${height}" fill="${REPORT_COLORS.background}"/>` +
    inner +
    `</svg>`
  );
}

/** XML-escape voor tekst in SVG <text> elementen. */
function xmlEscape(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Base64 encode van UTF-8 SVG string voor BM Reports image_source. */
export function svgToBase64(svg: string): string {
  // btoa werkt alleen met latin-1 — UTF-8 via encodeURIComponent workaround.
  return btoa(unescape(encodeURIComponent(svg)));
}

/**
 * Rasterize een standalone SVG-string naar base64-encoded PNG data (geen
 * `data:` prefix), zodat BM Reports (PyMuPDF) het kan inserten.
 *
 * De BM Reports renderer ondersteunt `image/svg+xml` in het schema maar
 * de PyMuPDF backend kan SVG niet parsen — vandaar client-side rasterize.
 *
 * @param svg Standalone SVG string (inclusief `width`/`height` attributen)
 * @param scale DPI-multiplier voor scherpere output (default 2×)
 */
export async function rasterizeSvgToPng(
  svg: string,
  scale = 2,
): Promise<{ data: string; widthPx: number; heightPx: number }> {
  const widthMatch = svg.match(/<svg[^>]*\swidth="(\d+(?:\.\d+)?)"/);
  const heightMatch = svg.match(/<svg[^>]*\sheight="(\d+(?:\.\d+)?)"/);
  const widthStr = widthMatch?.[1];
  const heightStr = heightMatch?.[1];
  if (!widthStr || !heightStr) {
    throw new Error("SVG zonder width/height attributen kan niet worden gerasterized");
  }
  const baseWidth = parseFloat(widthStr);
  const baseHeight = parseFloat(heightStr);
  const widthPx = Math.max(1, Math.round(baseWidth * scale));
  const heightPx = Math.max(1, Math.round(baseHeight * scale));

  // Blob URL is betrouwbaarder dan data: URL voor grotere SVGs en vermijdt
  // dubbele base64 round-trips.
  const blob = new Blob([svg], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);

  try {
    const img = new Image();
    img.decoding = "sync";
    img.src = url;
    await img.decode();

    const canvas = document.createElement("canvas");
    canvas.width = widthPx;
    canvas.height = heightPx;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas 2D context niet beschikbaar");
    }
    ctx.fillStyle = REPORT_COLORS.background;
    ctx.fillRect(0, 0, widthPx, heightPx);
    ctx.drawImage(img, 0, 0, widthPx, heightPx);

    const dataUrl = canvas.toDataURL("image/png");
    const data = dataUrl.replace(/^data:image\/png;base64,/, "");
    return { data, widthPx, heightPx };
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** Round up to a "nice" max value for Y-axis. */
function niceMax(value: number): number {
  if (value <= 0) return 100;
  const magnitude = Math.pow(10, Math.floor(Math.log10(value)));
  const normalized = value / magnitude;
  if (normalized <= 1) return magnitude;
  if (normalized <= 2) return 2 * magnitude;
  if (normalized <= 5) return 5 * magnitude;
  return 10 * magnitude;
}

/** Generate evenly spaced tick values. */
function generateTicks(max: number): number[] {
  const TICK_COUNT = 5;
  const step = max / TICK_COUNT;
  return Array.from({ length: TICK_COUNT + 1 }, (_, i) => Math.round(i * step));
}

/** Format watts with k suffix for large values. */
function formatWatts(w: number): string {
  const rounded = Math.round(w);
  if (rounded >= 10000) return `${(rounded / 1000).toFixed(1)}k W`;
  return `${rounded.toLocaleString("nl-NL")} W`;
}

/** Describe an SVG arc path for a donut segment. */
function describeArc(
  cx: number,
  cy: number,
  outerR: number,
  innerR: number,
  startAngle: number,
  endAngle: number,
): string {
  const GAP = 0.02;
  const sA = startAngle + GAP;
  const eA = endAngle - GAP;

  if (eA <= sA) return "";

  const largeArc = eA - sA > Math.PI ? 1 : 0;

  const outerStart = polarToCart(cx, cy, outerR, sA);
  const outerEnd = polarToCart(cx, cy, outerR, eA);
  const innerStart = polarToCart(cx, cy, innerR, eA);
  const innerEnd = polarToCart(cx, cy, innerR, sA);

  return [
    `M ${outerStart.x.toFixed(2)} ${outerStart.y.toFixed(2)}`,
    `A ${outerR} ${outerR} 0 ${largeArc} 1 ${outerEnd.x.toFixed(2)} ${outerEnd.y.toFixed(2)}`,
    `L ${innerStart.x.toFixed(2)} ${innerStart.y.toFixed(2)}`,
    `A ${innerR} ${innerR} 0 ${largeArc} 0 ${innerEnd.x.toFixed(2)} ${innerEnd.y.toFixed(2)}`,
    "Z",
  ].join(" ");
}

function polarToCart(
  cx: number,
  cy: number,
  r: number,
  angle: number,
): { x: number; y: number } {
  return {
    x: cx + r * Math.cos(angle),
    y: cy + r * Math.sin(angle),
  };
}

// ---------------------------------------------------------------------------
// 1. Stacked bar — warmteverliezen per vertrek
// ---------------------------------------------------------------------------

interface StackedBarCategory {
  readonly key: "transmission" | "ventilation" | "infiltration" | "heating_up" | "system";
  readonly label: string;
  readonly color: string;
}

const STACKED_CATEGORIES: readonly StackedBarCategory[] = [
  { key: "transmission", label: "Transmissie", color: REPORT_COLORS.transmission },
  { key: "ventilation", label: "Ventilatie", color: REPORT_COLORS.ventilation },
  { key: "infiltration", label: "Infiltratie", color: REPORT_COLORS.infiltration },
  { key: "heating_up", label: "Opwarmtoeslag", color: REPORT_COLORS.heatingUp },
  { key: "system", label: "Systeemverliezen", color: REPORT_COLORS.system },
] as const;

function getStackedValue(room: RoomResult, key: StackedBarCategory["key"]): number {
  switch (key) {
    case "transmission":
      return Math.max(0, room.transmission.phi_t);
    case "ventilation":
      return Math.max(0, room.ventilation.phi_v);
    case "infiltration":
      return Math.max(0, room.infiltration.phi_i);
    case "heating_up":
      return Math.max(0, room.heating_up.phi_hu);
    case "system":
      return Math.max(0, room.system_losses.phi_system_total);
  }
}

/** 1. Gestapelde staafgrafiek — warmteverliezen per vertrek. */
export function buildStackedBarSvg(rooms: RoomResult[]): string | null {
  if (rooms.length === 0) return null;

  const CHART_LEFT = 40;
  const CHART_RIGHT = 16;
  const CHART_TOP = 12;
  const CHART_BOTTOM = 60;
  const BAR_GAP = 8;
  const LEGEND_HEIGHT = 24;
  const MIN_CHART_WIDTH = 600;
  const PER_ROOM_WIDTH = 60;
  const MAX_BAR_WIDTH = 40;
  const CHART_HEIGHT = 260;

  const chartWidth = Math.max(
    MIN_CHART_WIDTH,
    rooms.length * PER_ROOM_WIDTH + CHART_LEFT + CHART_RIGHT,
  );
  const plotWidth = chartWidth - CHART_LEFT - CHART_RIGHT;
  const plotHeight = CHART_HEIGHT - CHART_TOP - CHART_BOTTOM - LEGEND_HEIGHT;

  const totals = rooms.map((room) =>
    STACKED_CATEGORIES.reduce((sum, cat) => sum + getStackedValue(room, cat.key), 0),
  );
  const maxTotal = Math.max(...totals, 1);
  const yMax = niceMax(maxTotal);
  const yTicks = generateTicks(yMax);

  const barWidth = Math.min(
    MAX_BAR_WIDTH,
    (plotWidth - BAR_GAP * (rooms.length + 1)) / rooms.length,
  );
  const totalBarsWidth = rooms.length * barWidth + (rooms.length - 1) * BAR_GAP;
  const offsetX = CHART_LEFT + (plotWidth - totalBarsWidth) / 2;

  const yScale = (v: number): number =>
    CHART_TOP + plotHeight - (v / yMax) * plotHeight;

  const parts: string[] = [];

  // Y-axis gridlines + labels
  for (const tick of yTicks) {
    const y = yScale(tick);
    parts.push(
      `<line x1="${CHART_LEFT}" x2="${chartWidth - CHART_RIGHT}" ` +
        `y1="${y.toFixed(2)}" y2="${y.toFixed(2)}" ` +
        `stroke="${REPORT_COLORS.grid}" stroke-dasharray="3,3"/>`,
    );
    const tickLabel = tick >= 1000 ? `${(tick / 1000).toFixed(1)}k` : String(tick);
    parts.push(
      `<text x="${CHART_LEFT - 4}" y="${y.toFixed(2)}" ` +
        `text-anchor="end" dominant-baseline="middle" ` +
        `fill="${REPORT_COLORS.textMuted}" font-size="9" font-family="${FONT_FAMILY}">` +
        `${xmlEscape(tickLabel)}</text>`,
    );
  }

  // Baseline
  const baselineY = yScale(0);
  parts.push(
    `<line x1="${CHART_LEFT}" x2="${chartWidth - CHART_RIGHT}" ` +
      `y1="${baselineY.toFixed(2)}" y2="${baselineY.toFixed(2)}" ` +
      `stroke="${REPORT_COLORS.gridStrong}"/>`,
  );

  // Bars + room labels
  rooms.forEach((room, i) => {
    const x = offsetX + i * (barWidth + BAR_GAP);
    let stackY = yScale(0);

    for (const cat of STACKED_CATEGORIES) {
      const val = getStackedValue(room, cat.key);
      if (val <= 0) continue;
      const barH = (val / yMax) * plotHeight;
      const y = stackY - barH;
      stackY = y;
      parts.push(
        `<rect x="${x.toFixed(2)}" y="${y.toFixed(2)}" ` +
          `width="${barWidth.toFixed(2)}" height="${barH.toFixed(2)}" ` +
          `fill="${cat.color}" rx="1"/>`,
      );
    }

    const labelX = x + barWidth / 2;
    const labelY = yScale(0) + 6;
    const shortName =
      room.room_name.length > 12 ? room.room_name.slice(0, 11) + "\u2026" : room.room_name;
    parts.push(
      `<text x="${labelX.toFixed(2)}" y="${labelY.toFixed(2)}" ` +
        `text-anchor="end" fill="${REPORT_COLORS.textSecondary}" ` +
        `font-size="9" font-family="${FONT_FAMILY}" ` +
        `transform="rotate(-35, ${labelX.toFixed(2)}, ${labelY.toFixed(2)})">` +
        `${xmlEscape(shortName)}</text>`,
    );
  });

  // Legend
  const LEGEND_ITEM_WIDTH = 110;
  const legendY = CHART_HEIGHT - LEGEND_HEIGHT + 4;
  STACKED_CATEGORIES.forEach((cat, i) => {
    const itemX = CHART_LEFT + i * LEGEND_ITEM_WIDTH;
    parts.push(
      `<rect x="${itemX}" y="${legendY}" width="10" height="10" ` +
        `fill="${cat.color}" rx="2"/>`,
    );
    parts.push(
      `<text x="${itemX + 14}" y="${legendY + 5}" ` +
        `dominant-baseline="middle" fill="${REPORT_COLORS.textMuted}" ` +
        `font-size="9" font-family="${FONT_FAMILY}">${xmlEscape(cat.label)}</text>`,
    );
  });

  return wrapSvg(chartWidth, CHART_HEIGHT, parts.join(""));
}

// ---------------------------------------------------------------------------
// 2. Summary donut — gebouwtotaal per verliestype (legenda in SVG)
// ---------------------------------------------------------------------------

interface DonutSegment {
  readonly key: keyof BuildingSummary;
  readonly label: string;
  readonly color: string;
}

const DONUT_SEGMENTS: readonly DonutSegment[] = [
  { key: "total_envelope_loss", label: "Transmissie", color: REPORT_COLORS.transmission },
  { key: "total_ventilation_loss", label: "Ventilatie", color: REPORT_COLORS.ventilation },
  { key: "total_heating_up", label: "Opwarmtoeslag", color: REPORT_COLORS.heatingUp },
  { key: "total_system_losses", label: "Systeemverliezen", color: REPORT_COLORS.system },
  { key: "total_neighbor_loss", label: "Buurwoningverlies", color: REPORT_COLORS.neighbor },
] as const;

/** 2. Donut — gebouwtotaal per verliestype, met legenda binnen de SVG. */
export function buildSummaryDonutSvg(summary: BuildingSummary): string | null {
  const DONUT_SIZE = 200;
  const CENTER = DONUT_SIZE / 2;
  const OUTER_R = 80;
  const INNER_R = 52;
  const LEGEND_X = 220;
  const LEGEND_ROW_HEIGHT = 22;
  const TOTAL_WIDTH = 560;
  const TOTAL_HEIGHT = 200;

  const values = DONUT_SEGMENTS.map((s) => ({
    ...s,
    value: Math.max(0, (summary[s.key] as number) ?? 0),
  }));
  const total = values.reduce((sum, v) => sum + v.value, 0);
  if (total <= 0) return null;

  // Build arc paths
  let startAngle = -Math.PI / 2;
  const arcs = values
    .filter((v) => v.value > 0)
    .map((v) => {
      const fraction = v.value / total;
      const angle = fraction * 2 * Math.PI;
      const endAngle = startAngle + angle;
      const path = describeArc(CENTER, CENTER, OUTER_R, INNER_R, startAngle, endAngle);
      startAngle = endAngle;
      return { ...v, path, fraction };
    });

  const parts: string[] = [];

  // Donut arcs
  for (const arc of arcs) {
    parts.push(`<path d="${arc.path}" fill="${arc.color}"/>`);
  }

  // Center text — aansluitvermogen
  parts.push(
    `<text x="${CENTER}" y="${CENTER - 6}" text-anchor="middle" ` +
      `dominant-baseline="middle" fill="${REPORT_COLORS.text}" ` +
      `font-size="16" font-weight="bold" font-family="${FONT_FAMILY}">` +
      `${xmlEscape(formatWatts(summary.connection_capacity))}</text>`,
  );
  parts.push(
    `<text x="${CENTER}" y="${CENTER + 12}" text-anchor="middle" ` +
      `dominant-baseline="middle" fill="${REPORT_COLORS.textMuted}" ` +
      `font-size="10" font-family="${FONT_FAMILY}">aansluitvermogen</text>`,
  );

  // Legend — rijen rechts van de donut
  const legendStartY = (TOTAL_HEIGHT - arcs.length * LEGEND_ROW_HEIGHT) / 2;
  const LEGEND_SWATCH_SIZE = 12;
  const LEGEND_LABEL_X = LEGEND_X + LEGEND_SWATCH_SIZE + 8;
  const LEGEND_VALUE_X = LEGEND_X + 220;
  const LEGEND_PCT_X = LEGEND_X + 290;

  arcs.forEach((arc, i) => {
    const rowY = legendStartY + i * LEGEND_ROW_HEIGHT;
    const swatchY = rowY + 3;
    const textY = rowY + LEGEND_SWATCH_SIZE / 2 + 3;

    parts.push(
      `<rect x="${LEGEND_X}" y="${swatchY}" ` +
        `width="${LEGEND_SWATCH_SIZE}" height="${LEGEND_SWATCH_SIZE}" ` +
        `fill="${arc.color}" rx="2"/>`,
    );
    parts.push(
      `<text x="${LEGEND_LABEL_X}" y="${textY}" ` +
        `dominant-baseline="middle" fill="${REPORT_COLORS.textSecondary}" ` +
        `font-size="11" font-family="${FONT_FAMILY}">${xmlEscape(arc.label)}</text>`,
    );
    parts.push(
      `<text x="${LEGEND_VALUE_X}" y="${textY}" ` +
        `text-anchor="end" dominant-baseline="middle" ` +
        `fill="${REPORT_COLORS.text}" font-size="11" font-family="${FONT_FAMILY}">` +
        `${xmlEscape(`${Math.round(arc.value)} W`)}</text>`,
    );
    parts.push(
      `<text x="${LEGEND_PCT_X}" y="${textY}" ` +
        `text-anchor="end" dominant-baseline="middle" ` +
        `fill="${REPORT_COLORS.textMuted}" font-size="11" font-family="${FONT_FAMILY}">` +
        `${xmlEscape(`${(arc.fraction * 100).toFixed(0)}%`)}</text>`,
    );
  });

  return wrapSvg(TOTAL_WIDTH, TOTAL_HEIGHT, parts.join(""));
}

// ---------------------------------------------------------------------------
// 3. Construction loss — horizontale bars per constructietype
// ---------------------------------------------------------------------------

interface ConstructionMatchInput {
  boundary_type: BoundaryType;
  description: string;
  vertical_position?: string;
}

interface ConstructionCategory {
  label: string;
  color: string;
  matchFn: (ce: ConstructionMatchInput) => boolean;
}

function isGlazing(description: string): boolean {
  const d = description.toLowerCase();
  return (
    d.includes("glas") ||
    d.includes("kozijn") ||
    d.includes("raam") ||
    d.includes("deur") ||
    d.includes("venster") ||
    d.includes("beglazing") ||
    d.includes("hr++") ||
    d.includes("hr+") ||
    d.includes("triple") ||
    d.includes("cwa") ||
    d.includes("vliesgevel") ||
    d.includes("curtain")
  );
}

const CONSTRUCTION_CATEGORIES: readonly ConstructionCategory[] = [
  {
    label: "Buitenwanden",
    color: REPORT_COLORS.walls,
    matchFn: (ce) =>
      ce.boundary_type === "exterior" &&
      (ce.vertical_position === "wall" || !ce.vertical_position) &&
      !isGlazing(ce.description),
  },
  {
    label: "Beglazing / kozijnen",
    color: REPORT_COLORS.glazing,
    matchFn: (ce) => ce.boundary_type === "exterior" && isGlazing(ce.description),
  },
  {
    label: "Daken / plafonds",
    color: REPORT_COLORS.roofs,
    matchFn: (ce) =>
      ce.boundary_type === "exterior" && ce.vertical_position === "ceiling",
  },
  {
    label: "Vloeren / grond",
    color: REPORT_COLORS.floors,
    matchFn: (ce) =>
      ce.boundary_type === "ground" || ce.vertical_position === "floor",
  },
  {
    label: "Binnenwanden / buren",
    color: REPORT_COLORS.internalWalls,
    matchFn: (ce) =>
      ce.boundary_type === "adjacent_room" ||
      ce.boundary_type === "adjacent_building" ||
      ce.boundary_type === "unheated_space",
  },
  {
    label: "Grensvlak water",
    color: REPORT_COLORS.floors,
    matchFn: (ce) => ce.boundary_type === "water",
  },
];

const FALLBACK_LABEL = "Overig";
const FALLBACK_COLOR = REPORT_COLORS.other;

/**
 * Default interne temperaturen per room-function voor chart-weergave.
 * Wijken (bewust) licht af van `ROOM_FUNCTION_TEMPERATURES`: deze tabel
 * is historisch gebonden aan de visualisatie en wordt alleen gebruikt
 * voor `thetaI` van de "self"-ruimte in de chart.
 */
const DEFAULT_CHART_TEMPERATURES: Record<string, number> = {
  living_room: 20,
  kitchen: 20,
  bedroom: 20,
  bathroom: 24,
  toilet: 20,
  hallway: 15,
  landing: 15,
  storage: 15,
  attic: 15,
  custom: 20,
};

function chartDefaultTemperature(fn: string): number {
  return DEFAULT_CHART_TEMPERATURES[fn] ?? 20;
}

interface ConstructionBar {
  label: string;
  color: string;
  value: number;
}

/** 3. Horizontale bars — verlies per constructietype. */
export function buildConstructionLossSvg(
  rooms: Room[],
  thetaE: number,
  thetaWater?: number,
): string | null {
  const totals = new Map<string, { color: string; value: number }>();
  const thetaW = thetaWater ?? DEFAULT_THETA_WATER;
  const roomLookup = buildRoomLookup(rooms);

  for (const room of rooms) {
    const thetaI = room.custom_temperature ?? chartDefaultTemperature(room.function);

    for (const ce of room.constructions) {
      const dT = computeDeltaT(ce.boundary_type, thetaI, thetaE, ce, {
        rooms: roomLookup,
        thetaWater: thetaW,
      });
      const phiT = ce.u_value * ce.area * dT;
      if (phiT <= 0) continue;

      const matched = CONSTRUCTION_CATEGORIES.find((cat) =>
        cat.matchFn({
          boundary_type: ce.boundary_type,
          description: ce.description,
          vertical_position: ce.vertical_position,
        }),
      );
      const label = matched?.label ?? FALLBACK_LABEL;
      const color = matched?.color ?? FALLBACK_COLOR;

      const existing = totals.get(label);
      if (existing) {
        existing.value += phiT;
      } else {
        totals.set(label, { color, value: phiT });
      }
    }
  }

  const bars: ConstructionBar[] = [];
  for (const [label, data] of totals) {
    bars.push({ label, ...data });
  }
  bars.sort((a, b) => b.value - a.value);

  if (bars.length === 0) return null;

  const LABEL_WIDTH = 140;
  const BAR_AREA_WIDTH = 340;
  const VALUE_WIDTH = 60;
  const CHART_WIDTH = LABEL_WIDTH + BAR_AREA_WIDTH + VALUE_WIDTH;
  const BAR_HEIGHT = 18;
  const BAR_GAP = 5;
  const PADDING_TOP = 6;
  const PADDING_BOTTOM = 6;
  const CHART_HEIGHT =
    PADDING_TOP + bars.length * (BAR_HEIGHT + BAR_GAP) + PADDING_BOTTOM;

  const maxValue = Math.max(...bars.map((b) => b.value), 1);

  const parts: string[] = [];

  bars.forEach((bar, i) => {
    const y = PADDING_TOP + i * (BAR_HEIGHT + BAR_GAP);
    const barW = (bar.value / maxValue) * BAR_AREA_WIDTH;
    const labelX = LABEL_WIDTH - 8;
    const labelY = y + BAR_HEIGHT / 2;

    // Label
    parts.push(
      `<text x="${labelX}" y="${labelY.toFixed(2)}" ` +
        `text-anchor="end" dominant-baseline="middle" ` +
        `fill="${REPORT_COLORS.textSecondary}" font-size="10" font-family="${FONT_FAMILY}">` +
        `${xmlEscape(bar.label)}</text>`,
    );
    // Bar
    parts.push(
      `<rect x="${LABEL_WIDTH}" y="${y}" ` +
        `width="${Math.max(barW, 2).toFixed(2)}" height="${BAR_HEIGHT}" ` +
        `fill="${bar.color}" rx="3"/>`,
    );
    // Value
    parts.push(
      `<text x="${LABEL_WIDTH + BAR_AREA_WIDTH + 8}" y="${labelY.toFixed(2)}" ` +
        `dominant-baseline="middle" fill="${REPORT_COLORS.text}" ` +
        `font-size="10" font-weight="500" font-family="${FONT_FAMILY}">` +
        `${xmlEscape(`${Math.round(bar.value)} W`)}</text>`,
    );
  });

  return wrapSvg(CHART_WIDTH, CHART_HEIGHT, parts.join(""));
}
