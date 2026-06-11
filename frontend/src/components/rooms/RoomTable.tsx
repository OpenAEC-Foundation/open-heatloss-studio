import { useCallback, useRef, useState } from "react";

import type { CatalogueEntry } from "../../lib/constructionCatalogue";
import {
  createConstruction,
  createConstructionFromCatalogue,
  createRoom,
} from "../../lib/roomDefaults";
import { useProjectStore } from "../../store/projectStore";
import type { ConstructionElement, Room, Zone } from "../../types";
import { useModellerStore } from "../modeller/modellerStore";
import { getProjectConstructionUValue } from "../modeller/projectConstructionUtils";
import type { ProjectConstruction } from "../modeller/types";
import { ConstructionCells } from "./ConstructionRow";
import { ConstructionPicker } from "./ConstructionPicker";
import { RoomHeaderCells } from "./RoomHeaderRow";
import { VentilationRow } from "./VentilationRow";

const EMPTY_ROOM_CELLS = (
  <>
    <td className="border-r border-[var(--oaec-border-subtle)]" />
    <td className="border-r border-[var(--oaec-border-subtle)]" />
    <td className="border-r border-[var(--oaec-border-subtle)]" />
    <td className="border-r border-[var(--oaec-border-subtle)]" />
    <td className="border-r border-[var(--oaec-border-subtle)]" />
  </>
);

export function RoomTable() {
  const rooms = useProjectStore((s) => s.project.rooms);
  const defaultHeatingSystem = useProjectStore(
    (s) => s.project.building.default_heating_system,
  );
  const addRoom = useProjectStore((s) => s.addRoom);
  const updateRoom = useProjectStore((s) => s.updateRoom);
  const removeRoom = useProjectStore((s) => s.removeRoom);
  const addConstruction = useProjectStore((s) => s.addConstruction);
  const updateConstruction = useProjectStore((s) => s.updateConstruction);
  const removeConstruction = useProjectStore((s) => s.removeConstruction);

  const handleAddRoom = useCallback(() => {
    addRoom(createRoom(defaultHeatingSystem));
  }, [addRoom, defaultHeatingSystem]);

  const handleAddConstruction = useCallback(
    (roomId: string, construction: ConstructionElement) => {
      addConstruction(roomId, construction);
    },
    [addConstruction],
  );

  return (
    <div className="overflow-x-auto rounded-lg border border-[var(--oaec-border)]">
      <table className="w-full table-fixed border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-surface-alt">
          <tr className="border-b-2 border-[var(--oaec-border)] text-left text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
            <th className="w-[140px] border-r border-[var(--oaec-border-subtle)] px-2 py-2">Vertrek</th>
            <th className="w-[360px] border-r border-[var(--oaec-border-subtle)] px-2 py-2">Functie</th>
            <th className="w-[60px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              {"θ"}i
            </th>
            <th className="w-[78px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              A<sub>v</sub> [m{"²"}]
            </th>
            <th className="w-[60px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              h [m]
            </th>
            <th className="w-[300px] px-2 py-2">Grensvlak</th>
            <th className="w-[140px] px-2 py-2">Type</th>
            <th className="w-[78px] px-2 py-2 text-right">
              A [m{"²"}]
            </th>
            <th className="w-[88px] px-2 py-2 text-right">
              U [W/m{"²"}K]
            </th>
            <th className="w-[76px] px-2 py-2">Pos.</th>
            <th className="w-[34px] px-1 py-2" />
          </tr>
        </thead>
        <tbody>
          {rooms.map((room) => (
            <RoomGroup
              key={room.id}
              room={room}
              onUpdateRoom={(partial) => updateRoom(room.id, partial)}
              onRemoveRoom={() => removeRoom(room.id)}
              onAddConstruction={(c) => handleAddConstruction(room.id, c)}
              onUpdateConstruction={(cId, partial) =>
                updateConstruction(room.id, cId, partial)
              }
              onRemoveConstruction={(cId) => removeConstruction(room.id, cId)}
            />
          ))}
          {/* Add room ghost row */}
          <tr
            onClick={handleAddRoom}
            className="cursor-pointer border-t-2 border-[var(--oaec-border)] text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface"
          >
            <td colSpan={11} className="px-3 py-2 text-sm font-medium">
              + vertrek toevoegen
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}

/**
 * Compacte zone-dropdown in de vertrek-kopregel — alleen zichtbaar wanneer
 * het project zones heeft (geen zones → geen extra UI, tabel blijft exact
 * zoals voorheen). Schrijft via de undo-aware store-action `setRoomZone`.
 */
function RoomZoneSelect({ room, zones }: { room: Room; zones: Zone[] }) {
  const setRoomZone = useProjectStore((s) => s.setRoomZone);
  return (
    <label
      className="flex shrink-0 items-center gap-1.5"
      onClick={(e) => e.stopPropagation()}
      title="Zone-indeling (groepeert de ventilatiebalans)"
    >
      <span className="font-medium text-on-surface-muted">Zone</span>
      <select
        value={room.zoneId ?? ""}
        onChange={(e) => setRoomZone(room.id, e.target.value || undefined)}
        className="max-w-40 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-xs text-on-surface focus:border-primary focus:outline-none"
      >
        <option value="">— geen zone —</option>
        {zones.map((z) => (
          <option key={z.id} value={z.id}>
            {z.name}
          </option>
        ))}
      </select>
    </label>
  );
}

interface RoomGroupProps {
  room: Room;
  onUpdateRoom: (partial: Partial<Room>) => void;
  onRemoveRoom: () => void;
  onAddConstruction: (construction: ConstructionElement) => void;
  onUpdateConstruction: (
    constructionId: string,
    partial: Partial<ConstructionElement>,
  ) => void;
  onRemoveConstruction: (constructionId: string) => void;
}

function RoomGroup({
  room,
  onUpdateRoom,
  onRemoveRoom,
  onAddConstruction,
  onUpdateConstruction,
  onRemoveConstruction,
}: RoomGroupProps) {
  const zones = useProjectStore((s) => s.project.building.zones);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [anchorRect, setAnchorRect] = useState<DOMRect | null>(null);
  const [collapsed, setCollapsed] = useState(
    () => room.constructions.length !== 0,
  );
  const addBtnRef = useRef<HTMLTableCellElement>(null);
  const ensureProjectConstruction = useModellerStore(
    (s) => s.ensureProjectConstruction,
  );
  const { constructions } = room;
  const constructionCount = constructions.length;
  const summaryLabel =
    constructionCount === 0
      ? "Geen grensvlakken"
      : `${constructionCount} ${constructionCount === 1 ? "grensvlak" : "grensvlakken"}`;

  const handleOpenPicker = useCallback(() => {
    if (addBtnRef.current) {
      setAnchorRect(addBtnRef.current.getBoundingClientRect());
    }
    setPickerOpen((prev) => !prev);
  }, []);

  const handleSelectCatalogue = useCallback(
    (entry: CatalogueEntry) => {
      const ce = createConstructionFromCatalogue(entry);
      // Auto-register als project construction. Ook entries zonder lagen
      // (kozijnen/vullingen: triple-glas, buitendeur, etc.) krijgen een
      // project entry zodat ze via de project-picker opnieuw kiesbaar zijn.
      // `ensureProjectConstruction` normaliseert de uValue-invariant zelf.
      const pcId = ensureProjectConstruction({
        name: entry.name,
        category: entry.category,
        materialType: entry.materialType,
        verticalPosition: entry.verticalPosition,
        layers: (entry.layers ?? []).map((l) => ({ ...l })),
        uValue: entry.uValue,
        catalogueSourceId: entry.id,
      });
      ce.project_construction_id = pcId;
      onAddConstruction(ce);
      setPickerOpen(false);
    },
    [onAddConstruction, ensureProjectConstruction],
  );

  const handleSelectProject = useCallback(
    (pc: ProjectConstruction) => {
      const ce: ConstructionElement = {
        id: crypto.randomUUID(),
        description: pc.name,
        area: 0,
        u_value: getProjectConstructionUValue(pc),
        boundary_type: "exterior",
        material_type: pc.materialType,
        vertical_position: pc.verticalPosition,
        use_forfaitaire_thermal_bridge: true,
        layers: pc.layers.map((l) => ({ ...l })),
        project_construction_id: pc.id,
      };
      onAddConstruction(ce);
      setPickerOpen(false);
    },
    [onAddConstruction],
  );

  const handleSelectBlank = useCallback(() => {
    onAddConstruction(createConstruction());
    setPickerOpen(false);
  }, [onAddConstruction]);

  const toggleCollapse = useCallback(() => setCollapsed((prev) => !prev), []);

  return (
    <>
      {/* First row: room info + grensvlak-samenvatting/toggle */}
      <tr className="border-b border-[var(--oaec-border-subtle)] bg-[var(--oaec-hover)]">
        <RoomHeaderCells
          room={room}
          onUpdate={onUpdateRoom}
          onRemove={onRemoveRoom}
          collapsed={collapsed}
          onToggleCollapse={toggleCollapse}
        />
        <td
          colSpan={5}
          onClick={toggleCollapse}
          className="cursor-pointer px-2 py-1 text-xs text-on-surface-muted"
        >
          <div className="flex items-center justify-between gap-2">
            <span>{summaryLabel}</span>
            {zones !== undefined && zones.length > 0 && (
              <RoomZoneSelect room={room} zones={zones} />
            )}
          </div>
        </td>
        <td />
      </tr>

      {/* All construction rows — verborgen bij ingeklapt */}
      {!collapsed &&
        constructions.map((c) => (
          <tr key={c.id} className="border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]">
            {EMPTY_ROOM_CELLS}
            <ConstructionCells
              construction={c}
              onUpdate={(partial) => onUpdateConstruction(c.id, partial)}
              onRemove={() => onRemoveConstruction(c.id)}
              ownerRoomId={room.id}
            />
          </tr>
        ))}

      {/* Ventilation settings — persistant zichtbaar per vertrek */}
      <VentilationRow
        room={room}
        onUpdate={onUpdateRoom}
        heavyBottomBorder={collapsed}
      />

      {/* Add construction ghost row — alleen bij uitgeklapt (picker anchor) */}
      {!collapsed && (
        <tr
          onClick={handleOpenPicker}
          className="cursor-pointer border-b-2 border-[var(--oaec-border)] text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface"
        >
          {EMPTY_ROOM_CELLS}
          <td ref={addBtnRef} colSpan={5} className="px-3 py-1 text-xs font-medium">
            + grensvlak toevoegen
          </td>
          <td />
        </tr>
      )}

      {pickerOpen && (
        <ConstructionPicker
          anchorRect={anchorRect}
          onSelectCatalogue={handleSelectCatalogue}
          onSelectProject={handleSelectProject}
          onSelectBlank={handleSelectBlank}
          onClose={() => setPickerOpen(false)}
        />
      )}
    </>
  );
}
