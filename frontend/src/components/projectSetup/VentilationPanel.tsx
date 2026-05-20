/**
 * VentilationPanel — V2 ventilatie-instellingen op project-niveau.
 *
 * Schrijft naar:
 *  - V1 `project.ventilation.system_type` (via inverse mapping van V2 kind)
 *  - V1 `project.ventilation.has_heat_recovery` + `heat_recovery_efficiency`
 *  - V2 sidecar `sharedExtra.{infiltration,mechanical_supply,mechanical_exhaust}_m3_per_h`
 *
 * V1 blijft single-source-of-truth voor de ISSO 51 backend; de V2-only
 * m³/h velden landen via `buildV2Payload` in de SharedProject ventilation
 * surface (NTA 8800 / TO-juli) — en blijven `undefined` als de gebruiker
 * leeg laat, zodat de Rust engine terugvalt op `default_ach` lookup.
 *
 * Lessons learned:
 *  - 10-04 (water θ): norm-afwijkende aannames structureel tonen → SFP
 *    forfaitair-voetnoot is altijd zichtbaar onder de invoer.
 *  - 17-05 (light theme): gebruik `--oaec-*` tokens, geen hardcoded kleuren.
 */
import { useCallback } from "react";

import { Card } from "../ui/Card";
import { Input } from "../ui/Input";
import { Select } from "../ui/Select";
import { useProjectStore } from "../../store/projectStore";
import type { VentilationConfig, VentilationSystemType } from "../../types";
import type {
  HeatRecovery,
  VentilationSystemKind,
} from "../../types/projectV2";

// ---------------------------------------------------------------------------
// V1 ↔ V2 ventilatiesysteem mapping
// ---------------------------------------------------------------------------

/**
 * V2 → V1 inverse mapping. V1 systemen A–E komen uit ISSO 51; voor V2
 * (NTA 8800 / TO-juli) gebruiken we de 4-tal categorisering.
 *   mech_balanced → system_d (gebalanceerd mechanisch — closest fit)
 *   mech_supply   → system_b (mechanische toevoer, natuurlijke afvoer)
 *   mech_exhaust  → system_c (mechanische afvoer, natuurlijke toevoer)
 *   natural       → system_a (natuurlijke toe- en afvoer)
 */
function v2ToV1SystemType(v2: VentilationSystemKind): VentilationSystemType {
  switch (v2) {
    case "mech_balanced":
      return "system_d";
    case "mech_supply":
      return "system_b";
    case "mech_exhaust":
      return "system_c";
    case "natural":
      return "system_a";
  }
}

/** V1 → V2 mapping voor uitlezen. Spiegelt `mapV1SystemTypeToV2` in projectV2Migration.ts. */
function v1ToV2SystemType(v1: VentilationSystemType): VentilationSystemKind {
  switch (v1) {
    case "system_a":
      return "natural";
    case "system_b":
      return "mech_supply";
    case "system_c":
      return "mech_exhaust";
    case "system_d":
      return "mech_balanced";
    case "system_e":
      return "mech_balanced"; // decentraal gebalanceerd — closest V2 match
  }
}

const V2_SYSTEM_OPTIONS: Array<{ value: VentilationSystemKind; label: string }> = [
  { value: "mech_balanced", label: "Mechanisch gebalanceerd (toe + afvoer)" },
  { value: "mech_supply", label: "Mechanische toevoer (natuurlijke afvoer)" },
  { value: "mech_exhaust", label: "Mechanische afvoer (natuurlijke toevoer)" },
  { value: "natural", label: "Natuurlijke ventilatie" },
];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function VentilationPanel() {
  const { project, updateProject, sharedExtra, updateSharedExtra } =
    useProjectStore();

  const v1Vent = project.ventilation;
  const currentSystem: VentilationSystemKind = v1ToV2SystemType(v1Vent.system_type);

  const hasSupply =
    currentSystem === "mech_balanced" || currentSystem === "mech_supply";
  const hasExhaust =
    currentSystem === "mech_balanced" || currentSystem === "mech_exhaust";
  const hasHeatRecoveryUi = currentSystem === "mech_balanced";

  const updateVentilation = useCallback(
    (partial: Partial<VentilationConfig>) => {
      updateProject({ ventilation: { ...project.ventilation, ...partial } });
    },
    [project.ventilation, updateProject],
  );

  const handleSystemChange = useCallback(
    (next: VentilationSystemKind) => {
      const v1Next = v2ToV1SystemType(next);
      // WTW alleen behouden bij mech_balanced (V1 systeem D/E). Anders wissen.
      const keepWtw = next === "mech_balanced";
      updateVentilation({
        system_type: v1Next,
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

  // Toon hint over hoe V2-velden zich verhouden tot V1 ISSO 51 ventilatie.
  const heatRecoveryHint: HeatRecovery | null =
    hasHeatRecoveryUi && v1Vent.has_heat_recovery && v1Vent.heat_recovery_efficiency != null
      ? {
          efficiency: v1Vent.heat_recovery_efficiency,
          frost_protection:
            v1Vent.frost_protection != null && v1Vent.frost_protection !== "unknown",
          ...(v1Vent.supply_temperature != null
            ? { supply_temperature: v1Vent.supply_temperature }
            : {}),
        }
      : null;

  return (
    <Card title="Ventilatie (TO-juli / NTA 8800)">
      <div className="grid grid-cols-3 gap-4">
        <Select
          id="ventilation_system_v2"
          label="Ventilatiesysteem"
          value={currentSystem}
          options={V2_SYSTEM_OPTIONS}
          onChange={(e) =>
            handleSystemChange(e.target.value as VentilationSystemKind)
          }
        />
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
        {hasSupply && (
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
              writeNumericExtra("mechanical_supply_m3_per_h", e.target.value)
            }
          />
        )}
        {hasExhaust && (
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
              writeNumericExtra("mechanical_exhaust_m3_per_h", e.target.value)
            }
          />
        )}
      </div>

      {hasHeatRecoveryUi && (
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
                    <span className="ml-2 text-on-surface-muted">θ_toevoer: </span>
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
