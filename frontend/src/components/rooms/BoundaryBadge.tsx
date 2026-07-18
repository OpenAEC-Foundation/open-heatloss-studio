import { BOUNDARY_COLORS, BOUNDARY_TYPE_LABELS } from "../../lib/constants";
import type { BoundaryType } from "../../types";

const COLOR_CLASSES: Record<string, string> = {
  blue: "bg-blue-500",
  purple: "bg-purple-500",
  green: "bg-green-500",
  amber: "bg-[var(--domain-boundary-adjacent-building)]",
  stone: "bg-stone-400",
  teal: "bg-teal-500",
};

interface BoundaryBadgeProps {
  type: BoundaryType;
}

export function BoundaryBadge({ type }: BoundaryBadgeProps) {
  const color = BOUNDARY_COLORS[type] ?? "stone";
  const classes = COLOR_CLASSES[color] ?? COLOR_CLASSES.stone;

  return (
    <span
      className={`inline-block h-2.5 w-2.5 shrink-0 rounded-full ${classes}`}
      title={BOUNDARY_TYPE_LABELS[type] ?? type}
    />
  );
}
