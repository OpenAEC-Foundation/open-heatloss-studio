import { useCallback } from "react";

import {
  ROOM_FUNCTION_LABELS,
  ROOM_FUNCTION_TEMPERATURES,
} from "../../lib/constants";
import type { Room, RoomFunction } from "../../types";
import { EditableCell } from "./EditableCell";
import { EditableSelect } from "./EditableSelect";

interface RoomHeaderRowProps {
  room: Room;
  onUpdate: (partial: Partial<Room>) => void;
  onRemove: () => void;
}

/**
 * Renders room-level cells (name, function, theta_i, floor_area, height).
 * Does NOT render a full <tr> — returns cell fragments to be composed by parent.
 */
export function RoomHeaderCells({
  room,
  onUpdate,
  onRemove,
}: RoomHeaderRowProps) {
  const thetaI =
    room.custom_temperature ?? ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;

  const handleFunctionChange = useCallback(
    (v: string) => {
      const fn = v as RoomFunction;
      onUpdate({
        function: fn,
        custom_temperature: null,
      });
    },
    [onUpdate],
  );

  const handleThetaChange = useCallback(
    (v: string) => {
      const num = Number(v);
      const defaultTemp = ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;
      onUpdate({
        custom_temperature: num === defaultTemp ? null : num,
      });
    },
    [onUpdate, room.function],
  );

  return (
    <>
      <td className="border-r border-stone-200 px-2 py-1 font-medium">
        <div className="flex items-center justify-between gap-1">
          <EditableCell
            value={room.name}
            onChange={(v) => onUpdate({ name: v })}
            placeholder="Vertreknaam..."
          />
          <button
            onClick={onRemove}
            className="shrink-0 rounded p-0.5 text-stone-400 hover:bg-red-50 hover:text-red-600"
            title="Verwijder vertrek"
          >
            <svg className="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        </div>
      </td>
      <td className="border-r border-stone-200 px-2 py-1">
        <EditableSelect
          value={room.function}
          onChange={handleFunctionChange}
          options={ROOM_FUNCTION_LABELS}
        />
      </td>
      <td className="border-r border-stone-200 px-2 py-1 text-right">
        <EditableCell
          value={thetaI}
          onChange={handleThetaChange}
          type="number"
          unit={"\u00B0C"}
        />
      </td>
      <td className="border-r border-stone-200 px-2 py-1 text-right">
        <EditableCell
          value={room.floor_area}
          onChange={(v) => onUpdate({ floor_area: Number(v) || 0 })}
          type="number"
          unit="m\u00B2"
        />
      </td>
      <td className="border-r border-stone-200 px-2 py-1 text-right">
        <EditableCell
          value={room.height ?? 2.6}
          onChange={(v) => onUpdate({ height: Number(v) || 2.6 })}
          type="number"
          unit="m"
        />
      </td>
    </>
  );
}
