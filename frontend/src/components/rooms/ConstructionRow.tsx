import { memo, useCallback } from "react";

import { BOUNDARY_TYPE_LABELS, VERTICAL_POSITION_LABELS } from "../../lib/constants";
import type { BoundaryType, ConstructionElement, VerticalPosition } from "../../types";
import { BoundaryBadge } from "./BoundaryBadge";
import { EditableCell } from "./EditableCell";
import { EditableSelect } from "./EditableSelect";

interface ConstructionCellsProps {
  construction: ConstructionElement;
  onUpdate: (partial: Partial<ConstructionElement>) => void;
  onRemove: () => void;
}

/**
 * Renders construction-level cells (description, boundary type, area, U, position, delete).
 * Returns cell fragments — the parent <tr> composes the full row.
 */
export const ConstructionCells = memo(function ConstructionCells({
  construction,
  onUpdate,
  onRemove,
}: ConstructionCellsProps) {
  const handleArea = useCallback(
    (v: string) => onUpdate({ area: Number(v) || 0 }),
    [onUpdate],
  );
  const handleUValue = useCallback(
    (v: string) => onUpdate({ u_value: Number(v) || 0 }),
    [onUpdate],
  );

  return (
    <>
      <td className="px-2 py-1">
        <EditableCell
          value={construction.description}
          onChange={(v) => onUpdate({ description: v })}
          placeholder="Beschrijving..."
        />
      </td>
      <td className="px-2 py-1">
        <div className="flex items-center gap-1.5">
          <EditableSelect
            value={construction.boundary_type}
            onChange={(v) => onUpdate({ boundary_type: v as BoundaryType })}
            options={BOUNDARY_TYPE_LABELS}
          />
          <BoundaryBadge type={construction.boundary_type} />
        </div>
      </td>
      <td className="px-2 py-1 text-right">
        <EditableCell
          value={construction.area}
          onChange={handleArea}
          type="number"
          unit="m\u00B2"
        />
      </td>
      <td className="px-2 py-1 text-right">
        <EditableCell
          value={construction.u_value}
          onChange={handleUValue}
          type="number"
          unit="W/m\u00B2K"
        />
      </td>
      <td className="px-2 py-1">
        <EditableSelect
          value={construction.vertical_position ?? "wall"}
          onChange={(v) => onUpdate({ vertical_position: v as VerticalPosition })}
          options={VERTICAL_POSITION_LABELS}
        />
      </td>
      <td className="px-1 py-1 text-center">
        <button
          onClick={onRemove}
          className="rounded p-0.5 text-stone-400 hover:bg-red-50 hover:text-red-600"
          title="Verwijder grensvlak"
        >
          <svg className="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
            <path
              fillRule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </td>
    </>
  );
});
