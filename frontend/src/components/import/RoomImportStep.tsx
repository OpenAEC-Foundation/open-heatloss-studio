/**
 * Step 2 — Room review table.
 *
 * Shows imported rooms with editable type (heated/unheated).
 * Pseudo-rooms (outside, ground, water) are greyed out and not editable.
 */
import { useCallback } from "react";
import { AlertCircle } from "lucide-react";

import type { ThermalRoom, ThermalRoomType } from "../../lib/thermalImport";
import { isPseudoRoom, roomTypeLabel } from "../../lib/thermalImport";
import { formatArea } from "../../lib/formatNumber";

interface RoomImportStepProps {
  rooms: ThermalRoom[];
  onRoomsChange: (rooms: ThermalRoom[]) => void;
}

export function RoomImportStep({ rooms, onRoomsChange }: RoomImportStepProps) {
  const realRooms = rooms.filter((r) => !isPseudoRoom(r));
  const pseudoRooms = rooms.filter((r) => isPseudoRoom(r));

  const handleTypeChange = useCallback(
    (roomId: string, newType: ThermalRoomType) => {
      onRoomsChange(
        rooms.map((r) => (r.id === roomId ? { ...r, type: newType } : r)),
      );
    },
    [rooms, onRoomsChange],
  );

  return (
    <div>
      <h2 className="mb-2 text-lg font-semibold text-gray-100">
        Ruimtes controleren
      </h2>
      <p className="mb-6 text-sm text-gray-400">
        Controleer de geimporteerde ruimtes en pas het type aan indien nodig.
        Verwarmde ruimtes worden meegenomen in de warmteverliesberekening.
      </p>

      {/* Real rooms table */}
      <div className="overflow-x-auto rounded-lg border border-gray-700">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 bg-gray-800/80 text-left text-xs font-semibold uppercase tracking-wider text-gray-400">
              <th className="px-4 py-3">Naam</th>
              <th className="px-4 py-3">Type</th>
              <th className="px-4 py-3">Niveau</th>
              <th className="px-4 py-3 text-right">Oppervlakte [m2]</th>
              <th className="px-4 py-3 text-right">Hoogte [m]</th>
            </tr>
          </thead>
          <tbody>
            {realRooms.map((room) => (
              <tr
                key={room.id}
                className="border-b border-gray-700/50 transition-colors hover:bg-gray-800/40"
              >
                <td className="px-4 py-2.5 font-medium text-gray-200">
                  {room.name}
                </td>
                <td className="px-4 py-2.5">
                  <select
                    value={room.type}
                    onChange={(e) =>
                      handleTypeChange(
                        room.id,
                        e.target.value as ThermalRoomType,
                      )
                    }
                    className={`rounded border border-gray-600 bg-gray-800 px-2 py-1 text-sm focus:border-[#45B6A8] focus:outline-none ${
                      room.type === "heated"
                        ? "text-[#45B6A8]"
                        : "text-amber-400"
                    }`}
                  >
                    <option value="heated">Verwarmd</option>
                    <option value="unheated">Onverwarmd</option>
                  </select>
                </td>
                <td className="px-4 py-2.5 text-gray-400">
                  {room.level ?? "—"}
                </td>
                <td className="px-4 py-2.5 text-right tabular-nums text-gray-300">
                  {formatArea(room.area_m2)}
                </td>
                <td className="px-4 py-2.5 text-right tabular-nums text-gray-300">
                  {room.height_m != null ? room.height_m.toFixed(2) : "—"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pseudo rooms */}
      {pseudoRooms.length > 0 && (
        <div className="mt-4">
          <p className="mb-2 text-xs font-medium uppercase tracking-wider text-gray-500">
            Randcondities (niet bewerkbaar)
          </p>
          <div className="flex flex-wrap gap-2">
            {pseudoRooms.map((room) => (
              <span
                key={room.id}
                className="inline-flex items-center gap-1.5 rounded-full bg-gray-800 px-3 py-1 text-xs text-gray-500"
              >
                <span
                  className={`h-2 w-2 rounded-full ${
                    room.type === "outside"
                      ? "bg-blue-500/50"
                      : room.type === "ground"
                        ? "bg-amber-600/50"
                        : "bg-cyan-500/50"
                  }`}
                />
                {room.name} ({roomTypeLabel(room.type)})
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Stats */}
      <div className="mt-4 flex items-center gap-4 text-xs text-gray-500">
        <span>
          {realRooms.filter((r) => r.type === "heated").length} verwarmde
          ruimtes
        </span>
        <span>
          {realRooms.filter((r) => r.type === "unheated").length} onverwarmde
          ruimtes
        </span>
        <span>{pseudoRooms.length} randcondities</span>
      </div>

      {/* Warning if no heated rooms */}
      {realRooms.filter((r) => r.type === "heated").length === 0 && (
        <div className="mt-4 flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-3">
          <AlertCircle className="mt-0.5 h-4 w-4 flex-shrink-0 text-amber-400" />
          <p className="text-sm text-amber-300">
            Er zijn geen verwarmde ruimtes. Wijzig het type van minimaal
            een ruimte naar "Verwarmd" om een warmteverliesberekening te
            kunnen maken.
          </p>
        </div>
      )}
    </div>
  );
}
