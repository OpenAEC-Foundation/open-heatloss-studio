import { useCallback, useRef, useState } from "react";

import type { CatalogueEntry } from "../../lib/constructionCatalogue";
import { calculateRc } from "../../lib/rcCalculation";
import {
  createConstruction,
  createConstructionFromCatalogue,
  createRoom,
} from "../../lib/roomDefaults";
import { useProjectStore } from "../../store/projectStore";
import type { ConstructionElement, Room } from "../../types";
import { useModellerStore } from "../modeller/modellerStore";
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
  const addRoom = useProjectStore((s) => s.addRoom);
  const updateRoom = useProjectStore((s) => s.updateRoom);
  const removeRoom = useProjectStore((s) => s.removeRoom);
  const addConstruction = useProjectStore((s) => s.addConstruction);
  const updateConstruction = useProjectStore((s) => s.updateConstruction);
  const removeConstruction = useProjectStore((s) => s.removeConstruction);

  const handleAddRoom = useCallback(() => {
    addRoom(createRoom());
  }, [addRoom]);

  const handleAddConstruction = useCallback(
    (roomId: string, construction: ConstructionElement) => {
      addConstruction(roomId, construction);
    },
    [addConstruction],
  );

  return (
    <div className="overflow-x-auto rounded-lg border border-[var(--oaec-border)]">
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-surface-alt">
          <tr className="border-b-2 border-[var(--oaec-border)] text-left text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
            <th className="w-[140px] border-r border-[var(--oaec-border-subtle)] px-2 py-2">Vertrek</th>
            <th className="w-[120px] border-r border-[var(--oaec-border-subtle)] px-2 py-2">Functie</th>
            <th className="w-[70px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              {"θ"}i
            </th>
            <th className="w-[80px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              A<sub>v</sub> [m{"²"}]
            </th>
            <th className="w-[70px] border-r border-[var(--oaec-border-subtle)] px-2 py-2 text-right">
              h [m]
            </th>
            <th className="w-[160px] px-2 py-2">Grensvlak</th>
            <th className="w-[160px] px-2 py-2">Type</th>
            <th className="w-[80px] px-2 py-2 text-right">
              A [m{"²"}]
            </th>
            <th className="w-[90px] px-2 py-2 text-right">
              U [W/m{"²"}K]
            </th>
            <th className="w-[80px] px-2 py-2">Pos.</th>
            <th className="w-[36px] px-1 py-2" />
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
  const [pickerOpen, setPickerOpen] = useState(false);
  const [ventOpen, setVentOpen] = useState(false);
  const [anchorRect, setAnchorRect] = useState<DOMRect | null>(null);
  const addBtnRef = useRef<HTMLTableCellElement>(null);
  const ensureProjectConstruction = useModellerStore(
    (s) => s.ensureProjectConstruction,
  );
  const { constructions } = room;
  const firstConstruction = constructions[0];

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
      const pcId = ensureProjectConstruction({
        name: entry.name,
        category: entry.category,
        materialType: entry.materialType,
        verticalPosition: entry.verticalPosition,
        layers: (entry.layers ?? []).map((l) => ({ ...l })),
        uValue: entry.layers && entry.layers.length > 0 ? undefined : entry.uValue,
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
      const rcResult = pc.layers.length > 0
        ? calculateRc(pc.layers, pc.verticalPosition)
        : null;
      const ce: ConstructionElement = {
        id: crypto.randomUUID(),
        description: pc.name,
        area: 0,
        // Fallback naar directe pc.uValue voor kozijnen/vullingen die geen
        // laag-gebaseerde Rc-berekening hebben (triple-glas, buitendeur).
        u_value: rcResult
          ? Math.round(rcResult.uValue * 1000) / 1000
          : pc.uValue ?? 0,
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

  return (
    <>
      {/* First row: room info + first construction (or empty) */}
      <tr className="border-b border-[var(--oaec-border-subtle)] bg-[var(--oaec-hover)]">
        <RoomHeaderCells
          room={room}
          onUpdate={onUpdateRoom}
          onRemove={onRemoveRoom}
          ventOpen={ventOpen}
          onToggleVent={() => setVentOpen((v) => !v)}
        />
        {firstConstruction ? (
          <ConstructionCells
            construction={firstConstruction}
            onUpdate={(partial) => onUpdateConstruction(firstConstruction.id, partial)}
            onRemove={() => onRemoveConstruction(firstConstruction.id)}
            ownerRoomId={room.id}
          />
        ) : (
          <>
            <td colSpan={5} className="px-2 py-1 text-xs text-on-surface-muted">
              Geen grensvlakken
            </td>
            <td />
          </>
        )}
      </tr>

      {/* Additional construction rows (index 1+) */}
      {constructions.slice(1).map((c) => (
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

      {/* Ventilation settings (uitklapbaar) */}
      {ventOpen && (
        <VentilationRow room={room} onUpdate={onUpdateRoom} />
      )}

      {/* Add construction ghost row */}
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
