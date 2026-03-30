import { BOUNDARY_COLORS, BOUNDARY_TYPE_LABELS } from "../../lib/constants";
import type { BoundaryType } from "../../types";

const COLOR_CLASSES: Record<string, string> = {
  blue: "bg-blue-600/15 text-blue-400",
  purple: "bg-purple-600/15 text-purple-400",
  green: "bg-green-600/15 text-green-400",
  amber: "bg-amber-600/15 text-amber-400",
  stone: "bg-[var(--oaec-hover)] text-on-surface-muted",
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
