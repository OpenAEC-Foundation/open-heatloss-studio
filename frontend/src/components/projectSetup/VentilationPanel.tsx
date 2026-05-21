/**
 * VentilationPanel — ventilatie-instellingen op project-niveau.
 *
 * Schrijft naar:
 *  - V1 `project.ventilation.system_type` (ISSO 51 systeem A–E, direct)
 *  - V1 `project.ventilation.has_heat_recovery` + `heat_recovery_efficiency`
 *  - V2 sidecar `sharedExtra.{infiltration,mechanical_supply,mechanical_exhaust}_m3_per_h`
 *
 * V1 blijft single-source-of-truth voor de ISSO 51 backend; de gebruiker
 * kiest hier rechtstreeks het ISSO 51 ventilatiesysteem (A t/m E). De
 * V1→V2 mapping voor de NTA 8800 / TO-juli backend gebeurt in
 * `projectV2Migration.ts` (`mapV1SystemTypeToV2`) bij `buildV2Payload`.
 *
 * De V2-only m³/h velden (infiltratie + mechanisch toe/afvoer) landen via
 * `buildV2Payload` in de SharedProject ventilation surface en blijven
 * `undefined` als de gebruiker leeg laat, zodat de Rust engine terugvalt
 * op `default_ach` lookup.
 *
 * Lessons learned:
 *  - 10-04 (water θ): norm-afwijkende aannames structureel tonen → SFP
 *    forfaitair-voetnoot is altijd zichtbaar onder de invoer.
 *  - 17-05 (light theme): gebruik `--oaec-*` tokens, geen hardcoded kleuren.
 *  - 21-05 (regressie b546610): systeemkeuze terug naar ISSO 51 A–E;
 *    V2-kind-select downgrade systeem E stil naar D.
 */
import { useCallback } from "react";

import { VENTILATION_SYSTEM_LABELS } from "../../lib/constants";
import { useProjectStore } from "../../store/projectStore";
import type { VentilationConfig, VentilationSystemType } from "../../types";
import type { HeatRecovery } from "../../types/projectV2";
import { Card } from "../ui/Card";
import { Input } from "../ui/Input";
import { Select } from "../ui/Select";

// ---------------------------------------------------------------------------
// ISSO 51 ventilatiesysteem — debiet-/WTW-velden per systeemtype
// ---------------------------------------------------------------------------

/**
 * Per ISSO 51 systeemtype: welke mechanische debiet-velden + WTW-veld zichtbaar
 * zijn. Consistent met de oude `WarmteverliesInstellingen.tsx` vóór b546610:
 * `supportsWtw = system_d || system_e`.
 *
 *   A — natuurlijke toe- en afvoer            → geen mech velden, geen WTW
 *   B — mechanische toevoer, natuurlijke afvoer → toevoer
 *   C — natuurlijke toevoer, mechanische afvoer → afvoer
 *   D — gebalanceerd mechanisch (centraal)     → toevoer + afvoer + WTW
 *   E — combinatie (decentraal gebalanceerd)   → toevoer + afvoer + WTW
 */
const SYSTEM_CAPABILITIES: Record<
  VentilationSystemType,
  { hasSupply: boolean; hasExhaust: boolean; hasWtw: boolean }
> = {
  system_a: { hasSupply: false, hasExhaust: false, hasWtw: false },
  system_b: { hasSupply: true, hasExhaust: false, hasWtw: false },
  system_c: { hasSupply: false, hasExhaust: true, hasWtw: false },
  system_d: { hasSupply: true, hasExhaust: true, hasWtw: true },
  system_e: { hasSupply: true, hasExhaust: true, hasWtw: true },
};

const VENTILATION_SYSTEM_OPTIONS: Array<{
  value: VentilationSystemType;
  label: string;
}> = (
  ["system_a", "system_b", "system_c", "system_d", "system_e"] as const
).map((value) => ({
  value,
  label: VENTILATION_SYSTEM_LABELS[value] ?? value,
}));

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function VentilationPanel() {
  const { project, updateProject, sharedExtra, updateSharedExtra } =
    useProjectStore();

  const v1Vent = project.ventilation;
  const currentSystem: VentilationSystemType = v1Vent.system_type;
  const { hasSupply, hasExhaust, hasWtw } = SYSTEM_CAPABILITIES[currentSystem];

  const updateVentilation = useCallback(
    (partial: Partial<VentilationConfig>) => {
      updateProject({ ventilation: { ...project.ventilation, ...partial } });
    },
    [project.ventilation, updateProject],
  );

  const handleSystemChange = useCallback(
    (next: VentilationSystemType) => {
      // WTW alleen behouden bij gebalanceerde systemen (D/E). Anders wissen.
      const keepWtw = SYSTEM_CAPABILITIES[next].hasWtw;
      updateVentilation({
        system_type: next,
        ...(keepWtw
          ? {}
          : {
              has_heat_recovery: false,
              heat_recovery_efficiency: undefined,
              frost_protection: undefined,
              supply_temperature: undefined,
            }),
      });
    },
    [updateVentilation],
  );

  // heat_recovery efficiency UI-waarde (procenten) — leesbaar uit V1.
  const heatRecoveryPct =
    v1Vent.has_heat_recovery && v1Vent.heat_recovery_efficiency != null
      ? Math.round(v1Vent.heat_recovery_efficiency * 100)
      : "";

  const handleHeatRecoveryEfficiencyChange = (raw: string) => {
    if (raw === "") {
      updateVentilation({
        has_heat_recovery: false,
        heat_recovery_efficiency: undefined,
      });
      return;
    }
    const pct = Number(raw);
    if (!Number.isFinite(pct)) return;
    const clamped = Math.max(0, Math.min(100, pct));
    updateVentilation({
      has_heat_recovery: true,
      heat_recovery_efficiency: clamped / 100,
    });
  };

  // V2-only m³/h velden — naar sidecar.
  const supplyValue = sharedExtra.mechanical_supply_m3_per_h ?? "";
  const exhaustValue = sharedExtra.mechanical_exhaust_m3_per_h ?? "";
  const infiltrationValue = sharedExtra.infiltration_m3_per_h ?? "";

  const writeNumericExtra = (
    key:
      | "mechanical_supply_m3_per_h"
      | "mechanical_exhaust_m3_per_h"
      | "infiltration_m3_per_h",
    raw: string,
  ) => {
    if (raw === "") {
      updateSharedExtra({ [key]: null });
      return;
    }
    const n = Number(raw);
    if (!Number.isFinite(n) || n < 0) return;
    updateSharedExtra({ [key]: n });
  };

  // Toon hint over hoe het WTW-veld zich verhoudt tot V1 ISSO 51 ventilatie.
  const heatRecoveryHint: HeatRecovery | null =
    hasWtw &&
    v1Vent.has_heat_recovery &&
    v1Vent.heat_recovery_efficiency != null
      ? {
          efficiency: v1Vent.heat_recovery_efficiency,
          frost_protection:
            v1Vent.frost_protection != null &&
            v1Vent.frost_protection !== "unknown",
          ...(v1Vent.supply_temperature != null
            ? { supply_temperature: v1Vent.supply_temperature }
            : {}),
        }
      : null;

  return (
    <Card title="Ventilatie (ISSO 51 / TO-juli)">
      <div className="grid grid-cols-3 gap-4">
        <Select
          id="ventilation_system_type"
          label="Ventilatiesysteem"
          value={currentSystem}
          options={VENTILATION_SYSTEM_OPTIONS}
          onChange={(e) =>
            handleSystemChange(e.target.value as VentilationSystemType)
          }
        />
      </div>

      <div className="mt-4 border-t border-[var(--oaec-border-subtle)] pt-4">
        <p className="mb-2 text-xs font-medium text-on-surface-secondary">
          Debieten (optioneel — NTA 8800 / TO-juli)
        </p>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <Input
              id="infiltration_m3_per_h"
              label="Basisinfiltratie"
              type="number"
              unit="m³/h"
              step={1}
              min={0}
              placeholder="Auto (NTA 8800)"
              value={infiltrationValue}
              onChange={(e) =>
                writeNumericExtra("infiltration_m3_per_h", e.target.value)
              }
            />
            <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
              TO-juli/NTA 8800 debiet-override. Niet de ISSO 51 luchtdichtheid
              `qv10` (Gebouw-tabblad) — leeg laten = backend rekent zelf
              (`default_ach`).
            </p>
          </div>
          {hasSupply && (
            <div>
              <Input
                id="mechanical_supply_m3_per_h"
                label="Mechanische toevoer"
                type="number"
                unit="m³/h"
                step={1}
                min={0}
                placeholder="Auto (NTA 8800)"
                value={supplyValue}
                onChange={(e) =>
                  writeNumericExtra(
                    "mechanical_supply_m3_per_h",
                    e.target.value,
                  )
                }
              />
              <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                Optionele debiet-override; leeg laten = `default_ach`.
              </p>
            </div>
          )}
          {hasExhaust && (
            <div>
              <Input
                id="mechanical_exhaust_m3_per_h"
                label="Mechanische afvoer"
                type="number"
                unit="m³/h"
                step={1}
                min={0}
                placeholder="Auto (NTA 8800)"
                value={exhaustValue}
                onChange={(e) =>
                  writeNumericExtra(
                    "mechanical_exhaust_m3_per_h",
                    e.target.value,
                  )
                }
              />
              <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                Optionele debiet-override; leeg laten = `default_ach`.
              </p>
            </div>
          )}
        </div>
      </div>

      {hasWtw && (
        <div className="mt-4 grid grid-cols-3 gap-4 border-t border-[var(--oaec-border-subtle)] pt-4">
          <div>
            <Input
              id="heat_recovery_efficiency_v2"
              label="WTW-rendement"
              type="number"
              unit="%"
              step={1}
              min={0}
              max={100}
              placeholder="Geen WTW"
              value={heatRecoveryPct}
              onChange={(e) =>
                handleHeatRecoveryEfficiencyChange(e.target.value)
              }
            />
            <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
              Leeg laten voor geen WTW. Spiegelt naar V1 ISSO 51
              `has_heat_recovery` + `heat_recovery_efficiency`.
            </p>
          </div>
          {heatRecoveryHint && (
            <div className="col-span-2 flex items-center">
              <div className="rounded-md bg-[var(--oaec-accent-soft)] px-3 py-2 text-xs">
                <span className="text-on-surface-muted">η_WTW: </span>
                <span className="font-semibold tabular-nums">
                  {Math.round(heatRecoveryHint.efficiency * 100)}%
                </span>
                {heatRecoveryHint.supply_temperature != null && (
                  <>
                    <span className="ml-2 text-on-surface-muted">
                      θ_toevoer:{" "}
                    </span>
                    <span className="font-semibold tabular-nums">
                      {heatRecoveryHint.supply_temperature}°C
                    </span>
                  </>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      <p className="mt-3 text-[10px] leading-tight text-on-surface-muted">
        Fan SFP forfaitair 0,125 W/(m³/h) per NTA 8800 tab 11.23
        (engineering-aanname, geen norm-waarde — backend gebruikt deze
        constante voor TO-juli electrische ventilator-energie).
      </p>
      <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
        Laat de m³/h-velden leeg om de backend default (NTA 8800 tabel 11.23
        `default_ach` lookup op basis van GO en systeemtype) te gebruiken.
        Vul ze in om met gemeten/ontworpen debieten te rekenen.
      </p>
    </Card>
  );
}
