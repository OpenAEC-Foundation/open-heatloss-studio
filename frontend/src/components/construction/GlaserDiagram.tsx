/**
 * SVG-diagram voor de Glaser-methode (dampspanningsverloop).
 *
 * Toont verzadigingsdampdruk (pSat) en werkelijke dampdruk (pActual)
 * door de constructie-opbouw heen. Condensatiezones worden rood gemarkeerd.
 */

import { useMemo } from "react";

import type { GlaserResult } from "../../lib/glaserCalculation";

interface GlaserDiagramProps {
  result: GlaserResult;
  thetaI: number;
  thetaE: number;
}

// ---------- Layout constanten ----------

const WIDTH = 640;
const HEIGHT = 340;
const MARGIN = { top: 20, right: 25, bottom: 60, left: 58 };
const PLOT_W = WIDTH - MARGIN.left - MARGIN.right;
const PLOT_H = HEIGHT - MARGIN.top - MARGIN.bottom;

const BAND_COLORS = ["#fafaf9", "#f5f5f4"]; // stone-50, stone-100

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

function generateTicks(max: number, count: number): number[] {
  const step = max / count;
  const ticks: number[] = [];
  for (let i = 0; i <= count; i++) {
    ticks.push(Math.round(step * i));
  }
  return ticks;
}

/** Kort materiaal-/laagnaam af tot max N tekens. */
function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen - 1) + "\u2026";
}

// ---------- Component ----------

export function GlaserDiagram({ result, thetaI, thetaE }: GlaserDiagramProps) {
  const { curvePoints, interfacePoints, layerThicknesses, layerNames, totalThickness } =
    result;

  const hasLayers = curvePoints.length >= 2 && totalThickness > 0;

  // Schalen berekenen
  const { yTicks, toX, toY } = useMemo(() => {
    if (!hasLayers) {
      return {
        maxP: 2500,
        yTicks: [0, 500, 1000, 1500, 2000, 2500],
        toX: () => MARGIN.left,
        toY: () => MARGIN.top + PLOT_H,
      };
    }

    const allP = [
      ...curvePoints.map((p) => p.pSat),
      ...interfacePoints.map((p) => p.pActual),
    ];
    const rawMax = Math.max(...allP, 100);
    const nMax = niceMax(rawMax * 1.1);
    const ticks = generateTicks(nMax, 5);

    return {
      maxP: nMax,
      yTicks: ticks,
      toX: (xMm: number) => MARGIN.left + (xMm / totalThickness) * PLOT_W,
      toY: (pPa: number) => MARGIN.top + PLOT_H - (pPa / nMax) * PLOT_H,
    };
  }, [curvePoints, interfacePoints, totalThickness, hasLayers]);

  // pSat-curve pad
  const pSatPath = useMemo(() => {
    if (!hasLayers) return "";
    return curvePoints
      .map((p, i) => `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pSat).toFixed(1)}`)
      .join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // pActual-lijn pad
  const pActualPath = useMemo(() => {
    if (!hasLayers) return "";
    return interfacePoints
      .map((p, i) => `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pActual).toFixed(1)}`)
      .join(" ");
  }, [interfacePoints, hasLayers, toX, toY]);

  // Condensatiezone: gebied waar pActual > pSat
  const condensationPath = useMemo(() => {
    if (!hasLayers) return "";

    // Zoek segmenten waar pActual > pSat in curvePoints
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
        // Sluit zone: ga terug via pSat-lijn
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

    // Sluit evt. openstaande zone
    if (inZone) {
      const lastInZone = curvePoints.filter((p) => p.pActual > p.pSat + 0.5);
      const backPath = [...lastInZone]
        .reverse()
        .map((bp) => `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`)
        .join(" ");
      zones.push(`${zoneStart} ${backPath} Z`);
    }

    return zones.join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // Laag-banden x-posities
  const layerBands = useMemo(() => {
    if (!hasLayers) return [];
    const bands: { x: number; w: number; name: string; color: string }[] = [];
    let xCum = 0;
    for (let i = 0; i < layerThicknesses.length; i++) {
      const d = layerThicknesses[i]!;
      bands.push({
        x: toX(xCum),
        w: (d / totalThickness) * PLOT_W,
        name: layerNames[i] ?? "",
        color: BAND_COLORS[i % 2]!,
      });
      xCum += d;
    }
    return bands;
  }, [layerThicknesses, layerNames, totalThickness, hasLayers, toX]);

  if (!hasLayers) {
    return (
      <div className="flex h-48 items-center justify-center rounded-lg border border-dashed border-stone-300 text-sm text-stone-400">
        Voeg lagen toe om het dampspanningsdiagram te zien.
      </div>
    );
  }

  return (
    <svg
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      className="w-full"
      style={{ maxHeight: 380 }}
    >
      {/* Achtergrond */}
      <rect
        x={MARGIN.left}
        y={MARGIN.top}
        width={PLOT_W}
        height={PLOT_H}
        fill="white"
        stroke="#e7e5e4"
        strokeWidth={1}
      />

      {/* Laag-banden */}
      {layerBands.map((band, i) => (
        <g key={i}>
          <rect
            x={band.x}
            y={MARGIN.top}
            width={Math.max(band.w, 1)}
            height={PLOT_H}
            fill={band.color}
          />
          {/* Laag-scheidingslijn */}
          {i > 0 && (
            <line
              x1={band.x}
              y1={MARGIN.top}
              x2={band.x}
              y2={MARGIN.top + PLOT_H}
              stroke="#d6d3d1"
              strokeWidth={0.5}
              strokeDasharray="3,2"
            />
          )}
          {/* Laagnaam */}
          {band.w > 12 && (
            <text
              x={band.x + band.w / 2}
              y={MARGIN.top + PLOT_H + 14}
              textAnchor="middle"
              fontSize={9}
              fill="#78716c"
              className="select-none"
            >
              {truncate(band.name, Math.max(4, Math.floor(band.w / 5.5)))}
            </text>
          )}
        </g>
      ))}

      {/* Y-as gridlijnen en labels */}
      {yTicks.map((tick) => {
        const y = toY(tick);
        return (
          <g key={tick}>
            {tick > 0 && (
              <line
                x1={MARGIN.left}
                y1={y}
                x2={MARGIN.left + PLOT_W}
                y2={y}
                stroke="#e7e5e4"
                strokeWidth={0.5}
              />
            )}
            <text
              x={MARGIN.left - 6}
              y={y + 3}
              textAnchor="end"
              fontSize={9}
              fill="#a8a29e"
            >
              {tick}
            </text>
          </g>
        );
      })}

      {/* Y-as label */}
      <text
        x={12}
        y={MARGIN.top + PLOT_H / 2}
        textAnchor="middle"
        fontSize={10}
        fill="#78716c"
        transform={`rotate(-90, 12, ${MARGIN.top + PLOT_H / 2})`}
      >
        Dampdruk [Pa]
      </text>

      {/* Condensatiezone */}
      {condensationPath && (
        <path d={condensationPath} fill="#fca5a5" fillOpacity={0.4} />
      )}

      {/* pSat curve (blauw) */}
      <path
        d={pSatPath}
        fill="none"
        stroke="#3b82f6"
        strokeWidth={2}
        strokeLinejoin="round"
      />

      {/* pActual lijn (amber) */}
      <path
        d={pActualPath}
        fill="none"
        stroke="#f59e0b"
        strokeWidth={2}
        strokeLinejoin="round"
      />

      {/* Punten op interfaces */}
      {interfacePoints.map((p, i) => (
        <g key={i}>
          <circle cx={toX(p.x)} cy={toY(p.pSat)} r={2.5} fill="#3b82f6" />
          <circle cx={toX(p.x)} cy={toY(p.pActual)} r={2.5} fill="#f59e0b" />
        </g>
      ))}

      {/* Temperatuurlabels bij interface-punten */}
      {interfacePoints.map((p, i) => {
        const x = toX(p.x);
        // Alleen eerste en laatste tonen, of als er weinig punten zijn
        if (
          interfacePoints.length > 4 &&
          i !== 0 &&
          i !== interfacePoints.length - 1
        )
          return null;
        return (
          <text
            key={`t-${i}`}
            x={x}
            y={MARGIN.top + PLOT_H + 28}
            textAnchor="middle"
            fontSize={9}
            fill="#78716c"
          >
            {p.temperature.toFixed(1)}°C
          </text>
        );
      })}

      {/* Binnen/Buiten labels */}
      <text
        x={MARGIN.left + 4}
        y={MARGIN.top + PLOT_H + 46}
        fontSize={10}
        fontWeight={600}
        fill="#57534e"
      >
        Binnen ({thetaI}°C)
      </text>
      <text
        x={MARGIN.left + PLOT_W - 4}
        y={MARGIN.top + PLOT_H + 46}
        textAnchor="end"
        fontSize={10}
        fontWeight={600}
        fill="#57534e"
      >
        Buiten ({thetaE}°C)
      </text>

      {/* Legenda */}
      <g transform={`translate(${MARGIN.left + PLOT_W - 190}, ${MARGIN.top + 8})`}>
        <rect
          x={-6}
          y={-4}
          width={196}
          height={42}
          rx={4}
          fill="white"
          fillOpacity={0.9}
          stroke="#e7e5e4"
          strokeWidth={0.5}
        />
        <line x1={0} y1={8} x2={18} y2={8} stroke="#3b82f6" strokeWidth={2} />
        <circle cx={9} cy={8} r={2.5} fill="#3b82f6" />
        <text x={24} y={11} fontSize={10} fill="#57534e">
          Verzadigingsdruk (p_sat)
        </text>
        <line x1={0} y1={26} x2={18} y2={26} stroke="#f59e0b" strokeWidth={2} />
        <circle cx={9} cy={26} r={2.5} fill="#f59e0b" />
        <text x={24} y={29} fontSize={10} fill="#57534e">
          Werkelijke dampdruk (p)
        </text>
      </g>
    </svg>
  );
}
