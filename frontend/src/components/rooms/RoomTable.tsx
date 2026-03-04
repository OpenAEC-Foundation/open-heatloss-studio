import { useCallback, useState } from "react";

import type { CatalogueEntry } from "../../lib/constructionCatalogue";
import {
  createConstruction,
  createConstructionFromCatalogue,
  createRoom,
} from "../../lib/roomDefaults";
import { useProjectStore } from "../../store/projectStore";
import type { ConstructionElement, Room } from "../../types";
import { ConstructionCells } from "./ConstructionRow";
import { ConstructionPicker } from "./ConstructionPicker";
import { RoomHeaderCells } from "./RoomHeaderRow";

const EMPTY_ROOM_CELLS = (
  <>
    <td className="border-r border-stone-200" />
    <td className="border-r border-stone-200" />
    <td className="border-r border-stone-200" />
    <td className="border-r border-stone-200" />
    <td className="border-r border-stone-200" />
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
    <div className="overflow-x-auto rounded-lg border border-stone-200">
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-stone-100">
          <tr className="border-b-2 border-stone-300 text-left text-xs font-semibold uppercase tracking-wider text-stone-600">
            <th className="w-[140px] border-r border-stone-200 px-2 py-2">Vertrek</th>
            <th className="w-[120px] border-r border-stone-200 px-2 py-2">Functie</th>
            <th className="w-[70px] border-r border-stone-200 px-2 py-2 text-right">
              {"\u03B8"}i
            </th>
            <th className="w-[80px] border-r border-stone-200 px-2 py-2 text-right">
              A<sub>v</sub> [m{"\u00B2"}]
            </th>
            <th className="w-[70px] border-r border-stone-200 px-2 py-2 text-right">
              h [m]
            </th>
            <th className="w-[160px] px-2 py-2">Grensvlak</th>
            <th className="w-[160px] px-2 py-2">Type</th>
            <th className="w-[80px] px-2 py-2 text-right">
              A [m{"\u00B2"}]
            </th>
            <th className="w-[90px] px-2 py-2 text-right">
              U [W/m{"\u00B2"}K]
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
            className="cursor-pointer border-t-2 border-stone-200 text-stone-400 hover:bg-amber-50 hover:text-stone-600"
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
  const { constructions } = room;
  const firstConstruction = constructions[0];

  const handleSelectCatalogue = useCallback(
    (entry: CatalogueEntry) => {
      onAddConstruction(createConstructionFromCatalogue(entry));
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
      <tr className="border-b border-stone-100 bg-stone-50/50">
        <RoomHeaderCells
          room={room}
          onUpdate={onUpdateRoom}
          onRemove={onRemoveRoom}
        />
        {firstConstruction ? (
          <ConstructionCells
            construction={firstConstruction}
            onUpdate={(partial) => onUpdateConstruction(firstConstruction.id, partial)}
            onRemove={() => onRemoveConstruction(firstConstruction.id)}
          />
        ) : (
          <>
            <td colSpan={5} className="px-2 py-1 text-xs text-stone-400">
              Geen grensvlakken
            </td>
            <td />
          </>
        )}
      </tr>

      {/* Additional construction rows (index 1+) */}
      {constructions.slice(1).map((c) => (
        <tr key={c.id} className="border-b border-stone-100 hover:bg-stone-50/30">
          {EMPTY_ROOM_CELLS}
          <ConstructionCells
            construction={c}
            onUpdate={(partial) => onUpdateConstruction(c.id, partial)}
            onRemove={() => onRemoveConstruction(c.id)}
          />
        </tr>
      ))}

      {/* Add construction ghost row */}
      <tr
        onClick={() => setPickerOpen((prev) => !prev)}
        className="cursor-pointer border-b-2 border-stone-200 text-stone-400 hover:bg-blue-50 hover:text-stone-600"
      >
        {EMPTY_ROOM_CELLS}
        <td colSpan={5} className="relative px-3 py-1 text-xs font-medium">
          + grensvlak toevoegen
          {pickerOpen && (
            <ConstructionPicker
              onSelectCatalogue={handleSelectCatalogue}
              onSelectBlank={handleSelectBlank}
              onClose={() => setPickerOpen(false)}
            />
          )}
        </td>
        <td />
      </tr>
    </>
  );
}
