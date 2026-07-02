/**
 * Horizontale bar chart — transmissieverlies per constructietype.
 *
 * Berekent per grensvlak: Phi_T = U * A * dT en groepeert per categorie.
 * Pure SVG, geen externe dependencies.
 */

import { useMemo } from "react";

import type { Room, BoundaryType } from "../../types";
import type {
  ActiveNorm,
  Isso53RoomState,
} from "../../types/projectV2";
import { CONSTRUCTION_CATEGORY_COLORS } from "../../lib/chartColors";
import { DEFAULT_THETA_WATER, ROOM_FUNCTION_TEMPERATURES } from "../../lib/constants";
import {
  TEMPERATURE_IS_EXTERIOR,
  design_indoor_temperature,
} from "../../lib/isso53Temperature";
import { resolveUnheatedRoomIds } from "../../lib/isso53Unheated";
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
      ce.boundary_type === "adjacent_building",
  },
  {
    label: "Onverwarmd",
    color: CONSTRUCTION_CATEGORY_COLORS.unheated,
    matchFn: (ce) => ce.boundary_type === "unheated_space",
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
  /**
   * Actieve norm. Bij `"isso53"` worden self- en buurruimte-temperaturen
   * uit de ISSO 53-sidecar (`ruimteType`) afgeleid i.p.v. de ISSO 51
   * `room.function`-tabel. Default/undefined → ISSO 51-gedrag (ongewijzigd).
   */
  norm?: ActiveNorm;
  /** ISSO 53 per-vertrek sidecar-state (key = room.id). Alleen relevant bij ISSO 53. */
  isso53Rooms?: Record<string, Isso53RoomState>;
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
  norm,
  isso53Rooms,
}: ConstructionLossChartProps) {
  const bars = useMemo(() => {
    const totals = new Map<
      string,
      { color: string; value: number; area: number; uTimesA: number }
    >();
    const thetaW = thetaWater ?? DEFAULT_THETA_WATER;
    const roomLookup = buildRoomLookup(rooms);
    const isIsso53 = norm === "isso53";

    // ISSO 53: gecombineerde set onverwarmde room-ids (impliciete
    // unheated_space-doelen ∪ expliciet via sidecar `isUnheated` gemarkeerd).
    // Een adjacent_room-wand náár een room in deze set wordt — net als in de
    // mapper — als onverwarmd grensvlak behandeld (categorie + f_k-ΔT). In
    // ISSO 51-modus blijft de set leeg → geen gedragswijziging.
    const unheatedRoomIds = isIsso53
      ? resolveUnheatedRoomIds(rooms, isso53Rooms ?? {})
      : new Set<string>();

    // Norm-aware temperatuur-resolver — alleen actief in ISSO 53-modus.
    // Leidt de design-θ van een ruimte af uit de sidecar-`ruimteType`
    // (tabel 2.2) i.p.v. de ISSO 51 room.function-tabel. `garage` → θ_e.
    // Retourneert null als er geen sidecar is → caller valt terug op ISSO 51.
    const resolveRoomTemperature = isIsso53
      ? (r: Room): number | null => {
          const sidecar = isso53Rooms?.[r.id];
          if (!sidecar) return null;
          const t = design_indoor_temperature(
            sidecar.gebruiksFunctie,
            sidecar.ruimteType,
          );
          return t === TEMPERATURE_IS_EXTERIOR ? thetaE : t;
        }
      : undefined;

    // ISSO 53: per-onverwarmde-doelruimte f_k uit de sidecar. `null` →
    // computeDeltaT valt terug op de norm-default 0,5. Niet gezet in ISSO
    // 51-modus → unheated-pad gebruikt enkel `ce.temperature_factor` ?? 0,5.
    const resolveUnheatedFactor = isIsso53
      ? (adjacentRoomId: string | null | undefined): number | null =>
          (adjacentRoomId != null
            ? isso53Rooms?.[adjacentRoomId]?.unheatedFactor
            : undefined) ?? null
      : undefined;

    for (const room of rooms) {
      // Self-θ: ISSO 53 → sidecar-ruimteType; anders ISSO 51 room.function.
      let thetaI: number;
      if (room.custom_temperature != null) {
        thetaI = room.custom_temperature;
      } else {
        const resolved = resolveRoomTemperature?.(room);
        thetaI = resolved ?? defaultTemperature(room.function);
      }

      for (const ce of room.constructions) {
        // Effectief grensvlaktype: een adjacent_room náár een onverwarmde
        // ruimte gedraagt zich als unheated_space (f_k-ΔT + categorie
        // "Onverwarmd"), consistent met de mapper. Andere types ongewijzigd.
        const effectiveBoundary: BoundaryType =
          ce.boundary_type === "adjacent_room" &&
          ce.adjacent_room_id != null &&
          unheatedRoomIds.has(ce.adjacent_room_id)
            ? "unheated_space"
            : ce.boundary_type;

        const dT = computeDeltaT(effectiveBoundary, thetaI, thetaE, ce, {
          rooms: roomLookup,
          thetaWater: thetaW,
          resolveRoomTemperature,
          resolveUnheatedFactor,
        });
        const phiT = ce.u_value * ce.area * dT;
        if (phiT <= 0) continue;

        const matched = CATEGORIES.find((cat) =>
          cat.matchFn({
            boundary_type: effectiveBoundary,
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
  }, [rooms, thetaE, thetaWater, norm, isso53Rooms]);

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
 * Default interne temperatuur (θ_i) van de "self"-ruimte voor chart-weergave.
 * Loopt via de single source of truth `ROOM_FUNCTION_TEMPERATURES`
 * (ISSO 51:2023 Tabel 2.11), zodat de chart dezelfde θ_i toont als de
 * berekening. Eerder stond hier een losse, afwijkende kopie — dat gaf een
 * andere ΔT in de grafiek dan in de rekenkern.
 */
function defaultTemperature(fn: string): number {
  return ROOM_FUNCTION_TEMPERATURES[fn] ?? 20;
}
