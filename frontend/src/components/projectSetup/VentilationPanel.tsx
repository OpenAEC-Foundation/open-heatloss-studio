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
 *  - 21-05 (feature D): BCRG-WTW-productselector naast het rendement-veld;
 *    catalogus-keuze vult `heat_recovery_efficiency`, "Handmatig" behoudt
 *    de vrije invoer. Géén parallelle state — de selector-keuze is puur
 *    UI-lokaal (`selectedWtwId`), de berekening leest enkel V1.
 */
import { useCallback, useMemo, useState } from "react";

import { VENTILATION_SYSTEM_LABELS } from "../../lib/constants";
import {
  MANUAL_PRODUCT_ID,
  findWtwUnit,
  getWtwUnits,
} from "../../lib/productCatalog";
import { bblMinimumVentilationRate } from "../../lib/roomDefaults";
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
// BCRG WTW-productselector (feature D)
// ---------------------------------------------------------------------------

/**
 * WTW-catalogus, statisch geladen. De dropdown-opties (incl. de "Handmatig
 * invoeren"-sentinel én de disabled-vlag bij te kleine units) worden per
 * render samengesteld op basis van de gebouw-q_v die uit de store komt.
 */
const WTW_UNITS = getWtwUnits();

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function VentilationPanel() {
  const { project, updateProject } = useProjectStore();
  const norm = useProjectStore((s) => s.norm);

  // Catalogus-keuze is puur UI-lokaal — de berekening leest uitsluitend V1
  // `heat_recovery_efficiency`. `MANUAL_PRODUCT_ID` = vrije invoer behouden.
  const [selectedWtwId, setSelectedWtwId] = useState<string>(MANUAL_PRODUCT_ID);

  const v1Vent = project.ventilation;
  const currentSystem: VentilationSystemType = v1Vent.system_type;
  const { hasWtw } = SYSTEM_CAPABILITIES[currentSystem];

  // Benodigd ventilatiedebiet op gebouwniveau — identiek aan de Gebouwtotaal-
  // berekening op /results: per kamer de q_v (dm³/s) optellen, met BBL-
  // minimum als fallback. Conversie naar m³/h: × 3.6.
  const requiredM3h = useMemo(() => {
    let qvSum = 0;
    for (const room of project.rooms) {
      qvSum +=
        room.ventilation_rate ??
        bblMinimumVentilationRate(room.function, room.floor_area);
    }
    return qvSum * 3.6;
  }, [project.rooms]);

  // Dropdown-opties met capaciteit zichtbaar in het label en disabled-vlag
  // voor units onder de drempel. De huidige selectie blijft altijd selecteer-
  // baar (geen stilzwijgend disabled) — als de user al een te kleine unit
  // had gekozen, zien we dat verderop in een waarschuwingstekst.
  const wtwProductOptions = useMemo(
    () => [
      { value: MANUAL_PRODUCT_ID, label: "Handmatig invoeren" },
      ...WTW_UNITS.map((u) => ({
        value: u.id,
        label: `${u.brand} ${u.model} — ${u.q_nominal_m3h} m³/h — η_hr ${Math.round(
          u.eta_hr * 100,
        )}%`,
        disabled: u.q_nominal_m3h < requiredM3h && u.id !== selectedWtwId,
      })),
    ],
    [requiredM3h, selectedWtwId],
  );

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
      if (!keepWtw) {
        // WTW vervalt → catalogus-keuze terug naar handmatig (geen herkomst).
        setSelectedWtwId(MANUAL_PRODUCT_ID);
      }
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
    // Handmatige aanpassing van het rendement → de catalogus-herkomst klopt
    // niet meer; selector terug naar "Handmatig invoeren".
    setSelectedWtwId(MANUAL_PRODUCT_ID);
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

  // BCRG-productselector: een catalogus-keuze vult het bestaande V1
  // `heat_recovery_efficiency`-veld (geen parallelle state). f_SFP blijft
  // catalogus-data zonder UI-binding — er is nog geen SFP-invoerveld in V1.
  const handleWtwProductChange = (id: string) => {
    setSelectedWtwId(id);
    if (id === MANUAL_PRODUCT_ID) return;
    const unit = findWtwUnit(id);
    if (!unit) return;
    updateVentilation({
      has_heat_recovery: true,
      heat_recovery_efficiency: unit.eta_hr,
    });
  };

  const selectedWtwUnit =
    selectedWtwId === MANUAL_PRODUCT_ID ? undefined : findWtwUnit(selectedWtwId);

  // Waarschuwing wanneer de gekozen unit onder de gebouw-drempel ligt.
  const selectedUnitTooSmall =
    selectedWtwUnit != null && selectedWtwUnit.q_nominal_m3h < requiredM3h;

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
      {/* Systeemkeuze voor ISSO 53 verborgen: het ventilatiesysteem op de
          Project-tab (Isso53BuildingFields) is daar leidend. De WTW-UI
          hieronder blijft wel actief — die V1-velden gebruikt de isso53-calc. */}
      {norm !== "isso53" && (
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
      )}

      {hasWtw && (
        <div className="mt-4 grid grid-cols-3 gap-4 border-t border-[var(--oaec-border-subtle)] pt-4">
          <div className="col-span-3 -mb-2 rounded-md bg-[var(--oaec-accent-soft)] px-3 py-1.5 text-xs">
            <span className="text-on-surface-muted">
              Min. benodigde capaciteit (gebouw q_v):{" "}
            </span>
            <span className="font-semibold tabular-nums text-on-surface">
              {requiredM3h.toFixed(0)} m³/h
            </span>
          </div>
          <div>
            <Select
              id="wtw_product"
              label="WTW-unit (BCRG)"
              value={selectedWtwId}
              options={wtwProductOptions}
              onChange={(e) => handleWtwProductChange(e.target.value)}
            />
            <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
              Kies een BCRG-unit om η_hr automatisch in te vullen, of
              "Handmatig invoeren" voor een eigen waarde. Units met een
              nominale capaciteit onder de gebouw-q_v worden uitgegrijsd.
            </p>
            {selectedUnitTooSmall && (
              <p className="mt-1 text-[10px] leading-tight text-amber-600">
                ⚠ Capaciteit te laag voor dit gebouw (
                {selectedWtwUnit?.q_nominal_m3h} m³/h &lt;{" "}
                {requiredM3h.toFixed(0)} m³/h).
              </p>
            )}
          </div>
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
              {selectedWtwUnit
                ? `${selectedWtwUnit.brand} ${selectedWtwUnit.model} — η_hr=${Math.round(
                    selectedWtwUnit.eta_hr * 100,
                  )}% (BCRG-verkl. nr. ${
                    selectedWtwUnit.bcrg_declaration_nr || "—"
                  })`
                : "Het rendement (η) bepaalt de toevoertemperatuur: θ_t = θ_e + η × (θ_i − θ_e). Hoger η = warmere toevoerlucht = minder ventilatieverlies. Leeg = geen WTW; als alleen vorstbeveiliging is gekozen valt de calc terug op ISSO 51 Tabel 2.14."}
            </p>
          </div>
          {heatRecoveryHint && (
            <div className="flex items-center">
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
