import { memo, useCallback, useState, useMemo } from "react";

import { BOUNDARY_TYPE_LABELS, VERTICAL_POSITION_LABELS } from "../../lib/constants";
import { calculateRc } from "../../lib/rcCalculation";
import type {
  BoundaryType,
  ConstructionElement,
  ConstructionElementLayer,
  VerticalPosition,
} from "../../types";
import { useModellerStore } from "../modeller/modellerStore";
import type { ProjectConstruction } from "../modeller/types";
import { LayerEditor } from "../construction/LayerEditor";
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
  const [layerEditorOpen, setLayerEditorOpen] = useState(false);

  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );

  const handleArea = useCallback(
    (v: string) => onUpdate({ area: Number(v) || 0 }),
    [onUpdate],
  );
  const handleUValue = useCallback(
    (v: string) => onUpdate({ u_value: Number(v) || 0 }),
    [onUpdate],
  );

  const handleApplyLayers = useCallback(
    (layers: ConstructionElementLayer[], uValue: number) => {
      onUpdate({
        layers: layers.length > 0 ? layers : undefined,
        u_value: Math.round(uValue * 1000) / 1000,
      });
      setLayerEditorOpen(false);
    },
    [onUpdate],
  );

  const handleSelectConstruction = useCallback(
    (pcId: string) => {
      if (pcId === "__manual__") {
        // Ontkoppel — houd huidige waarden
        onUpdate({ project_construction_id: undefined });
        return;
      }
      const pc = projectConstructions.find(
        (c: ProjectConstruction) => c.id === pcId,
      );
      if (!pc) return;
      const rcResult =
        pc.layers.length > 0
          ? calculateRc(pc.layers, pc.verticalPosition)
          : null;
      onUpdate({
        description: pc.name,
        u_value: rcResult
          ? Math.round(rcResult.uValue * 1000) / 1000
          : construction.u_value,
        material_type: pc.materialType,
        vertical_position: pc.verticalPosition,
        layers: pc.layers.map((l) => ({ ...l })),
        project_construction_id: pc.id,
      });
    },
    [projectConstructions, onUpdate, construction.u_value],
  );

  const layerCount = construction.layers?.length ?? 0;
  const isLinked = !!construction.project_construction_id;

  // Build dropdown options
  const dropdownValue = construction.project_construction_id ?? "__manual__";

  // Group project constructions by category for the dropdown
  const sortedConstructions = useMemo(
    () =>
      [...projectConstructions].sort((a, b) => a.name.localeCompare(b.name)),
    [projectConstructions],
  );

  return (
    <>
      <td className="px-2 py-1">
        <div className="flex items-center gap-1">
          <select
            value={dropdownValue}
            onChange={(e) => handleSelectConstruction(e.target.value)}
            className={`min-w-0 flex-1 truncate rounded border px-1.5 py-0.5 text-xs ${
              isLinked
                ? "border-blue-500/30 bg-blue-600/15 text-blue-400"
                : "border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface-secondary"
            }`}
            title={construction.description || "Kies constructie..."}
          >
            <option value="__manual__">
              {construction.description || "Handmatig..."}
            </option>
            {sortedConstructions.map((pc) => (
              <option key={pc.id} value={pc.id}>
                {pc.name}
              </option>
            ))}
          </select>
          {!isLinked && (
            <EditableCell
              value={construction.description}
              onChange={(v) => onUpdate({ description: v })}
              placeholder="Beschrijving..."
            />
          )}
        </div>
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
          unit="m²"
        />
      </td>
      <td className="px-2 py-1 text-right">
        <div className="flex items-center justify-end gap-1">
          <EditableCell
            value={construction.u_value}
            onChange={handleUValue}
            type="number"
            unit="W/m²K"
          />
          <button
            onClick={() => setLayerEditorOpen(true)}
            className={`shrink-0 rounded px-1.5 py-0.5 text-xs ${
              layerCount > 0
                ? "bg-blue-600/15 text-blue-400 hover:bg-blue-600/25"
                : "text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface"
            }`}
            title="Constructie-opbouw bewerken"
          >
            {layerCount > 0 ? `${layerCount} lagen` : "Lagen"}
          </button>
        </div>
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
          className="rounded p-0.5 text-on-surface-muted hover:bg-red-600/15 hover:text-red-400"
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

      {layerEditorOpen && (
        <LayerEditor
          layers={construction.layers ?? []}
          position={construction.vertical_position ?? "wall"}
          onApply={handleApplyLayers}
          onClose={() => setLayerEditorOpen(false)}
        />
      )}
    </>
  );
});
