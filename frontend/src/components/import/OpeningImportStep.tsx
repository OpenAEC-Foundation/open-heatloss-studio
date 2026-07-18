/**
 * Step 4 — Opening review table.
 *
 * Shows windows, doors, and curtain walls with editable U-values.
 * Curtain walls show their Revit U-value as default.
 */
import { useCallback } from "react";

import type {
  ThermalOpening,
  ThermalConstruction,
  ThermalRoom,
} from "../../lib/thermalImport";
import { openingTypeLabel } from "../../lib/thermalImport";

interface OpeningImportStepProps {
  openings: ThermalOpening[];
  constructions: ThermalConstruction[];
  rooms: ThermalRoom[];
  onOpeningsChange: (openings: ThermalOpening[]) => void;
}

/** Get the room_a name for a construction. */
function getHostRoomName(
  constructionId: string,
  constructions: ThermalConstruction[],
  rooms: ThermalRoom[],
): string {
  const constr = constructions.find((c) => c.id === constructionId);
  if (!constr) return "—";
  const room = rooms.find((r) => r.id === constr.room_a);
  return room?.name ?? constr.room_a;
}

export function OpeningImportStep({
  openings,
  constructions,
  rooms,
  onOpeningsChange,
}: OpeningImportStepProps) {
  const handleUValueChange = useCallback(
    (openingId: string, value: string) => {
      const numValue = parseFloat(value);
      onOpeningsChange(
        openings.map((o) =>
          o.id === openingId
            ? { ...o, u_value: isNaN(numValue) ? undefined : numValue }
            : o,
        ),
      );
    },
    [openings, onOpeningsChange],
  );

  const windows = openings.filter((o) => o.type === "window");
  const doors = openings.filter((o) => o.type === "door");
  const curtainWalls = openings.filter((o) => o.type === "curtain_wall");

  if (openings.length === 0) {
    return (
      <div>
        <h2 className="mb-2 text-lg font-semibold text-gray-100">
          Openingen controleren
        </h2>
        <p className="mt-6 text-center text-sm text-gray-500">
          Geen openingen gevonden in het importbestand.
        </p>
      </div>
    );
  }

  return (
    <div>
      <h2 className="mb-2 text-lg font-semibold text-gray-100">
        Openingen controleren
      </h2>
      <p className="mb-6 text-sm text-gray-400">
        Controleer de openingen en vul de U-waarden in. Voor vliesgevels wordt de
        U-waarde uit Revit als standaard overgenomen.
      </p>

      {/* Openings table */}
      <div className="overflow-x-auto rounded-lg border border-gray-700">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 bg-gray-800/80 text-left text-xs font-semibold uppercase tracking-wider text-gray-400">
              <th className="px-4 py-3">Type</th>
              <th className="px-4 py-3">Revit type</th>
              <th className="px-4 py-3">Ruimte</th>
              <th className="px-4 py-3 text-right">Afmetingen [mm]</th>
              <th className="px-4 py-3 text-right">Kozijnhoogte [mm]</th>
              <th className="px-4 py-3 text-right">U-waarde [W/m2K]</th>
            </tr>
          </thead>
          <tbody>
            {/* Windows */}
            {windows.length > 0 && (
              <SectionHeader label="Ramen" count={windows.length} />
            )}
            {windows.map((o) => (
              <OpeningRow
                key={o.id}
                opening={o}
                hostRoom={getHostRoomName(o.construction_id, constructions, rooms)}
                onUValueChange={handleUValueChange}
              />
            ))}

            {/* Doors */}
            {doors.length > 0 && (
              <SectionHeader label="Deuren" count={doors.length} />
            )}
            {doors.map((o) => (
              <OpeningRow
                key={o.id}
                opening={o}
                hostRoom={getHostRoomName(o.construction_id, constructions, rooms)}
                onUValueChange={handleUValueChange}
              />
            ))}

            {/* Curtain walls */}
            {curtainWalls.length > 0 && (
              <SectionHeader label="Vliesgevels" count={curtainWalls.length} />
            )}
            {curtainWalls.map((o) => (
              <OpeningRow
                key={o.id}
                opening={o}
                hostRoom={getHostRoomName(o.construction_id, constructions, rooms)}
                onUValueChange={handleUValueChange}
              />
            ))}
          </tbody>
        </table>
      </div>

      {/* Stats */}
      <div className="mt-4 flex items-center gap-4 text-xs text-gray-500">
        <span>{windows.length} ramen</span>
        <span>{doors.length} deuren</span>
        <span>{curtainWalls.length} vliesgevels</span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function SectionHeader({ label, count }: { label: string; count: number }) {
  return (
    <tr className="bg-gray-800/30">
      <td
        colSpan={6}
        className="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        {label} ({count})
      </td>
    </tr>
  );
}

interface OpeningRowProps {
  opening: ThermalOpening;
  hostRoom: string;
  onUValueChange: (id: string, value: string) => void;
}

function OpeningRow({ opening, hostRoom, onUValueChange }: OpeningRowProps) {
  return (
    <tr className="border-b border-gray-700/50 transition-colors hover:bg-gray-800/40">
      <td className="px-4 py-2.5">
        <span
          className={`inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium ${
            opening.type === "window"
              ? "bg-blue-500/10 text-blue-400"
              : opening.type === "door"
                ? "bg-[var(--theme-accent-soft)] text-[var(--theme-accent)]"
                : "bg-purple-500/10 text-purple-400"
          }`}
        >
          {openingTypeLabel(opening.type)}
        </span>
      </td>
      <td className="px-4 py-2.5 text-sm text-gray-300">
        {opening.revit_type_name ?? "—"}
      </td>
      <td className="px-4 py-2.5 text-sm text-gray-400">{hostRoom}</td>
      <td className="px-4 py-2.5 text-right tabular-nums text-gray-300">
        {opening.width_mm} x {opening.height_mm}
      </td>
      <td className="px-4 py-2.5 text-right tabular-nums text-gray-400">
        {opening.sill_height_mm != null ? opening.sill_height_mm : "—"}
      </td>
      <td className="px-4 py-2.5 text-right">
        <input
          type="number"
          step="0.01"
          min="0"
          value={opening.u_value ?? ""}
          onChange={(e) => onUValueChange(opening.id, e.target.value)}
          placeholder={opening.type === "curtain_wall" ? "Revit" : "—"}
          className="w-20 rounded border border-gray-600 bg-gray-800 px-2 py-1 text-right text-sm tabular-nums text-gray-200 placeholder-gray-600 focus:border-[#45B6A8] focus:outline-none"
        />
      </td>
    </tr>
  );
}
