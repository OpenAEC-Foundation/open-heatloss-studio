/**
 * Horizontale bar chart — transmissieverlies per constructietype.
 *
 * Berekent per grensvlak: Phi_T = U * A * dT en groepeert per categorie.
 * Pure SVG, geen externe dependencies.
 */

import { useMemo } from "react";

import type { Room, BoundaryType } from "../../types";
import { CONSTRUCTION_CATEGORY_COLORS } from "../../lib/chartColors";
import { DEFAULT_THETA_WATER } from "../../lib/constants";
import { buildRoomLookup, computeDeltaT } from "./deltaT";

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
    color: CONSTRUCTION_CATEGORY_COLORS.walls,
    matchFn: (ce) =>
      ce.boundary_type === "exterior" &&
      (ce.vertical_position === "wall" || !ce.vertical_position) &&
      !isGlazing(ce.description),
  },
  {
    label: "Beglazing / kozijnen",
    color: CONSTRUCTION_CATEGORY_COLORS.glazing,
    matchFn: (ce) =>
      ce.boundary_type === "exterior" && isGlazing(ce.description),
  },
  {
    label: "Daken / plafonds",
    color: CONSTRUCTION_CATEGORY_COLORS.roofs,
    matchFn: (ce) =>
      ce.boundary_type === "exterior" && ce.vertical_position === "ceiling",
  },
  {
    label: "Vloeren / grond",
    color: CONSTRUCTION_CATEGORY_COLORS.floors,
    matchFn: (ce) =>
      ce.boundary_type === "ground" || ce.vertical_position === "floor",
  },
  {
    label: "Binnenwanden / buren",
    color: CONSTRUCTION_CATEGORY_COLORS.internalWalls,
    matchFn: (ce) =>
      ce.boundary_type === "adjacent_room" ||
      ce.boundary_type === "adjacent_building" ||
      ce.boundary_type === "unheated_space",
  },
  {
    label: "Grensvlak water",
    color: CONSTRUCTION_CATEGORY_COLORS.floors,
    matchFn: (ce) => ce.boundary_type === "water",
  },
];

const FALLBACK_LABEL = "Overig";
const FALLBACK_COLOR = CONSTRUCTION_CATEGORY_COLORS.other;

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

// ---------------------------------------------------------------------------
// Props & component
// ---------------------------------------------------------------------------

interface ConstructionLossChartProps {
  rooms: Room[];
  thetaE: number;
  /** Ontwerp-watertemperatuur (°C). Valt terug op DEFAULT_THETA_WATER. */
  thetaWater?: number;
}

interface BarData {
  label: string;
  color: string;
  value: number;
  area: number;
  uAvg: number;
}

export function ConstructionLossChart({
  rooms,
  thetaE,
  thetaWater,
}: ConstructionLossChartProps) {
  const bars = useMemo(() => {
    const totals = new Map<
      string,
      { color: string; value: number; area: number; uTimesA: number }
    >();
    const thetaW = thetaWater ?? DEFAULT_THETA_WATER;
    const roomLookup = buildRoomLookup(rooms);

    for (const room of rooms) {
      const thetaI = room.custom_temperature ?? defaultTemperature(room.function);

      for (const ce of room.constructions) {
        const dT = computeDeltaT(ce.boundary_type, thetaI, thetaE, ce, {
          rooms: roomLookup,
          thetaWater: thetaW,
        });
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
          existing.area += ce.area;
          existing.uTimesA += ce.u_value * ce.area;
        } else {
          totals.set(label, {
            color,
            value: phiT,
            area: ce.area,
            uTimesA: ce.u_value * ce.area,
          });
        }
      }
    }

    const result: BarData[] = [];
    for (const [label, data] of totals) {
      const uAvg = data.area > 0 ? data.uTimesA / data.area : 0;
      result.push({
        label,
        color: data.color,
        value: data.value,
        area: data.area,
        uAvg,
      });
    }
    result.sort((a, b) => b.value - a.value);
    return result;
  }, [rooms, thetaE, thetaWater]);

  if (bars.length === 0) return null;

  // Layout — VALUE_WIDTH verbreed naar 170 om "1234 W · 78.5 m² · U 0.35"
  // rechts naast de bar te tonen. m² in muted tint, U_gem nog zachter zodat
  // W dominant blijft (kleuren-volgorde: primary → muted → extra-muted).
  const LABEL_WIDTH = 140;
  const BAR_AREA_WIDTH = 340;
  const VALUE_WIDTH = 170;
  const CHART_WIDTH = LABEL_WIDTH + BAR_AREA_WIDTH + VALUE_WIDTH;
  const BAR_HEIGHT = 18;
  const BAR_GAP = 5;
  const PADDING_TOP = 6;
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
              className="fill-on-surface-secondary"
              fontSize="10"
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
                {bar.label}: {Math.round(bar.value)} W · {bar.area.toFixed(1)} m²
                {bar.area > 0
                  ? ` · U_gem = ${bar.uAvg.toFixed(2)} W/(m²·K)`
                  : " · U_gem = —"}
              </title>
            </rect>
            {/* Value — W dominant, m² muted, U_gem nog zachter (opacity) */}
            <text
              x={LABEL_WIDTH + BAR_AREA_WIDTH + 8}
              y={y + BAR_HEIGHT / 2}
              dominantBaseline="middle"
              fontSize="10"
            >
              <tspan className="fill-on-surface" fontWeight="500">
                {Math.round(bar.value)} W
              </tspan>
              <tspan className="fill-on-surface-muted" dx="6">
                · {bar.area.toFixed(1)} m²
              </tspan>
              <tspan className="fill-on-surface-muted" dx="6" opacity="0.65">
                · U {bar.area > 0 ? bar.uAvg.toFixed(2) : "—"}
              </tspan>
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

/**
 * Default interne temperaturen per room-function voor chart-weergave.
 * Wijken (bewust) licht af van `ROOM_FUNCTION_TEMPERATURES`: deze tabel
 * is historisch gebonden aan de visualisatie en wordt alleen gebruikt
 * voor `thetaI` van de "self"-ruimte in de chart.
 */
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
