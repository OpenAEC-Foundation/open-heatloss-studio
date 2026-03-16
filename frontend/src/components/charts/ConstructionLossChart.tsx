/**
 * Horizontale bar chart — transmissieverlies per constructietype.
 *
 * Berekent per grensvlak: Phi_T = U * A * dT en groepeert per categorie.
 * Pure SVG, geen externe dependencies.
 */

import { useMemo } from "react";

import type { Room, BoundaryType } from "../../types";

// ---------------------------------------------------------------------------
// Category grouping
// ---------------------------------------------------------------------------

interface CategoryGroup {
  label: string;
  color: string;
  matchFn: (ce: { boundary_type: BoundaryType; description: string; vertical_position?: string }) => boolean;
}

const CATEGORIES: CategoryGroup[] = [
  {
    label: "Buitenwanden",
    color: "#ef4444",
    matchFn: (ce) =>
      ce.boundary_type === "exterior" &&
      (ce.vertical_position === "wall" || !ce.vertical_position) &&
      !isGlazing(ce.description),
  },
  {
    label: "Beglazing / kozijnen",
    color: "#3b82f6",
    matchFn: (ce) =>
      ce.boundary_type === "exterior" && isGlazing(ce.description),
  },
  {
    label: "Daken / plafonds",
    color: "#f59e0b",
    matchFn: (ce) =>
      ce.boundary_type === "exterior" && ce.vertical_position === "ceiling",
  },
  {
    label: "Vloeren / grond",
    color: "#22c55e",
    matchFn: (ce) =>
      ce.boundary_type === "ground" || ce.vertical_position === "floor",
  },
  {
    label: "Binnenwanden / buren",
    color: "#8b5cf6",
    matchFn: (ce) =>
      ce.boundary_type === "adjacent_room" ||
      ce.boundary_type === "adjacent_building" ||
      ce.boundary_type === "unheated_space",
  },
];

const FALLBACK_LABEL = "Overig";
const FALLBACK_COLOR = "#78716c";

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
    d.includes("triple")
  );
}

// ---------------------------------------------------------------------------
// Props & component
// ---------------------------------------------------------------------------

interface ConstructionLossChartProps {
  rooms: Room[];
  thetaE: number;
}

interface BarData {
  label: string;
  color: string;
  value: number;
}

export function ConstructionLossChart({ rooms, thetaE }: ConstructionLossChartProps) {
  const bars = useMemo(() => {
    const totals = new Map<string, { color: string; value: number }>();

    for (const room of rooms) {
      const thetaI = room.custom_temperature ?? defaultTemperature(room.function);

      for (const ce of room.constructions) {
        const dT = computeDeltaT(ce.boundary_type, thetaI, thetaE, ce);
        const phiT = ce.u_value * ce.area * dT;
        if (phiT <= 0) continue;

        const matched = CATEGORIES.find((cat) =>
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

    const result: BarData[] = [];
    for (const [label, data] of totals) {
      result.push({ label, ...data });
    }
    result.sort((a, b) => b.value - a.value);
    return result;
  }, [rooms, thetaE]);

  if (bars.length === 0) return null;

  // Layout
  const LABEL_WIDTH = 160;
  const BAR_AREA_WIDTH = 340;
  const VALUE_WIDTH = 70;
  const CHART_WIDTH = LABEL_WIDTH + BAR_AREA_WIDTH + VALUE_WIDTH;
  const BAR_HEIGHT = 24;
  const BAR_GAP = 8;
  const PADDING_TOP = 8;
  const CHART_HEIGHT = PADDING_TOP + bars.length * (BAR_HEIGHT + BAR_GAP);

  const maxValue = Math.max(...bars.map((b) => b.value), 1);

  return (
    <svg
      viewBox={`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`}
      className="w-full"
      role="img"
      aria-label="Transmissieverlies per constructietype"
    >
      {bars.map((bar, i) => {
        const y = PADDING_TOP + i * (BAR_HEIGHT + BAR_GAP);
        const barW = (bar.value / maxValue) * BAR_AREA_WIDTH;

        return (
          <g key={bar.label}>
            {/* Label */}
            <text
              x={LABEL_WIDTH - 8}
              y={y + BAR_HEIGHT / 2}
              textAnchor="end"
              dominantBaseline="middle"
              className="fill-stone-600"
              fontSize="11"
            >
              {bar.label}
            </text>
            {/* Bar */}
            <rect
              x={LABEL_WIDTH}
              y={y}
              width={Math.max(barW, 2)}
              height={BAR_HEIGHT}
              fill={bar.color}
              rx={3}
            >
              <title>
                {bar.label}: {Math.round(bar.value)} W
              </title>
            </rect>
            {/* Value */}
            <text
              x={LABEL_WIDTH + BAR_AREA_WIDTH + 8}
              y={y + BAR_HEIGHT / 2}
              dominantBaseline="middle"
              className="fill-stone-700"
              fontSize="11"
              fontWeight="500"
            >
              {Math.round(bar.value)} W
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const DEFAULT_TEMPERATURES: Record<string, number> = {
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

function defaultTemperature(fn: string): number {
  return DEFAULT_TEMPERATURES[fn] ?? 20;
}

function computeDeltaT(
  boundaryType: BoundaryType,
  thetaI: number,
  thetaE: number,
  ce: { temperature_factor?: number | null; adjacent_temperature?: number | null },
): number {
  switch (boundaryType) {
    case "exterior":
      return thetaI - thetaE;
    case "ground":
      return thetaI - thetaE;
    case "unheated_space":
      if (ce.temperature_factor != null) {
        return ce.temperature_factor * (thetaI - thetaE);
      }
      return thetaI - thetaE;
    case "adjacent_building":
      return thetaI - (ce.adjacent_temperature ?? thetaE);
    case "adjacent_room":
      return ce.adjacent_temperature != null
        ? thetaI - ce.adjacent_temperature
        : 0;
    default:
      return thetaI - thetaE;
  }
}
