/**
 * Ventilatiebalans-zijpaneel (Modeller, ventilatie-mode).
 *
 * Toont per vertrek de BBL-eis (toevoer/afvoer in dm³/s, m³/h secundair),
 * het aanwezige ventiel-debiet en de status, plus de gebouwbalans onderaan
 * en de systeem A–D-selector bovenaan. Volgt het panel-patroon van
 * `PropertiesPanel` (rechter zijpaneel, `border-l` + `bg-surface-alt`).
 *
 * Gedeelde bouwstenen (selector, status-badge, gebouwbalans, labels) leven in
 * `components/ventilation/shared.tsx` en worden ook door de
 * Ventilatiebalans-tab (`pages/VentilationBalance.tsx`) gebruikt.
 *
 * **Eenheden:** dm³/s intern; m³/h alleen als afgeleide weergave.
 */

import { useMemo } from "react";

import type { Room } from "../../types";
import type { Selection } from "./types";
import {
  ventilationSystemOf,
  type BblFunctionKey,
  type VentilationState,
  type VentilationSystemKey,
  type VentilationRoomState,
} from "../../types/ventilation";
import {
  aggregateVentilationBalance,
  type RoomVentilationBalance,
} from "../../lib/ventilationBalance";
import {
  BuildingBalanceSummary,
  FUNCTION_OPTIONS,
  StatusBadge,
  SystemSelector,
  flowLabel,
  m3hLabel,
} from "../ventilation/shared";
import { formatArea } from "../../lib/formatNumber";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface VentilationBalancePanelProps {
  /** Project-ruimtes (calc-bron van waarheid: naam + floor_area). */
  rooms: Room[];
  /** Afgeleide per-room ventilatie-state (eisen in dm³/s), gekeyed op id. */
  ventilationRooms: Record<string, VentilationRoomState>;
  /** Ventilatie-sidecar (terminals + systeem). */
  ventilation: VentilationState;
  /** Huidige canvas-selectie (voor rij-highlight). */
  selection: Selection;
  /** Selecteer/highlight een ruimte op de canvas. */
  onSelectRoom: (roomId: string) => void;
  /** Schrijf de gebruiksfunctie-override voor een ruimte. */
  onChangeFunction: (roomId: string, fn: BblFunctionKey) => void;
  /** Schrijf de bezetting (personen-toeslag); `undefined` = geen toeslag. */
  onChangeOccupancy: (roomId: string, occupancy: number | undefined) => void;
  /** Zet het gebouw-niveau ventilatiesysteem. */
  onChangeSystem: (system: VentilationSystemKey) => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function VentilationBalancePanel({
  rooms,
  ventilationRooms,
  ventilation,
  selection,
  onSelectRoom,
  onChangeFunction,
  onChangeOccupancy,
  onChangeSystem,
}: VentilationBalancePanelProps) {
  const balance = useMemo(
    () =>
      aggregateVentilationBalance(
        ventilationRooms,
        ventilation.terminals,
        ventilation.system,
      ),
    [ventilationRooms, ventilation.terminals, ventilation.system],
  );
  const sys = ventilationSystemOf(ventilation);

  const selectedRoomId =
    selection && "roomId" in selection ? selection.roomId : null;

  return (
    <div className="flex w-96 shrink-0 flex-col overflow-y-auto border-l border-[var(--oaec-border)] bg-surface-alt">
      {/* Header */}
      <div className="border-b border-[var(--oaec-border-subtle)] px-4 py-3">
        <span className="text-sm font-bold text-on-surface">
          Ventilatiebalans
        </span>
        <div className="mt-1 text-xs text-on-surface-muted">
          BBL-eis per vertrek + gebouwbalans (dm³/s)
        </div>
      </div>

      {/* Systeem A–D selector */}
      <div className="border-b border-[var(--oaec-border-subtle)] px-4 py-3">
        <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-scaffold-gray">
          Ventilatiesysteem
        </div>
        <SystemSelector value={sys.key} onChange={onChangeSystem} />
      </div>

      {/* Per-vertrek tabel */}
      <div className="flex-1 px-2 py-2">
        {rooms.length === 0 && (
          <div className="px-2 py-4 text-xs text-on-surface-muted">
            Geen vertrekken in het project.
          </div>
        )}
        {rooms.map((room) => {
          const vr = ventilationRooms[room.id];
          const row = balance.rooms[room.id];
          if (!vr || !row) return null;
          return (
            <RoomRow
              key={room.id}
              room={room}
              vr={vr}
              row={row}
              supplyMechanical={sys.supplyMechanical}
              exhaustMechanical={sys.exhaustMechanical}
              selected={selectedRoomId === room.id}
              onSelect={() => onSelectRoom(room.id)}
              onChangeFunction={(fn) => onChangeFunction(room.id, fn)}
              onChangeOccupancy={(n) => onChangeOccupancy(room.id, n)}
            />
          );
        })}
      </div>

      {/* Gebouwbalans */}
      <div className="border-t border-[var(--oaec-border)] px-4 py-3">
        <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-scaffold-gray">
          Gebouwbalans
        </div>
        <BuildingBalanceSummary balance={balance} />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Per-vertrek rij
// ---------------------------------------------------------------------------

interface RoomRowProps {
  room: Room;
  vr: VentilationRoomState;
  row: RoomVentilationBalance;
  supplyMechanical: boolean;
  exhaustMechanical: boolean;
  selected: boolean;
  onSelect: () => void;
  onChangeFunction: (fn: BblFunctionKey) => void;
  onChangeOccupancy: (occupancy: number | undefined) => void;
}

function RoomRow({
  room,
  vr,
  row,
  supplyMechanical,
  exhaustMechanical,
  selected,
  onSelect,
  onChangeFunction,
  onChangeOccupancy,
}: RoomRowProps) {
  const isSupply = vr.requiredSupplyDm3s > 0;
  const isExhaust = vr.requiredExhaustDm3s > 0;
  const required = isSupply
    ? vr.requiredSupplyDm3s
    : isExhaust
      ? vr.requiredExhaustDm3s
      : 0;
  const present = isSupply ? row.presentSupplyDm3s : row.presentExhaustDm3s;
  const mechanical = isSupply ? supplyMechanical : exhaustMechanical;
  const deficit = isSupply ? row.supplyDeficitDm3s : row.exhaustDeficitDm3s;

  return (
    <div
      className={`mb-1.5 cursor-pointer rounded-md border px-2.5 py-2 transition-colors ${
        selected
          ? "border-primary bg-primary/10"
          : "border-primary/15 bg-surface hover:bg-primary/5"
      }`}
      onClick={onSelect}
    >
      {/* Kop: naam · m² · status */}
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate text-xs font-semibold text-on-surface">
          {room.name}
        </span>
        <span className="text-[10px] tabular-nums text-scaffold-gray">
          {formatArea(room.floor_area)} m²
        </span>
        <StatusBadge
          isSupply={isSupply}
          isExhaust={isExhaust}
          mechanical={mechanical}
          deficit={deficit}
        />
      </div>

      {/* Functie-dropdown + bezetting */}
      <div className="mt-1.5 flex items-center gap-1.5">
        <select
          value={vr.ventilationFunction}
          onChange={(e) =>
            onChangeFunction(e.target.value as BblFunctionKey)
          }
          onClick={(e) => e.stopPropagation()}
          className="min-w-0 flex-1 rounded border border-primary/20 bg-surface px-1 py-0.5 text-[10px] text-on-surface"
          title="BBL-gebruiksfunctie (override)"
        >
          {FUNCTION_OPTIONS.map((fn) => (
            <option key={fn} value={fn}>
              {fn}
            </option>
          ))}
        </select>
        <label
          className="flex items-center gap-1 text-[10px] text-scaffold-gray"
          onClick={(e) => e.stopPropagation()}
        >
          pers.
          <input
            type="number"
            min={0}
            step={1}
            value={vr.occupancy ?? ""}
            placeholder="–"
            onChange={(e) => {
              const v = e.target.value;
              if (v === "") {
                onChangeOccupancy(undefined);
                return;
              }
              const n = Number(v);
              onChangeOccupancy(
                Number.isFinite(n) && n > 0 ? Math.floor(n) : undefined,
              );
            }}
            className="w-12 rounded border border-primary/20 bg-surface px-1 py-0.5 text-right text-[10px] tabular-nums text-on-surface"
            title="Bezetting (personen-toeslag: max(opp×dm³/m², pers×pp, minimum))"
          />
        </label>
      </div>

      {/* Cijfers: eis / aanwezig */}
      <div className="mt-1.5 grid grid-cols-2 gap-1 text-[10px] tabular-nums">
        <div>
          <span className="text-scaffold-gray">
            Eis {isSupply ? "toevoer" : isExhaust ? "afvoer" : "—"}
          </span>
          <div className="font-medium text-on-surface">
            {isSupply || isExhaust ? (
              <>
                {flowLabel(required)}{" "}
                <span className="font-normal text-scaffold-gray">
                  ({m3hLabel(required)})
                </span>
              </>
            ) : (
              "geen eis"
            )}
          </div>
        </div>
        <div>
          <span className="text-scaffold-gray">Aanwezig</span>
          <div className="font-medium text-on-surface">
            {isSupply || isExhaust ? (
              mechanical ? (
                <>
                  {flowLabel(present)}{" "}
                  <span className="font-normal text-scaffold-gray">
                    ({m3hLabel(present)})
                  </span>
                </>
              ) : (
                <span className="font-normal text-scaffold-gray">
                  via gevelroosters
                </span>
              )
            ) : (
              "—"
            )}
          </div>
        </div>
      </div>

      {/* Ventielen zonder debiet */}
      {row.missingFlowCount > 0 && (
        <div className="mt-1 text-[10px] font-medium text-amber-600">
          ⚠ {row.missingFlowCount} ventiel
          {row.missingFlowCount > 1 ? "en" : ""} zonder debiet (telt als 0)
        </div>
      )}
    </div>
  );
}
