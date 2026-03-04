import { BOUNDARY_COLORS, BOUNDARY_TYPE_LABELS } from "../../lib/constants";
import type { BoundaryType } from "../../types";

const COLOR_CLASSES: Record<string, string> = {
  blue: "bg-blue-100 text-blue-800",
  purple: "bg-purple-100 text-purple-800",
  green: "bg-green-100 text-green-800",
  amber: "bg-amber-100 text-amber-800",
  stone: "bg-stone-200 text-stone-700",
};

interface BoundaryBadgeProps {
  type: BoundaryType;
}

export function BoundaryBadge({ type }: BoundaryBadgeProps) {
  const color = BOUNDARY_COLORS[type] ?? "stone";
  const classes = COLOR_CLASSES[color] ?? COLOR_CLASSES.stone;

  return (
    <span
      className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${classes}`}
    >
      {BOUNDARY_TYPE_LABELS[type] ?? type}
    </span>
  );
}
