/**
 * Gestapelde staafgrafiek — warmteverliezen per vertrek.
 *
 * Pure SVG, geen externe dependencies.
 */

import type { RoomResult } from "../../types";
import { STACKED_BAR_COLORS, CHART_GRID_COLORS } from "../../lib/chartColors";

const CATEGORIES = [
  { key: "transmission", label: "Transmissie", color: STACKED_BAR_COLORS.transmission },
  { key: "ventilation", label: "Ventilatie", color: STACKED_BAR_COLORS.ventilation },
  { key: "infiltration", label: "Infiltratie", color: STACKED_BAR_COLORS.infiltration },
  { key: "heating_up", label: "Opwarmtoeslag", color: STACKED_BAR_COLORS.heatingUp },
  { key: "system", label: "Systeemverliezen", color: STACKED_BAR_COLORS.system },
] as const;

type CategoryKey = (typeof CATEGORIES)[number]["key"];

function getValue(room: RoomResult, key: CategoryKey): number {
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

interface StackedBarChartProps {
  rooms: RoomResult[];
}

export function StackedBarChart({ rooms }: StackedBarChartProps) {
  if (rooms.length === 0) return null;

  const CHART_LEFT = 40;
  const CHART_RIGHT = 16;
  const CHART_TOP = 12;
  const CHART_BOTTOM = 60;
  const BAR_GAP = 8;
  const LEGEND_HEIGHT = 24;

  const chartWidth = Math.max(400, rooms.length * 60 + CHART_LEFT + CHART_RIGHT);
  const chartHeight = 260;
  const plotWidth = chartWidth - CHART_LEFT - CHART_RIGHT;
  const plotHeight = chartHeight - CHART_TOP - CHART_BOTTOM - LEGEND_HEIGHT;

  // Calculate totals per room
  const totals = rooms.map((room) =>
    CATEGORIES.reduce((sum, cat) => sum + getValue(room, cat.key), 0),
  );
  const maxTotal = Math.max(...totals, 1);

  // Nice Y-axis max
  const yMax = niceMax(maxTotal);
  const yTicks = generateTicks(yMax);

  const barWidth = Math.min(40, (plotWidth - BAR_GAP * (rooms.length + 1)) / rooms.length);
  const totalBarsWidth = rooms.length * barWidth + (rooms.length - 1) * BAR_GAP;
  const offsetX = CHART_LEFT + (plotWidth - totalBarsWidth) / 2;

  const yScale = (v: number) => CHART_TOP + plotHeight - (v / yMax) * plotHeight;

  return (
    <svg
      viewBox={`0 0 ${chartWidth} ${chartHeight}`}
      className="w-full"
      role="img"
      aria-label="Warmteverliezen per vertrek"
    >
      {/* Y-axis gridlines + labels */}
      {yTicks.map((tick) => (
        <g key={tick}>
          <line
            x1={CHART_LEFT}
            x2={chartWidth - CHART_RIGHT}
            y1={yScale(tick)}
            y2={yScale(tick)}
            stroke={CHART_GRID_COLORS.grid}
            strokeDasharray="3,3"
          />
          <text
            x={CHART_LEFT - 4}
            y={yScale(tick)}
            textAnchor="end"
            dominantBaseline="middle"
            className="fill-stone-400"
            fontSize="9"
          >
            {tick >= 1000 ? `${(tick / 1000).toFixed(1)}k` : String(tick)}
          </text>
        </g>
      ))}

      {/* Baseline */}
      <line
        x1={CHART_LEFT}
        x2={chartWidth - CHART_RIGHT}
        y1={yScale(0)}
        y2={yScale(0)}
        stroke={CHART_GRID_COLORS.gridStrong}
      />

      {/* Bars */}
      {rooms.map((room, i) => {
        const x = offsetX + i * (barWidth + BAR_GAP);
        let stackY = yScale(0);

        return (
          <g key={room.room_id}>
            {CATEGORIES.map((cat) => {
              const val = getValue(room, cat.key);
              if (val <= 0) return null;
              const barH = (val / yMax) * plotHeight;
              const y = stackY - barH;
              stackY = y;
              return (
                <rect
                  key={cat.key}
                  x={x}
                  y={y}
                  width={barWidth}
                  height={barH}
                  fill={cat.color}
                  rx={1}
                >
                  <title>
                    {room.room_name} — {cat.label}: {Math.round(val)} W
                  </title>
                </rect>
              );
            })}
            {/* Room label */}
            <text
              x={x + barWidth / 2}
              y={yScale(0) + 6}
              textAnchor="end"
              className="fill-stone-600"
              fontSize="9"
              transform={`rotate(-35, ${x + barWidth / 2}, ${yScale(0) + 6})`}
            >
              {room.room_name.length > 12
                ? room.room_name.slice(0, 11) + "\u2026"
                : room.room_name}
            </text>
          </g>
        );
      })}

      {/* Legend */}
      <g transform={`translate(${CHART_LEFT}, ${chartHeight - LEGEND_HEIGHT + 4})`}>
        {CATEGORIES.map((cat, i) => (
          <g key={cat.key} transform={`translate(${i * 90}, 0)`}>
            <rect width={10} height={10} fill={cat.color} rx={2} />
            <text
              x={14}
              y={5}
              dominantBaseline="middle"
              className="fill-stone-500"
              fontSize="9"
            >
              {cat.label}
            </text>
          </g>
        ))}
      </g>
    </svg>
  );
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
