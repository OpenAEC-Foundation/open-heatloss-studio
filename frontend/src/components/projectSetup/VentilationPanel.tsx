/**
 * VentilationPanel — ventilatie-instellingen op project-niveau (ISSO 51).
 *
 * Schrijft naar:
 *  - V1 `project.ventilation.system_type` (ISSO 51 systeem A–E, direct)
 *  - V1 `project.ventilation.has_heat_recovery` + `heat_recovery_efficiency`
 *
 * V1 blijft single-source-of-truth voor de ISSO 51 backend; de gebruiker
 * kiest hier rechtstreeks het ISSO 51 ventilatiesysteem (A t/m E). De
 * V1→V2 mapping voor de NTA 8800 / TO-juli backend gebeurt in
 * `projectV2Migration.ts` (`mapV1SystemTypeToV2`) bij `buildV2Payload`.
 *
 * De NTA 8800 / TO-juli m³/h-debieten (infiltratie + mechanisch toe/afvoer)
 * staan NIET hier — die voeden uitsluitend de TO-juli-engine en worden op
 * de TO-juli-tab ingevoerd (`pages/TojuliFull.tsx`). Dit paneel is puur
 * ISSO 51. De m³/h-velden landen nog steeds in de `sharedExtra`-sidecar.
 *
 * Lessons learned:
 *  - 10-04 (water θ): norm-afwijkende aannames structureel tonen.
 *  - 17-05 (light theme): gebruik `--oaec-*` tokens, geen hardcoded kleuren.
 *  - 21-05 (regressie b546610): systeemkeuze terug naar ISSO 51 A–E;
 *    V2-kind-select downgrade systeem E stil naar D.
 *  - 21-05 (werkpakket A): m³/h-debieten verplaatst naar de TO-juli-tab —
 *    ze voeden alleen de NTA 8800-engine, niet de ISSO 51-berekening.
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
 * Per ISSO 51 systeemtype: of het WTW-veld zichtbaar is. Consistent met de
 * oude `WarmteverliesInstellingen.tsx` vóór b546610: `supportsWtw =
 * system_d || system_e`. `hasSupply`/`hasExhaust` blijven gedefinieerd zodat
 * de TO-juli-tab dezelfde capability-tabel kan spiegelen voor de m³/h-velden.
 *
 *   A — natuurlijke toe- en afvoer            → geen WTW
 *   B — mechanische toevoer, natuurlijke afvoer → geen WTW
 *   C — natuurlijke toevoer, mechanische afvoer → geen WTW
 *   D — gebalanceerd mechanisch (centraal)     → WTW
 *   E — combinatie (decentraal gebalanceerd)   → WTW
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
  const { project, updateProject } = useProjectStore();

  const v1Vent = project.ventilation;
  const currentSystem: VentilationSystemType = v1Vent.system_type;
  const { hasWtw } = SYSTEM_CAPABILITIES[currentSystem];

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
    <Card title="Ventilatie (ISSO 51)">
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
        ISSO 51 ventilatie — systeemtype A–E + WTW-rendement. De NTA 8800 /
        TO-juli luchtdebieten (m³/h) staan op de TO-juli-tab.
      </p>
    </Card>
  );
}
