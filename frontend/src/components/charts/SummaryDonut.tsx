/**
 * Donut-chart — gebouwtotaal warmteverliezen.
 *
 * Pure SVG, geen externe dependencies.
 */

import type { BuildingSummary } from "../../types";
import { LOSS_TYPE_COLORS } from "../../lib/chartColors";

const SEGMENTS = [
  { key: "total_envelope_loss", label: "Transmissie", color: LOSS_TYPE_COLORS.transmission },
  { key: "total_ventilation_loss", label: "Ventilatie", color: LOSS_TYPE_COLORS.ventilation },
  { key: "total_heating_up", label: "Opwarmtoeslag", color: LOSS_TYPE_COLORS.heatingUp },
  { key: "total_system_losses", label: "Systeemverliezen", color: LOSS_TYPE_COLORS.system },
  { key: "total_neighbor_loss", label: "Buurwoningverlies", color: LOSS_TYPE_COLORS.neighbor },
] as const;

type SegmentKey = (typeof SEGMENTS)[number]["key"];

interface SummaryDonutProps {
  summary: BuildingSummary;
}

export function SummaryDonut({ summary }: SummaryDonutProps) {
  const SIZE = 200;
  const CENTER = SIZE / 2;
  const OUTER_R = 80;
  const INNER_R = 52;

  const values = SEGMENTS.map((s) => ({
    ...s,
    value: Math.max(0, summary[s.key as SegmentKey] as number),
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
      const midAngle = startAngle + angle / 2;
      startAngle = endAngle;
      return { ...v, path, fraction, midAngle };
    });

  return (
    <div className="flex items-center gap-6">
      <svg
        viewBox={`0 0 ${SIZE} ${SIZE}`}
        className="h-48 w-48 shrink-0"
        role="img"
        aria-label={`Totaal aansluitvermogen: ${Math.round(summary.connection_capacity)} W`}
      >
        {arcs.map((arc) => (
          <path key={arc.key} d={arc.path} fill={arc.color}>
            <title>
              {arc.label}: {Math.round(arc.value)} W ({(arc.fraction * 100).toFixed(1)}%)
            </title>
          </path>
        ))}
        {/* Center text */}
        <text
          x={CENTER}
          y={CENTER - 6}
          textAnchor="middle"
          dominantBaseline="middle"
          className="fill-on-surface"
          fontSize="16"
          fontWeight="bold"
        >
          {formatWatts(summary.connection_capacity)}
        </text>
        <text
          x={CENTER}
          y={CENTER + 12}
          textAnchor="middle"
          dominantBaseline="middle"
          className="fill-on-surface-muted"
          fontSize="10"
        >
          aansluitvermogen
        </text>
      </svg>

      {/* Legend */}
      <div className="space-y-1.5">
        {arcs.map((arc) => (
          <div key={arc.key} className="flex items-center gap-2 text-sm">
            <span
              className="inline-block h-3 w-3 shrink-0 rounded"
              style={{ backgroundColor: arc.color }}
            />
            <span className="text-on-surface-secondary">{arc.label}</span>
            <span className="ml-auto tabular-nums text-on-surface">
              {Math.round(arc.value)} W
            </span>
            <span className="w-12 text-right tabular-nums text-on-surface-muted">
              {(arc.fraction * 100).toFixed(0)}%
            </span>
          </div>
        ))}
      </div>
    </div>
  );
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
  const GAP = 0.02; // Small gap between segments
  const sA = startAngle + GAP;
  const eA = endAngle - GAP;

  if (eA <= sA) return "";

  const largeArc = eA - sA > Math.PI ? 1 : 0;

  const outerStart = polarToCart(cx, cy, outerR, sA);
  const outerEnd = polarToCart(cx, cy, outerR, eA);
  const innerStart = polarToCart(cx, cy, innerR, eA);
  const innerEnd = polarToCart(cx, cy, innerR, sA);

  return [
    `M ${outerStart.x} ${outerStart.y}`,
    `A ${outerR} ${outerR} 0 ${largeArc} 1 ${outerEnd.x} ${outerEnd.y}`,
    `L ${innerStart.x} ${innerStart.y}`,
    `A ${innerR} ${innerR} 0 ${largeArc} 0 ${innerEnd.x} ${innerEnd.y}`,
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
