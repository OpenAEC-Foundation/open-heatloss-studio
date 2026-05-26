/**
 * TO-juli — NTA 8800 H.10 volledige berekening (woning + utiliteit).
 *
 * Werkt op het huidige `projectStore` project: shared (gebouwtype, A_g)
 * + geometry (rooms/constructions/openings) wordt automatisch omgezet
 * naar NTA 8800-model via de Rust `compute_tojuli_full` orchestrator.
 *
 * Specifieke TO-juli inputs op deze pagina: cooling system + COP +
 * distributie/emissie-rendementen + zon-schaduw + setpoints.
 *
 * V1 stub-pijler in backend (zie `openaec-project-shared::tojuli`):
 * transmissie + ventilatie worden uit Σ A·U + ach×volume gesynthesizeerd.
 * F7.2 wisselt dit uit naar `nta8800-transmission` + `nta8800-ventilation`.
 */
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { useProjectStore } from "../store/projectStore";
import { buildV2Payload } from "../lib/projectV2Migration";
import { tojuliCalculate } from "../lib/backend";
import { VENTILATION_SYSTEM_LABELS } from "../lib/constants";
import {
  MANUAL_PRODUCT_ID,
  findCoolingUnit,
  getCoolingUnits,
} from "../lib/productCatalog";
import type { VentilationSystemType } from "../types";
import type { ProjectV2 } from "../types/projectV2";

// ---------------------------------------------------------------------------
// Ventilatie — m³/h-debieten (NTA 8800 §11.2). Zichtbaarheid van de
// mechanische toe-/afvoer-velden volgt het ISSO 51 systeemtype A–E, dat op de
// Warmteverlies-tab wordt gekozen (`project.ventilation.system_type`) en hier
// alleen read-only wordt getoond. Tabel spiegelt `SYSTEM_CAPABILITIES` in
// `components/projectSetup/VentilationPanel.tsx`.
// ---------------------------------------------------------------------------

const SYSTEM_FLOW_CAPABILITIES: Record<
  VentilationSystemType,
  { hasSupply: boolean; hasExhaust: boolean }
> = {
  system_a: { hasSupply: false, hasExhaust: false },
  system_b: { hasSupply: true, hasExhaust: false },
  system_c: { hasSupply: false, hasExhaust: true },
  system_d: { hasSupply: true, hasExhaust: true },
  system_e: { hasSupply: true, hasExhaust: true },
};

// ---------------------------------------------------------------------------
// Type-mirrors van Rust structs (openaec-project-shared::tojuli)
// ---------------------------------------------------------------------------

type CoolingSystemKind = "compression_cooling" | "absorption_cooling" | "free_cooling";

interface CoolingSystem {
  type: CoolingSystemKind;
  scop_cooling?: number;
  cop?: number;
  factor?: number;
}

interface CoolingDistribution {
  efficiency: number;
}

interface CoolingEmission {
  efficiency: number;
  regulation_factor: number;
}

interface TojuliFullInputs {
  system: CoolingSystem;
  distribution: CoolingDistribution;
  emission: CoolingEmission;
  shading_factor: number;
  heating_setpoint_c: number;
  cooling_setpoint_c: number;
}

interface MonthlyProfile<T = number> {
  values: T[];
}

interface TojuliResult {
  monthly_q_c_nd_mj: MonthlyProfile;
  monthly_q_c_use_mj: MonthlyProfile;
  annual_q_c_use_mj: number;
  annual_q_c_use_kwh: number;
  monthly_q_h_nd_mj: MonthlyProfile;
  transmission_h_t_w_per_k: number;
  ventilation_h_v_w_per_k: number;
  monthly_theta_e_c: MonthlyProfile;
  tau_hours: number;
}

const MONTH_LABELS = [
  "Jan", "Feb", "Mrt", "Apr", "Mei", "Jun",
  "Jul", "Aug", "Sep", "Okt", "Nov", "Dec",
];

const DEFAULT_INPUTS: TojuliFullInputs = {
  system: { type: "compression_cooling", scop_cooling: 3.5 },
  distribution: { efficiency: 0.95 },
  emission: { efficiency: 0.95, regulation_factor: 0.95 },
  shading_factor: 1.0,
  heating_setpoint_c: 20,
  cooling_setpoint_c: 24,
};

// ---------------------------------------------------------------------------
// BCRG koelunit-productselector (feature D)
// ---------------------------------------------------------------------------

/**
 * Dropdown-opties voor de koelunit-selector: "Handmatig invoeren" (sentinel)
 * gevolgd door de BCRG-catalogus-units. Statische lijst — buiten de component.
 */
const COOLING_PRODUCT_OPTIONS: Array<{ value: string; label: string }> = [
  { value: MANUAL_PRODUCT_ID, label: "Handmatig invoeren" },
  ...getCoolingUnits().map((u) => ({
    value: u.id,
    label: `${u.brand} ${u.model}`,
  })),
];

/**
 * Vertaal een catalogus-koelunit naar een `CoolingSystem`-payload. Vult per
 * type het juiste prestatieveld; valt terug op de bestaande TojuliFull-
 * defaults wanneer de catalogus-unit dat veld niet bevat.
 */
function coolingSystemFromCatalog(id: string): CoolingSystem | null {
  const unit = findCoolingUnit(id);
  if (!unit) return null;
  if (unit.type === "compression_cooling") {
    return { type: unit.type, scop_cooling: unit.scop_cooling ?? 3.5 };
  }
  if (unit.type === "absorption_cooling") {
    return { type: unit.type, cop: unit.cop ?? 0.8 };
  }
  return { type: unit.type, factor: unit.factor ?? 0.3 };
}

function numVal(v: string): number {
  return v === "" ? 0 : Number(v);
}

interface NumberFieldProps {
  label: string;
  unit: string;
  value: number;
  step?: number | string;
  onChange: (v: number) => void;
  hint?: string;
}

function NumberField({ label, unit, value, step, onChange, hint }: NumberFieldProps) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="font-medium text-on-surface">
        {label}{" "}
        <span className="text-on-surface-muted font-normal">[{unit}]</span>
      </span>
      <input
        type="number"
        step={step ?? "any"}
        value={Number.isFinite(value) ? value : ""}
        onChange={(e) => onChange(numVal(e.target.value))}
        className="rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:outline-none focus:ring-1 focus:border-primary focus:ring-primary"
      />
      {hint && <span className="text-xs text-on-surface-muted">{hint}</span>}
    </label>
  );
}

interface FlowFieldProps {
  label: string;
  unit: string;
  /** Lege string = niet ingevuld → backend rekent zelf. */
  value: number | string;
  step?: number | string;
  placeholder?: string;
  onChange: (raw: string) => void;
  hint?: string;
}

/**
 * Optioneel debiet-veld dat de leeg/placeholder-semantiek behoudt: anders dan
 * `NumberField` mag de waarde leeg blijven (geen forced `0`), zodat de backend
 * op de NTA 8800-default kan terugvallen. Styling spiegelt `NumberField`.
 */
function FlowField({
  label,
  unit,
  value,
  step,
  placeholder,
  onChange,
  hint,
}: FlowFieldProps) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="font-medium text-on-surface">
        {label}{" "}
        <span className="text-on-surface-muted font-normal">[{unit}]</span>
      </span>
      <input
        type="number"
        step={step ?? "any"}
        min={0}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:outline-none focus:ring-1 focus:border-primary focus:ring-primary"
      />
      {hint && <span className="text-xs text-on-surface-muted">{hint}</span>}
    </label>
  );
}

export function TojuliFull() {
  const { t } = useTranslation();
  const { project, sharedExtra, updateSharedExtra } = useProjectStore();
  const [inputs, setInputs] = useState<TojuliFullInputs>(DEFAULT_INPUTS);
  const [result, setResult] = useState<TojuliResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // BCRG koelunit-keuze — puur UI-lokaal; de berekening leest enkel
  // `inputs.system`. `MANUAL_PRODUCT_ID` = vrije invoer behouden.
  const [selectedCoolingId, setSelectedCoolingId] =
    useState<string>(MANUAL_PRODUCT_ID);

  // Ventilatie — ISSO 51 systeemtype (Warmteverlies-tab) bepaalt welke
  // mechanische debiet-velden zinvol zijn. Bij ontbrekend systeem valt de
  // backend forfaitair terug op systeem C (engineering-aanname, niet uit
  // NTA 8800 — zie QC-review b546610 bevinding 3).
  const ventilationSystem: VentilationSystemType | undefined =
    project.ventilation?.system_type;
  const ventilationSystemLabel = ventilationSystem
    ? VENTILATION_SYSTEM_LABELS[ventilationSystem] ?? ventilationSystem
    : "Niet opgegeven";
  const { hasSupply, hasExhaust } = ventilationSystem
    ? SYSTEM_FLOW_CAPABILITIES[ventilationSystem]
    : { hasSupply: true, hasExhaust: true };

  // m³/h-debieten lezen/schrijven via de sharedExtra-sidecar. Leeg = backend
  // rekent zelf (NTA 8800-default). Sidecar-keys ongewijzigd t.o.v. b546610.
  const infiltrationFlow = sharedExtra.infiltration_m3_per_h ?? "";
  const supplyFlow = sharedExtra.mechanical_supply_m3_per_h ?? "";
  const exhaustFlow = sharedExtra.mechanical_exhaust_m3_per_h ?? "";

  const writeFlowExtra = useCallback(
    (
      key:
        | "infiltration_m3_per_h"
        | "mechanical_supply_m3_per_h"
        | "mechanical_exhaust_m3_per_h",
      raw: string,
    ) => {
      if (raw === "") {
        updateSharedExtra({ [key]: null });
        return;
      }
      const n = Number(raw);
      if (!Number.isFinite(n) || n < 0) return;
      updateSharedExtra({ [key]: n });
    },
    [updateSharedExtra],
  );

  // Bouw huidige V1 Project + sharedExtra naar ProjectV2 voor de backend call.
  // V1-rooms worden door buildV2Payload naar geometry.spaces[] gemapt
  // (F6.2). Layers/openings blijven leeg — TO-juli H_T leest alleen
  // area * u_value per boundary.
  const projectV2: ProjectV2 = useMemo(
    () => buildV2Payload(project, sharedExtra),
    [project, sharedExtra],
  );

  const setField = useCallback(
    <K extends keyof TojuliFullInputs>(key: K, value: TojuliFullInputs[K]) => {
      setInputs((prev) => ({ ...prev, [key]: value }));
    },
    [],
  );

  // Handmatige wijziging van type/SCOP/COP → catalogus-herkomst klopt niet
  // meer; selector terug naar "Handmatig invoeren".
  const setSystem = useCallback((sys: CoolingSystem) => {
    setSelectedCoolingId(MANUAL_PRODUCT_ID);
    setInputs((prev) => ({ ...prev, system: sys }));
  }, []);

  // BCRG-productselector: een catalogus-keuze zet `system.type` +
  // `scop_cooling`/`cop`/`factor` (geen parallelle state).
  const selectCoolingProduct = useCallback((id: string) => {
    setSelectedCoolingId(id);
    if (id === MANUAL_PRODUCT_ID) return;
    const sys = coolingSystemFromCatalog(id);
    if (!sys) return;
    setInputs((prev) => ({ ...prev, system: sys }));
  }, []);

  const selectedCoolingUnit =
    selectedCoolingId === MANUAL_PRODUCT_ID
      ? undefined
      : findCoolingUnit(selectedCoolingId);

  const handleCalculate = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const r = await tojuliCalculate<
        { project: ProjectV2; inputs: TojuliFullInputs },
        TojuliResult
      >({ project: projectV2, inputs });
      setResult(r);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setResult(null);
    } finally {
      setBusy(false);
    }
  }, [projectV2, inputs]);

  const handleReset = useCallback(() => {
    setInputs(DEFAULT_INPUTS);
    setSelectedCoolingId(MANUAL_PRODUCT_ID);
    setResult(null);
    setError(null);
  }, []);

  const monthlyValues = (mp: MonthlyProfile<number> | undefined): number[] => {
    if (!mp) return new Array(12).fill(0);
    // Rust serialiseert MonthlyProfile als `{ values: [...] }` of als plain array
    // afhankelijk van de versie; we ondersteunen beide.
    const v = (mp as unknown as { values?: number[] }).values;
    if (Array.isArray(v)) return v;
    if (Array.isArray(mp)) return mp as unknown as number[];
    return new Array(12).fill(0);
  };

  return (
    <div>
      <PageHeader
        title={t("tojuliFull.title", "TO-juli — volledige H.10 berekening")}
        subtitle={t(
          "tojuliFull.subtitle",
          "NTA 8800 §10 koeling met maandelijkse Q_C;use voor woning én utiliteit",
        )}
        breadcrumbs={[{ label: t("tojuliFull.title", "TO-juli") }]}
        actions={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={handleReset}>
              {t("tojuli.reset", "Standaardwaarden")}
            </Button>
            <Button onClick={handleCalculate} disabled={busy}>
              {busy
                ? t("tojuli.calculating", "Bezig…")
                : t("tojuli.calculate", "Bereken")}
            </Button>
          </div>
        }
      />

      <div className="space-y-4 p-6">
        {error && (
          <div className="rounded-md border border-red-600/30 bg-red-600/15 px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <Card title={t("tojuliFull.contextTitle", "Project-context (read-only)")}>
          <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
            <ContextRow label="Projectnaam" value={projectV2.shared.name} />
            <ContextRow
              label="Gebouwtype"
              value={`${projectV2.shared.building_type.kind} / ${projectV2.shared.building_type.subtype}`}
            />
            <ContextRow
              label="A_g"
              value={`${projectV2.shared.gross_floor_area_m2?.toFixed(1) ?? "—"} m²`}
            />
            <ContextRow
              label="Spaces / Constructies"
              value={`${projectV2.geometry.spaces.length} / ${
                projectV2.geometry.spaces.reduce(
                  (n, s) => n + (s.constructions?.length ?? 0),
                  0,
                )
              }`}
            />
            <ContextRow
              label="Ventilatiesysteem"
              value={ventilationSystemLabel}
            />
          </div>
          <p className="mt-3 text-xs text-on-surface-muted">
            {t(
              "tojuliFull.contextHint",
              "Vul shared/geometrie in via tab Algemeen + Modeller. Wijzigingen worden hier direct meegenomen.",
            )}
          </p>
          {!ventilationSystem && (
            <p className="mt-1 text-xs text-on-surface-muted">
              {t(
                "tojuliFull.ventilationFallbackHint",
                "Geen ventilatiesysteem opgegeven — de TO-juli-engine neemt forfaitair systeem C aan (engineering-aanname, niet uit NTA 8800). Kies het systeem op de Warmteverlies-tab.",
              )}
            </p>
          )}
        </Card>

        <Card title={t("tojuliFull.systemTitle", "Koelopwekker")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("tojuliFull.fields.cooling_product", "Koelunit (BCRG)")}
              </span>
              <select
                value={selectedCoolingId}
                onChange={(e) => selectCoolingProduct(e.target.value)}
                className="rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface"
              >
                {COOLING_PRODUCT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
              <span className="text-xs text-on-surface-muted">
                {t(
                  "tojuliFull.fields.coolingProductHint",
                  "Kies een BCRG-unit of 'Handmatig invoeren'.",
                )}
              </span>
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("tojuliFull.fields.system_type", "Type")}
              </span>
              <select
                value={inputs.system.type}
                onChange={(e) => {
                  const kind = e.target.value as CoolingSystemKind;
                  if (kind === "compression_cooling") {
                    setSystem({ type: kind, scop_cooling: inputs.system.scop_cooling ?? 3.5 });
                  } else if (kind === "absorption_cooling") {
                    setSystem({ type: kind, cop: inputs.system.cop ?? 0.8 });
                  } else {
                    setSystem({ type: kind, factor: inputs.system.factor ?? 0.3 });
                  }
                }}
                className="rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface"
              >
                <option value="compression_cooling">
                  {t("tojuliFull.system.compression", "Compressiekoeling")}
                </option>
                <option value="absorption_cooling">
                  {t("tojuliFull.system.absorption", "Absorptiekoeling")}
                </option>
                <option value="free_cooling">
                  {t("tojuliFull.system.free", "Vrije koeling")}
                </option>
              </select>
            </label>

            {inputs.system.type === "compression_cooling" && (
              <NumberField
                label={t("tojuliFull.fields.scop", "SCOP koeling")}
                unit="—"
                step={0.1}
                value={inputs.system.scop_cooling ?? 3.5}
                onChange={(v) => setSystem({ type: "compression_cooling", scop_cooling: v })}
                hint="Compressie: 3,0–5,0"
              />
            )}
            {inputs.system.type === "absorption_cooling" && (
              <NumberField
                label={t("tojuliFull.fields.cop", "COP")}
                unit="—"
                step={0.1}
                value={inputs.system.cop ?? 0.8}
                onChange={(v) => setSystem({ type: "absorption_cooling", cop: v })}
                hint="Absorptie: 0,6–1,3"
              />
            )}
            {inputs.system.type === "free_cooling" && (
              <NumberField
                label={t("tojuliFull.fields.factor", "Benuttingsfractie")}
                unit="0..1"
                step={0.05}
                value={inputs.system.factor ?? 0.3}
                onChange={(v) => setSystem({ type: "free_cooling", factor: v })}
                hint="Ventilatieve koeling: 0,1–0,4"
              />
            )}
          </div>
          <p className="mt-3 text-xs text-on-surface-muted">
            {selectedCoolingUnit
              ? `${selectedCoolingUnit.brand} ${selectedCoolingUnit.model} — ${
                  selectedCoolingUnit.scop_cooling != null
                    ? `SCOP=${selectedCoolingUnit.scop_cooling}`
                    : selectedCoolingUnit.cop != null
                      ? `COP=${selectedCoolingUnit.cop}`
                      : `benuttingsfractie=${selectedCoolingUnit.factor ?? "—"}`
                } (BCRG-verkl. nr. ${selectedCoolingUnit.bcrg_declaration_nr || "—"})`
              : "(handmatige invoer)"}
          </p>
        </Card>

        <Card title={t("tojuliFull.systemAuxTitle", "Distributie + emissie")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <NumberField
              label="η_dist;C"
              unit="0..1"
              step={0.01}
              value={inputs.distribution.efficiency}
              onChange={(v) => setField("distribution", { efficiency: v })}
            />
            <NumberField
              label="η_em;C"
              unit="0..1"
              step={0.01}
              value={inputs.emission.efficiency}
              onChange={(v) =>
                setField("emission", { ...inputs.emission, efficiency: v })
              }
            />
            <NumberField
              label="f_reg"
              unit="0..1"
              step={0.01}
              value={inputs.emission.regulation_factor}
              onChange={(v) =>
                setField("emission", { ...inputs.emission, regulation_factor: v })
              }
            />
          </div>
        </Card>

        <Card title={t("tojuliFull.ventilationTitle", "Ventilatie (NTA 8800)")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <FlowField
              label={t("tojuliFull.fields.infiltration", "Basisinfiltratie")}
              unit="m³/h"
              step={1}
              placeholder="Auto (NTA 8800)"
              value={infiltrationFlow}
              onChange={(raw) => writeFlowExtra("infiltration_m3_per_h", raw)}
              hint={t(
                "tojuliFull.fields.infiltrationHint",
                "Leeg laten = backend rekent zelf (NTA 8800-default).",
              )}
            />
            {hasSupply && (
              <FlowField
                label={t("tojuliFull.fields.mechSupply", "Mechanische toevoer")}
                unit="m³/h"
                step={1}
                placeholder="Auto (NTA 8800)"
                value={supplyFlow}
                onChange={(raw) =>
                  writeFlowExtra("mechanical_supply_m3_per_h", raw)
                }
                hint={t(
                  "tojuliFull.fields.flowHint",
                  "Leeg laten = NTA 8800-default.",
                )}
              />
            )}
            {hasExhaust && (
              <FlowField
                label={t("tojuliFull.fields.mechExhaust", "Mechanische afvoer")}
                unit="m³/h"
                step={1}
                placeholder="Auto (NTA 8800)"
                value={exhaustFlow}
                onChange={(raw) =>
                  writeFlowExtra("mechanical_exhaust_m3_per_h", raw)
                }
                hint={t(
                  "tojuliFull.fields.flowHint",
                  "Leeg laten = NTA 8800-default.",
                )}
              />
            )}
          </div>
          <p className="mt-3 text-xs text-on-surface-muted">
            {t(
              "tojuliFull.ventilationHint",
              "NTA 8800 §11.2 luchtdebieten — voeden de TO-juli-engine. De zichtbare mechanische velden volgen het ISSO 51-systeemtype (Warmteverlies-tab). Leeg = backend-default.",
            )}
          </p>
        </Card>

        <Card title={t("tojuliFull.advancedTitle", "Geavanceerd")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <NumberField
              label={t("tojuliFull.fields.shading", "Schaduwfactor F_sh")}
              unit="0..1"
              step={0.05}
              value={inputs.shading_factor}
              onChange={(v) => setField("shading_factor", v)}
              hint="1.0 = geen schaduw"
            />
            <NumberField
              label={t("tojuliFull.fields.heating_sp", "Verwarmings-setpoint")}
              unit="°C"
              step={0.5}
              value={inputs.heating_setpoint_c}
              onChange={(v) => setField("heating_setpoint_c", v)}
            />
            <NumberField
              label={t("tojuliFull.fields.cooling_sp", "Koel-setpoint")}
              unit="°C"
              step={0.5}
              value={inputs.cooling_setpoint_c}
              onChange={(v) => setField("cooling_setpoint_c", v)}
            />
          </div>
        </Card>

        {result && (
          <Card title={t("tojuliFull.resultTitle", "Resultaat")}>
            <div className="space-y-4">
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                <ResultRow
                  label="Q_C;use jaarsom"
                  value={`${result.annual_q_c_use_kwh.toFixed(0)} kWh/jaar`}
                  highlight
                />
                <ResultRow
                  label="H_T"
                  value={`${result.transmission_h_t_w_per_k.toFixed(1)} W/K`}
                />
                <ResultRow
                  label="H_V"
                  value={`${result.ventilation_h_v_w_per_k.toFixed(1)} W/K`}
                />
                <ResultRow
                  label="τ (tijdconstante)"
                  value={`${result.tau_hours.toFixed(1)} h`}
                />
                <ResultRow
                  label="Q_C;nd (jaarsom)"
                  value={`${(
                    monthlyValues(result.monthly_q_c_nd_mj).reduce((a, b) => a + b, 0) / 3.6
                  ).toFixed(0)} kWh/jaar`}
                />
                <ResultRow
                  label="Q_H;nd (jaarsom)"
                  value={`${(
                    monthlyValues(result.monthly_q_h_nd_mj).reduce((a, b) => a + b, 0) / 3.6
                  ).toFixed(0)} kWh/jaar`}
                />
              </div>

              <div className="border-t border-[var(--oaec-border-subtle)] pt-3">
                <h3 className="mb-2 text-sm font-semibold text-on-surface">
                  {t("tojuliFull.monthly", "Maandelijks (MJ)")}
                </h3>
                <table className="w-full text-xs">
                  <thead>
                    <tr className="border-b border-[var(--oaec-border)]">
                      <th className="px-2 py-1 text-left">Maand</th>
                      {MONTH_LABELS.map((m) => (
                        <th key={m} className="px-2 py-1 text-right">{m}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    <MonthlyRow label="θ_e (°C)" values={monthlyValues(result.monthly_theta_e_c)} digits={1} />
                    <MonthlyRow label="Q_C;nd" values={monthlyValues(result.monthly_q_c_nd_mj)} digits={0} />
                    <MonthlyRow label="Q_C;use" values={monthlyValues(result.monthly_q_c_use_mj)} digits={0} bold />
                    <MonthlyRow label="Q_H;nd" values={monthlyValues(result.monthly_q_h_nd_mj)} digits={0} />
                  </tbody>
                </table>
              </div>

              <p className="text-xs text-on-surface-muted">
                {t(
                  "tojuliFull.normRef",
                  "Berekend volgens NTA 8800:2025+C1:2026 hoofdstukken 7 en 10 met volledige nta8800-transmission + nta8800-ventilation pipeline (drukmodel C2.3, §11.2.1 massabalans).",
                )}
              </p>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

function ContextRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-xs text-on-surface-muted">{label}</div>
      <div className="font-medium text-on-surface">{value}</div>
    </div>
  );
}

function ResultRow({
  label,
  value,
  highlight,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div
      className={`rounded-md border px-4 py-3 ${
        highlight
          ? "border-primary/40 bg-primary/10"
          : "border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-subtle)]"
      }`}
    >
      <div className="text-xs text-on-surface-muted">{label}</div>
      <div
        className={`text-lg font-semibold ${
          highlight ? "text-primary" : "text-on-surface"
        }`}
      >
        {value}
      </div>
    </div>
  );
}

function MonthlyRow({
  label,
  values,
  digits,
  bold,
}: {
  label: string;
  values: number[];
  digits: number;
  bold?: boolean;
}) {
  return (
    <tr className={`border-b border-[var(--oaec-border-subtle)] ${bold ? "font-semibold" : ""}`}>
      <td className="px-2 py-1 text-on-surface">{label}</td>
      {values.map((v, i) => (
        <td key={i} className="px-2 py-1 text-right tabular-nums">
          {Number.isFinite(v) ? v.toFixed(digits) : "—"}
        </td>
      ))}
    </tr>
  );
}
