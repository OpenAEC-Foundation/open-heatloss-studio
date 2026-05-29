import { useCallback, useMemo } from "react";

import { getHeatingSystemLabels, ROOM_FUNCTION_TEMPERATURES } from "../../lib/constants";
import { bblMinimumVentilationRate } from "../../lib/roomDefaults";
import { useProjectStore } from "../../store/projectStore";
import type { HeatingSystem, Room } from "../../types";
import { NumberInputBare } from "../ui/NumberInputBare";

/** Effectieve binnentemperatuur θ_i van een kamer (custom override of forfait). */
function roomInternalTemp(room: Room): number {
  return room.custom_temperature ?? ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;
}

interface VentilationRowProps {
  room: Room;
  onUpdate: (partial: Partial<Room>) => void;
}

/**
 * Uitklapbare rij met ventilatie-instellingen per vertrek.
 *
 * Velden: q_v [dm³/s], mech. afvoer, mech. toevoer, f_buitenlucht.
 * Als q_v leeg is, wordt het BBL minimum als placeholder getoond
 * en door de Rust core automatisch berekend.
 */
export function VentilationRow({ room, onUpdate }: VentilationRowProps) {
  const bblMinimum = useMemo(
    () => bblMinimumVentilationRate(room.function, room.floor_area),
    [room.function, room.floor_area],
  );

  // Norm-aware heating-systeem opties (ISSO 51 woningen vs. ISSO 53 utiliteit).
  const norm = useProjectStore((s) => s.norm);
  const heatingLabels = useMemo(
    () => getHeatingSystemLabels(norm === "isso53" ? "isso53" : "isso51"),
    [norm],
  );
  const heatingTableRef = norm === "isso53" ? "ISSO 53 Tabel 2.3" : "ISSO 51 Tabel 2.12";

  // Bron-kamer dropdown: alle andere rooms in project (exclude self).
  const allRooms = useProjectStore((s) => s.project.rooms);
  const otherRooms = useMemo(
    () => allRooms.filter((r) => r.id !== room.id),
    [allRooms, room.id],
  );

  const handleAirSourceChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const val = e.target.value;
      if (val === "exterior") {
        // Buitenlucht/gevelrooster → terug naar default systeem-gedrag
        onUpdate({ air_source_room_id: null, supply_air_temperature: null });
        return;
      }
      const source = allRooms.find((r) => r.id === val);
      if (source) {
        // Overstroom uit bron-kamer: θ_t = source kamer's θ_i
        onUpdate({
          air_source_room_id: source.id,
          supply_air_temperature: roomInternalTemp(source),
        });
      }
    },
    [allRooms, onUpdate],
  );

  const handleQvChange = useCallback(
    (raw: string) => {
      // Empty field → null (auto-calculate from BBL)
      onUpdate({ ventilation_rate: raw === "" ? null : Number(raw) || 0 });
    },
    [onUpdate],
  );

  // m³/h is dezelfde grootheid in andere eenheid: 1 dm³/s = 3,6 m³/h
  const handleQvM3hChange = useCallback(
    (raw: string) => {
      if (raw === "") {
        onUpdate({ ventilation_rate: null });
        return;
      }
      const m3h = Number(raw) || 0;
      onUpdate({ ventilation_rate: m3h / 3.6 });
    },
    [onUpdate],
  );

  const handleExhaustChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onUpdate({ has_mechanical_exhaust: e.target.checked });
    },
    [onUpdate],
  );

  const handleSupplyChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onUpdate({ has_mechanical_supply: e.target.checked });
    },
    [onUpdate],
  );

  const handleFractionChange = useCallback(
    (raw: string) => {
      onUpdate({ fraction_outside_air: Number(raw) || 0 });
    },
    [onUpdate],
  );

  const handleHeatingSystemChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      onUpdate({ heating_system: e.target.value as HeatingSystem });
    },
    [onUpdate],
  );

  return (
    <tr className="border-b border-[var(--oaec-border-subtle)] bg-[var(--oaec-accent-soft)]">
      <td colSpan={11} className="px-3 py-2">
        <div className="flex items-center gap-6 text-xs">
          {/* q_v in dm³/s + m³/h naast elkaar — beide editable, syncen via 1 dm³/s = 3,6 m³/h */}
          <label className="flex items-center gap-1.5">
            <span className="font-medium text-on-surface-muted">
              q<sub>v</sub>
            </span>
            <NumberInputBare
              value={room.ventilation_rate ?? ""}
              onCommit={handleQvChange}
              className="w-16 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-right text-xs text-on-surface tabular-nums focus:border-primary focus:outline-none"
              placeholder={bblMinimum > 0 ? bblMinimum.toFixed(1) : "0"}
            />
            <span className="text-[10px] text-on-surface-muted">dm³/s</span>
            <span className="text-on-surface-muted">↔</span>
            <NumberInputBare
              value={
                room.ventilation_rate != null
                  ? (room.ventilation_rate * 3.6).toFixed(1)
                  : ""
              }
              onCommit={handleQvM3hChange}
              className="w-16 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-right text-xs text-on-surface tabular-nums focus:border-primary focus:outline-none"
              placeholder={bblMinimum > 0 ? (bblMinimum * 3.6).toFixed(1) : "0"}
            />
            <span className="text-[10px] text-on-surface-muted">m³/h</span>
            {bblMinimum > 0 && room.ventilation_rate == null && (
              <span className="text-[10px] text-on-surface-muted">BBL min.</span>
            )}
          </label>

          {/* Mech. afvoer */}
          <label className="flex items-center gap-1.5 text-on-surface-muted">
            <input
              type="checkbox"
              checked={room.has_mechanical_exhaust ?? false}
              onChange={handleExhaustChange}
              className="h-3.5 w-3.5 rounded border-[var(--oaec-border)] accent-primary"
            />
            <span className="font-medium">Mech. afvoer</span>
          </label>

          {/* Toevoer (mechanisch of natuurlijk) */}
          <label className="flex items-center gap-1.5 text-on-surface-muted">
            <input
              type="checkbox"
              checked={room.has_mechanical_supply ?? false}
              onChange={handleSupplyChange}
              className="h-3.5 w-3.5 rounded border-[var(--oaec-border)] accent-primary"
            />
            <span className="font-medium">Toevoer</span>
          </label>

          {/* f_buitenlucht */}
          <label className="flex items-center gap-1.5">
            <span className="font-medium text-on-surface-muted">
              f<sub>buitenlucht</sub>
            </span>
            <NumberInputBare
              value={room.fraction_outside_air ?? ""}
              onCommit={handleFractionChange}
              className="w-14 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-right text-xs text-on-surface tabular-nums focus:border-primary focus:outline-none"
              placeholder="1.0"
            />
          </label>

          {/* Bron van toevoerlucht — gevel/buiten of overstroom uit andere kamer */}
          <label
            className="flex items-center gap-1.5"
            title="Bron van de toevoerlucht. Buitenlucht via gevelrooster = systeem-default θ_t. Andere kamer = overstroom, θ_t = bron-kamer θ_i."
          >
            <span className="font-medium text-on-surface-muted">Lucht uit</span>
            <select
              value={room.air_source_room_id ?? "exterior"}
              onChange={handleAirSourceChange}
              className="min-w-28 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-xs text-on-surface focus:border-primary focus:outline-none"
            >
              <option value="exterior">Buitenlucht (gevel)</option>
              {otherRooms.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name || r.id} ({roomInternalTemp(r).toFixed(0)}°C)
                </option>
              ))}
            </select>
          </label>

          {/* Visuele scheiding tussen ventilatie- en verwarmingssectie */}
          <div className="h-8 w-px bg-[var(--oaec-border)]" aria-hidden="true" />

          {/* Verwarmingssysteem per vertrek */}
          <label
            className="flex items-center gap-1.5"
            title={`${heatingTableRef} — bepaalt Δθ₁/Δθ₂/Δθᵥ correcties`}
          >
            <span className="flex flex-col leading-tight">
              <span className="font-medium text-on-surface-muted">Verwarming</span>
              <span className="text-[10px] text-on-surface-muted">
                {heatingTableRef}
              </span>
            </span>
            <select
              value={room.heating_system}
              onChange={handleHeatingSystemChange}
              className="min-w-32 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-xs text-on-surface focus:border-primary focus:outline-none"
            >
              {Object.entries(heatingLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
        </div>
      </td>
    </tr>
  );
}
