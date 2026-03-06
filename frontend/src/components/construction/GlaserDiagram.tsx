/**
 * SVG-diagram voor de Glaser-methode (dampspanningsverloop).
 *
 * Toont verzadigingsdampdruk (pSat) en werkelijke dampdruk (pActual)
 * door de constructie-opbouw heen. Materiaallagen worden getoond met
 * categorie-specifieke kleuren en arceringen.
 */

import { useMemo } from "react";

import type { GlaserResult } from "../../lib/glaserCalculation";
import {
  MATERIAL_CATEGORY_VISUALS,
  type MaterialCategory,
} from "../../lib/materialsDatabase";

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

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen - 1) + "\u2026";
}

function categoryColor(cat: MaterialCategory): string {
  return MATERIAL_CATEGORY_VISUALS[cat]?.color ?? "#e5e7eb";
}

function categoryPattern(cat: MaterialCategory): string | undefined {
  return MATERIAL_CATEGORY_VISUALS[cat]?.patternId;
}

// ---------- SVG Hatching Patterns ----------

function HatchPatterns() {
  return (
    <defs>
      {/* Metselwerk: diagonaal kruisarcering */}
      <pattern
        id="hatch-masonry"
        width="8"
        height="8"
        patternUnits="userSpaceOnUse"
        patternTransform="rotate(45)"
      >
        <line x1="0" y1="0" x2="0" y2="8" stroke="rgba(0,0,0,0.18)" strokeWidth="1" />
        <line x1="4" y1="0" x2="4" y2="8" stroke="rgba(0,0,0,0.08)" strokeWidth="0.5" />
      </pattern>

      {/* Beton: stippen */}
      <pattern
        id="hatch-concrete"
        width="6"
        height="6"
        patternUnits="userSpaceOnUse"
      >
        <circle cx="1.5" cy="1.5" r="0.7" fill="rgba(0,0,0,0.15)" />
        <circle cx="4.5" cy="4.5" r="0.7" fill="rgba(0,0,0,0.15)" />
      </pattern>

      {/* Isolatie: zigzag */}
      <pattern
        id="hatch-insulation"
        width="10"
        height="8"
        patternUnits="userSpaceOnUse"
      >
        <polyline
          points="0,6 2.5,2 5,6 7.5,2 10,6"
          fill="none"
          stroke="rgba(0,0,0,0.15)"
          strokeWidth="0.8"
        />
      </pattern>

      {/* Hout: horizontale nerf */}
      <pattern
        id="hatch-wood"
        width="12"
        height="6"
        patternUnits="userSpaceOnUse"
      >
        <line x1="0" y1="2" x2="12" y2="2" stroke="rgba(0,0,0,0.12)" strokeWidth="0.6" />
        <line x1="0" y1="5" x2="12" y2="5" stroke="rgba(0,0,0,0.08)" strokeWidth="0.4" />
      </pattern>

      {/* Folie: dichte horizontale strepen */}
      <pattern
        id="hatch-foil"
        width="4"
        height="3"
        patternUnits="userSpaceOnUse"
      >
        <line x1="0" y1="1.5" x2="4" y2="1.5" stroke="rgba(0,0,0,0.2)" strokeWidth="1" />
      </pattern>

      {/* Metaal: dichte diagonaal */}
      <pattern
        id="hatch-metal"
        width="4"
        height="4"
        patternUnits="userSpaceOnUse"
        patternTransform="rotate(45)"
      >
        <line x1="0" y1="0" x2="0" y2="4" stroke="rgba(0,0,0,0.2)" strokeWidth="1" />
      </pattern>
    </defs>
  );
}

// ---------- Component ----------

export function GlaserDiagram({ result, thetaI, thetaE }: GlaserDiagramProps) {
  const {
    curvePoints,
    interfacePoints,
    layerThicknesses,
    layerNames,
    layerCategories,
    totalThickness,
  } = result;

  const hasLayers = curvePoints.length >= 2 && totalThickness > 0;

  // Schalen berekenen
  const { yTicks, toX, toY } = useMemo(() => {
    if (!hasLayers) {
      return {
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
      yTicks: ticks,
      toX: (xMm: number) => MARGIN.left + (xMm / totalThickness) * PLOT_W,
      toY: (pPa: number) => MARGIN.top + PLOT_H - (pPa / nMax) * PLOT_H,
    };
  }, [curvePoints, interfacePoints, totalThickness, hasLayers]);

  // pSat-curve pad
  const pSatPath = useMemo(() => {
    if (!hasLayers) return "";
    return curvePoints
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pSat).toFixed(1)}`,
      )
      .join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // pActual-lijn pad
  const pActualPath = useMemo(() => {
    if (!hasLayers) return "";
    return interfacePoints
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"}${toX(p.x).toFixed(1)},${toY(p.pActual).toFixed(1)}`,
      )
      .join(" ");
  }, [interfacePoints, hasLayers, toX, toY]);

  // Condensatiezone: gebied waar pActual > pSat
  const condensationPath = useMemo(() => {
    if (!hasLayers) return "";

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
          .map(
            (bp) =>
              `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`,
          )
          .join(" ");
        zones.push(`${zoneStart} ${backPath} Z`);
        inZone = false;
      }
    }

    if (inZone) {
      const lastInZone = curvePoints.filter(
        (p) => p.pActual > p.pSat + 0.5,
      );
      const backPath = [...lastInZone]
        .reverse()
        .map(
          (bp) =>
            `L${toX(bp.x).toFixed(1)},${toY(bp.pSat).toFixed(1)}`,
        )
        .join(" ");
      zones.push(`${zoneStart} ${backPath} Z`);
    }

    return zones.join(" ");
  }, [curvePoints, hasLayers, toX, toY]);

  // Laag-banden met categorie-kleuren
  const layerBands = useMemo(() => {
    if (!hasLayers) return [];
    const bands: {
      x: number;
      w: number;
      name: string;
      color: string;
      pattern?: string;
    }[] = [];
    let xCum = 0;
    for (let i = 0; i < layerThicknesses.length; i++) {
      const d = layerThicknesses[i]!;
      const cat = layerCategories[i] as MaterialCategory | undefined;
      bands.push({
        x: toX(xCum),
        w: (d / totalThickness) * PLOT_W,
        name: layerNames[i] ?? "",
        color: cat ? categoryColor(cat) : "#e5e7eb",
        pattern: cat ? categoryPattern(cat) : undefined,
      });
      xCum += d;
    }
    return bands;
  }, [
    layerThicknesses,
    layerNames,
    layerCategories,
    totalThickness,
    hasLayers,
    toX,
  ]);

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
      <HatchPatterns />

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

      {/* Laag-banden met kleur en arcering */}
      {layerBands.map((band, i) => (
        <g key={i}>
          {/* Kleurvulling */}
          <rect
            x={band.x}
            y={MARGIN.top}
            width={Math.max(band.w, 1)}
            height={PLOT_H}
            fill={band.color}
            fillOpacity={0.55}
          />
          {/* Arcering overlay */}
          {band.pattern && (
            <rect
              x={band.x}
              y={MARGIN.top}
              width={Math.max(band.w, 1)}
              height={PLOT_H}
              fill={`url(#${band.pattern})`}
            />
          )}
          {/* Laag-scheidingslijn */}
          {i > 0 && (
            <line
              x1={band.x}
              y1={MARGIN.top}
              x2={band.x}
              y2={MARGIN.top + PLOT_H}
              stroke="#78716c"
              strokeWidth={0.5}
            />
          )}
          {/* Laagnaam */}
          {band.w > 14 && (
            <text
              x={band.x + band.w / 2}
              y={MARGIN.top + PLOT_H + 14}
              textAnchor="middle"
              fontSize={9}
              fill="#57534e"
              fontWeight={500}
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
                stroke="#d6d3d1"
                strokeWidth={0.5}
                strokeDasharray="3,3"
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
        <path d={condensationPath} fill="#fca5a5" fillOpacity={0.45} />
      )}

      {/* pSat curve (blauw) */}
      <path
        d={pSatPath}
        fill="none"
        stroke="#2563eb"
        strokeWidth={2.5}
        strokeLinejoin="round"
      />

      {/* pActual lijn (amber) */}
      <path
        d={pActualPath}
        fill="none"
        stroke="#d97706"
        strokeWidth={2.5}
        strokeLinejoin="round"
        strokeDasharray="6,3"
      />

      {/* Punten op interfaces */}
      {interfacePoints.map((p, i) => (
        <g key={i}>
          <circle cx={toX(p.x)} cy={toY(p.pSat)} r={3} fill="#2563eb" />
          <circle
            cx={toX(p.x)}
            cy={toY(p.pActual)}
            r={3}
            fill="#d97706"
          />
        </g>
      ))}

      {/* Temperatuurlabels bij interface-punten */}
      {interfacePoints.map((p, i) => {
        const x = toX(p.x);
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
      <g
        transform={`translate(${MARGIN.left + PLOT_W - 200}, ${MARGIN.top + 8})`}
      >
        <rect
          x={-6}
          y={-4}
          width={206}
          height={42}
          rx={4}
          fill="white"
          fillOpacity={0.92}
          stroke="#d6d3d1"
          strokeWidth={0.5}
        />
        <line
          x1={0}
          y1={8}
          x2={18}
          y2={8}
          stroke="#2563eb"
          strokeWidth={2.5}
        />
        <circle cx={9} cy={8} r={3} fill="#2563eb" />
        <text x={24} y={11} fontSize={10} fill="#57534e">
          Verzadigingsdruk (p_sat)
        </text>
        <line
          x1={0}
          y1={26}
          x2={18}
          y2={26}
          stroke="#d97706"
          strokeWidth={2.5}
          strokeDasharray="6,3"
        />
        <circle cx={9} cy={26} r={3} fill="#d97706" />
        <text x={24} y={29} fontSize={10} fill="#57534e">
          Werkelijke dampdruk (p)
        </text>
      </g>
    </svg>
  );
}
