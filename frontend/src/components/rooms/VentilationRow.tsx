import { useCallback, useMemo } from "react";

import { HEATING_SYSTEM_LABELS } from "../../lib/constants";
import { bblMinimumVentilationRate } from "../../lib/roomDefaults";
import type { HeatingSystem, Room } from "../../types";

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

  const handleQvChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = e.target.value;
      // Empty field → null (auto-calculate from BBL)
      onUpdate({ ventilation_rate: val === "" ? null : Number(val) || 0 });
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
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onUpdate({ fraction_outside_air: Number(e.target.value) || 0 });
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
          {/* q_v */}
          <label className="flex items-center gap-1.5">
            <span className="font-medium text-on-surface-muted">
              q<sub>v</sub> [dm³/s]
            </span>
            <input
              type="number"
              step="any"
              min="0"
              value={room.ventilation_rate ?? ""}
              onChange={handleQvChange}
              className="w-16 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-right text-xs text-on-surface tabular-nums focus:border-primary focus:outline-none"
              placeholder={bblMinimum > 0 ? bblMinimum.toFixed(1) : "0"}
            />
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
            <input
              type="number"
              step="0.1"
              min="0"
              max="1"
              value={room.fraction_outside_air ?? ""}
              onChange={handleFractionChange}
              className="w-14 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-right text-xs text-on-surface tabular-nums focus:border-primary focus:outline-none"
              placeholder="1.0"
            />
          </label>

          {/* Visuele scheiding tussen ventilatie- en verwarmingssectie */}
          <div className="h-8 w-px bg-[var(--oaec-border)]" aria-hidden="true" />

          {/* Verwarmingssysteem per vertrek */}
          <label
            className="flex items-center gap-1.5"
            title="ISSO 51 Tabel 2.12 — bepaalt Δθ₁/Δθ₂/Δθᵥ correcties"
          >
            <span className="flex flex-col leading-tight">
              <span className="font-medium text-on-surface-muted">Verwarming</span>
              <span className="text-[10px] text-on-surface-muted">
                ISSO 51 Tabel 2.12
              </span>
            </span>
            <select
              value={room.heating_system}
              onChange={handleHeatingSystemChange}
              className="min-w-32 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-xs text-on-surface focus:border-primary focus:outline-none"
            >
              {Object.entries(HEATING_SYSTEM_LABELS).map(([value, label]) => (
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
