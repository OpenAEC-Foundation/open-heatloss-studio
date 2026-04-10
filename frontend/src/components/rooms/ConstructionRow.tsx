import { memo, useCallback, useState, useMemo } from "react";

import {
  BOUNDARY_TYPE_LABELS,
  ROOM_FUNCTION_TEMPERATURES,
  VERTICAL_POSITION_LABELS,
} from "../../lib/constants";
import { formatArea } from "../../lib/formatNumber";
import {
  isFrameConstruction,
  isFrameOverrideActive,
} from "../../lib/frameOverride";
import { getMaterialById } from "../../lib/materialsDatabase";
import { calculateRc } from "../../lib/rcCalculation";
import { useProjectStore } from "../../store/projectStore";
import type {
  BoundaryType,
  ConstructionElement,
  ConstructionElementLayer,
  Room,
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
  /**
   * ID van de ruimte waar deze construction in hangt. Wordt gebruikt om
   * de ruimte zelf uit te sluiten bij het kiezen van een aangrenzende
   * ruimte (een ruimte kan niet aangrenzend zijn aan zichzelf).
   */
  ownerRoomId?: string;
}

/**
 * Onverwarmd-drempel in °C. Ruimten met een design-temperatuur kleiner dan
 * of gelijk aan deze drempel krijgen de "(onverwarmd)" suffix in het
 * adjacent-room label. Geen ISSO-norm, puur UX-indicatie voor gebruikers.
 */
const UNHEATED_TEMPERATURE_THRESHOLD = 15;

/**
 * Bepaalt de effectieve design-temperatuur voor een ruimte — gebruikt
 * voor het onverwarmd-label. Gededupeerd met `deltaT.ts` om te voorkomen
 * dat we hier een nieuwe dependency chain bouwen.
 */
function roomDesignTemperature(room: Room): number {
  if (room.custom_temperature != null) {
    return room.custom_temperature;
  }
  return ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;
}

/**
 * Renders construction-level cells (description, boundary type, area, U, position, delete).
 * Returns cell fragments — the parent <tr> composes the full row.
 */
export const ConstructionCells = memo(function ConstructionCells({
  construction,
  onUpdate,
  onRemove,
  ownerRoomId,
}: ConstructionCellsProps) {
  const [layerEditorOpen, setLayerEditorOpen] = useState(false);

  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );

  const projectRooms = useProjectStore((s) => s.project.rooms);
  const frameUValueOverride = useProjectStore(
    (s) => s.project.frameUValueOverride,
  );

  const frameOverrideActive = useMemo(
    () =>
      isFrameOverrideActive(frameUValueOverride) &&
      isFrameConstruction(construction, projectConstructions),
    [frameUValueOverride, construction, projectConstructions],
  );

  const adjacentRoomOptions = useMemo(
    () =>
      projectRooms
        .filter((r) => r.id !== ownerRoomId)
        .sort((a, b) => a.name.localeCompare(b.name)),
    [projectRooms, ownerRoomId],
  );

  const adjacentRoom = useMemo<Room | null>(() => {
    if (!construction.adjacent_room_id) return null;
    return projectRooms.find((r) => r.id === construction.adjacent_room_id) ?? null;
  }, [projectRooms, construction.adjacent_room_id]);

  const handleAdjacentRoomChange = useCallback(
    (id: string) => {
      onUpdate({ adjacent_room_id: id === "" ? null : id });
    },
    [onUpdate],
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
      // Prioriteit: (1) rcResult uit lagen, (2) directe pc.uValue voor
      // kozijnen/vullingen, (3) huidige waarde behouden als laatste redmiddel.
      const nextUValue = rcResult
        ? Math.round(rcResult.uValue * 1000) / 1000
        : pc.uValue ?? construction.u_value;
      onUpdate({
        description: pc.name,
        u_value: nextUValue,
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

  // Fallback labels per laag: wanneer een materialId geen database-match
  // oplevert (typisch bij Revit thermal-import layers met raw namen als
  // `i1_hout_bamboe`), tonen we de raw string zodat de gebruiker ziet wat
  // uit de import kwam. Zelfde patroon als in ConstructionImportStep.
  const layerDisplayOverrides = useMemo<(string | null)[]>(
    () =>
      (construction.layers ?? []).map((l) =>
        getMaterialById(l.materialId) ? null : l.materialId || null,
      ),
    [construction.layers],
  );

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
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-1.5">
            <EditableSelect
              value={construction.boundary_type}
              onChange={(v) => onUpdate({ boundary_type: v as BoundaryType })}
              options={BOUNDARY_TYPE_LABELS}
            />
            <BoundaryBadge type={construction.boundary_type} />
          </div>
          {construction.boundary_type === "adjacent_room" && (
            <div className="flex items-center gap-1 pl-1 text-[10px] text-on-surface-muted">
              <span className="shrink-0">&rarr;</span>
              <select
                value={construction.adjacent_room_id ?? ""}
                onChange={(e) => handleAdjacentRoomChange(e.target.value)}
                className={`min-w-0 flex-1 truncate rounded border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-input)] px-1 py-0.5 text-[10px] ${
                  adjacentRoom
                    ? "text-on-surface-secondary"
                    : "text-on-surface-muted italic"
                }`}
                title={
                  adjacentRoom
                    ? `Aangrenzend aan ${adjacentRoom.name}`
                    : "Kies aangrenzende ruimte"
                }
              >
                <option value="">Kies ruimte...</option>
                {adjacentRoomOptions.map((r) => {
                  const unheated =
                    roomDesignTemperature(r) <= UNHEATED_TEMPERATURE_THRESHOLD;
                  return (
                    <option key={r.id} value={r.id}>
                      {r.name}
                      {unheated ? " (onverwarmd)" : ""}
                    </option>
                  );
                })}
              </select>
            </div>
          )}
        </div>
      </td>
      <td className="px-2 py-1 text-right">
        <EditableCell
          value={construction.area}
          onChange={handleArea}
          type="number"
          unit="m²"
          displayFormatter={formatArea}
        />
      </td>
      <td className="px-2 py-1 text-right">
        <div className="flex flex-col items-end gap-0.5">
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
          {frameOverrideActive && (
            <span
              className="text-[10px] italic text-amber-400"
              title={`Project-override actief: ${frameUValueOverride} W/(m²·K) wordt in de berekening gebruikt in plaats van deze waarde.`}
            >
              override actief
            </span>
          )}
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
          layerDisplayOverrides={layerDisplayOverrides}
        />
      )}
    </>
  );
});
