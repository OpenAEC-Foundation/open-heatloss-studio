/**
 * Standalone SVG chart voor de jaarlijkse vochtbalans.
 *
 * Toont per maand:
 * - Staafdiagram: condensatie (rood) / droging (blauw) snelheid g_c
 * - Lijn: opgebouwd vocht M_a (oranje)
 *
 * Produceert een SVG string die als base64 image in het rapport wordt geëmbed.
 */

import type { YearlyMoistureResult } from "./yearlyMoistureCalculation";

// ---------- Layout constanten ----------

const WIDTH = 720;
const HEIGHT = 320;
const MARGIN = { top: 20, right: 60, bottom: 48, left: 58 };
const PLOT_W = WIDTH - MARGIN.left - MARGIN.right;
const PLOT_H = HEIGHT - MARGIN.top - MARGIN.bottom;

// ---------- Helpers ----------

function niceMax(value: number): number {
  if (value <= 0) return 10;
  const magnitude = Math.pow(10, Math.floor(Math.log10(value)));
  const normalized = value / magnitude;
  if (normalized <= 1) return magnitude;
  if (normalized <= 2) return 2 * magnitude;
  if (normalized <= 5) return 5 * magnitude;
  return 10 * magnitude;
}

function generateTicks(min: number, max: number, count: number): number[] {
  const step = (max - min) / count;
  const ticks: number[] = [];
  for (let i = 0; i <= count; i++) {
    ticks.push(Math.round((min + step * i) * 10) / 10);
  }
  return ticks;
}

// ---------- Generator ----------

export function generateMoistureYearSvg(result: YearlyMoistureResult): string {
  const { months } = result;
  if (months.length === 0) return "";

  // Determine scales
  const gcValues = months.map((m) => m.gc);
  const maValues = months.map((m) => m.ma);

  const gcMax = niceMax(Math.max(...gcValues.map(Math.abs), 1));
  const gcMin = -gcMax; // Symmetric around zero
  const maMax = niceMax(Math.max(...maValues, 1));

  const barWidth = PLOT_W / 12;
  const barPad = barWidth * 0.15;

  // Scale functions
  const toX = (i: number) => MARGIN.left + i * barWidth + barWidth / 2;
  const toYgc = (gc: number) =>
    MARGIN.top + PLOT_H / 2 - (gc / gcMax) * (PLOT_H / 2);
  const toYma = (ma: number) =>
    MARGIN.top + PLOT_H - (ma / maMax) * PLOT_H;
  const zeroY = MARGIN.top + PLOT_H / 2;

  const parts: string[] = [];

  // SVG open
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${HEIGHT}" width="${WIDTH}" height="${HEIGHT}" style="font-family: system-ui, -apple-system, sans-serif;">`,
  );

  // Background
  parts.push(
    `<rect x="${MARGIN.left}" y="${MARGIN.top}" width="${PLOT_W}" height="${PLOT_H}" fill="white" stroke="#e7e5e4" stroke-width="1"/>`,
  );

  // Y-axis left: g_c ticks
  const gcTicks = generateTicks(gcMin, gcMax, 6);
  for (const tick of gcTicks) {
    const y = toYgc(tick);
    parts.push(
      `<line x1="${MARGIN.left}" y1="${y.toFixed(1)}" x2="${MARGIN.left + PLOT_W}" y2="${y.toFixed(1)}" stroke="#e7e5e4" stroke-width="0.5" stroke-dasharray="3,3"/>`,
    );
    parts.push(
      `<text x="${MARGIN.left - 6}" y="${(y + 3).toFixed(1)}" text-anchor="end" font-size="9" fill="#a8a29e">${tick.toFixed(0)}</text>`,
    );
  }

  // Zero line
  parts.push(
    `<line x1="${MARGIN.left}" y1="${zeroY.toFixed(1)}" x2="${MARGIN.left + PLOT_W}" y2="${zeroY.toFixed(1)}" stroke="#78716c" stroke-width="1"/>`,
  );

  // Y-axis right: M_a ticks
  const maTicks = generateTicks(0, maMax, 4);
  for (const tick of maTicks) {
    const y = toYma(tick);
    parts.push(
      `<text x="${MARGIN.left + PLOT_W + 6}" y="${(y + 3).toFixed(1)}" text-anchor="start" font-size="9" fill="#d97706">${tick.toFixed(0)}</text>`,
    );
  }

  // Y-axis labels
  parts.push(
    `<text x="12" y="${MARGIN.top + PLOT_H / 2}" text-anchor="middle" font-size="10" fill="#78716c" transform="rotate(-90, 12, ${MARGIN.top + PLOT_H / 2})">g_c [g/m\u00B2]</text>`,
  );
  parts.push(
    `<text x="${WIDTH - 8}" y="${MARGIN.top + PLOT_H / 2}" text-anchor="middle" font-size="10" fill="#d97706" transform="rotate(90, ${WIDTH - 8}, ${MARGIN.top + PLOT_H / 2})">M_a [g/m\u00B2]</text>`,
  );

  // Bars (g_c)
  for (let i = 0; i < months.length; i++) {
    const m = months[i]!;
    const x = MARGIN.left + i * barWidth + barPad;
    const w = barWidth - 2 * barPad;

    if (Math.abs(m.gc) > 0.01) {
      const color = m.gc > 0 ? "#ef4444" : "#3b82f6";
      const opacity = m.gc > 0 ? "0.7" : "0.6";
      const barTop = m.gc > 0 ? toYgc(m.gc) : zeroY;
      const barH = Math.abs(toYgc(m.gc) - zeroY);

      parts.push(
        `<rect x="${x.toFixed(1)}" y="${barTop.toFixed(1)}" width="${w.toFixed(1)}" height="${Math.max(barH, 1).toFixed(1)}" fill="${color}" fill-opacity="${opacity}" rx="1"/>`,
      );
    }

    // Month label
    parts.push(
      `<text x="${toX(i).toFixed(1)}" y="${MARGIN.top + PLOT_H + 14}" text-anchor="middle" font-size="9" fill="#57534e" font-weight="500">${m.month}</text>`,
    );
  }

  // M_a line (orange)
  const maPath = months
    .map(
      (m, i) =>
        `${i === 0 ? "M" : "L"}${toX(i).toFixed(1)},${toYma(m.ma).toFixed(1)}`,
    )
    .join(" ");
  parts.push(
    `<path d="${maPath}" fill="none" stroke="#d97706" stroke-width="2.5" stroke-linejoin="round"/>`,
  );

  // M_a dots
  for (let i = 0; i < months.length; i++) {
    const m = months[i]!;
    parts.push(
      `<circle cx="${toX(i).toFixed(1)}" cy="${toYma(m.ma).toFixed(1)}" r="3" fill="#d97706"/>`,
    );
  }

  // Legend
  const lx = MARGIN.left + PLOT_W - 260;
  const ly = MARGIN.top + 8;
  parts.push(`<g transform="translate(${lx}, ${ly})">`);
  parts.push(
    `<rect x="-6" y="-4" width="266" height="28" rx="4" fill="white" fill-opacity="0.92" stroke="#d6d3d1" stroke-width="0.5"/>`,
  );
  // Condensation bar
  parts.push(`<rect x="0" y="3" width="14" height="10" fill="#ef4444" fill-opacity="0.7" rx="1"/>`);
  parts.push(`<text x="18" y="12" font-size="9" fill="#57534e">Condensatie</text>`);
  // Drying bar
  parts.push(`<rect x="80" y="3" width="14" height="10" fill="#3b82f6" fill-opacity="0.6" rx="1"/>`);
  parts.push(`<text x="98" y="12" font-size="9" fill="#57534e">Droging</text>`);
  // Ma line
  parts.push(`<line x1="148" y1="8" x2="166" y2="8" stroke="#d97706" stroke-width="2.5"/>`);
  parts.push(`<circle cx="157" cy="8" r="3" fill="#d97706"/>`);
  parts.push(`<text x="170" y="12" font-size="9" fill="#57534e">Opgebouwd vocht (M_a)</text>`);
  parts.push(`</g>`);

  // Condensation/drying zone label below
  parts.push(
    `<text x="${MARGIN.left + 4}" y="${MARGIN.top + PLOT_H + 32}" font-size="9" fill="#a8a29e">Boven nul = condensatie, onder nul = droging</text>`,
  );

  parts.push(`</svg>`);

  return parts.join("\n");
}
