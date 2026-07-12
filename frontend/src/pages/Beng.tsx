/**
 * BENG — NTA 8800 energieprestatie (BENG 1/2/3 + TOjuli + energielabel).
 *
 * Werkt op het huidige `projectStore` project: shared (gebouwtype, A_g) +
 * geometry (rooms/constructions) wordt via `buildV2Payload` naar een
 * `ProjectV2` gemapt; het installatie-/opwek-invoerblok (`energy`) leeft
 * additief in de store (`projectStore.energy`) en wordt hier bewerkt. De
 * volledige BENG-keten draait in de Rust-backend (`compute_beng`).
 *
 * Anders dan de TO-juli-tab kent BENG géén los `inputs`-blok — alle invoer
 * zit in `project.energy`. Raam-zonwering/belemmering per raam is
 * modeller-territorium en loopt NIET via dit paneel.
 */
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { useProjectStore } from "../store/projectStore";
import { buildV2Payload } from "../lib/projectV2Migration";
import { bengCalculate, BengInputError } from "../lib/bengClient";
import type {
  AutomationInput,
  BacsClassInput,
  BengResult,
  CoolingGeneratorType,
  CoolingInput,
  DhwGeneratorType,
  DhwInput,
  EnergyVentilationSystemType,
  HeatEmissionType,
  HeatGeneratorType,
  HeatingInput,
  HrBoilerClass,
  IndicatorReport,
  PvInput,
  VentilationInput,
} from "../types/beng";
import type { ProjectV2 } from "../types/projectV2";

// ---------------------------------------------------------------------------
// Keuzelijsten (label + serde-waarde). De serde-waarde is normatief
// (spiegel `crates/openaec-project-shared/src/energy.rs`).
// ---------------------------------------------------------------------------

const HEAT_GENERATORS: Array<{ value: HeatGeneratorType; label: string }> = [
  { value: "hr_boiler", label: "HR-ketel (gas)" },
  { value: "heat_pump_air", label: "Lucht/water-warmtepomp" },
  { value: "heat_pump_ground", label: "Bodem/water-warmtepomp" },
  { value: "electric_resistance", label: "Elektrische weerstand" },
  { value: "district_heating", label: "Stadsverwarming" },
];

const HR_CLASSES: Array<{ value: HrBoilerClass; label: string }> = [
  { value: "hr100", label: "HR-100" },
  { value: "hr104", label: "HR-104" },
  { value: "hr107", label: "HR-107" },
];

const HEAT_EMISSIONS: Array<{ value: HeatEmissionType; label: string }> = [
  { value: "radiator_high_temp", label: "Radiator HT (70–90 °C)" },
  { value: "radiator_low_temp", label: "Radiator LT (~55 °C)" },
  { value: "floor_heating", label: "Vloerverwarming (~35 °C)" },
  { value: "air_heating", label: "Luchtverwarming" },
  { value: "radiant_panel", label: "Stralingspanelen" },
];

const DHW_GENERATORS: Array<{ value: DhwGeneratorType; label: string }> = [
  { value: "hr_combi_boiler", label: "HR-combiketel (gas)" },
  { value: "electric_boiler", label: "Elektrische boiler" },
  { value: "heat_pump", label: "Tapwater-warmtepomp" },
  { value: "district_heating", label: "Stadsverwarming" },
];

const VENT_SYSTEMS: Array<{ value: EnergyVentilationSystemType; label: string }> =
  [
    { value: "A", label: "A — natuurlijk toe + afvoer" },
    { value: "B", label: "B — mech. toevoer, nat. afvoer" },
    { value: "C", label: "C — mech. afvoer, nat. toevoer" },
    { value: "D", label: "D — gebalanceerd (WTW)" },
    { value: "E", label: "E — decentraal met WTW" },
  ];

const COOLING_GENERATORS: Array<{ value: CoolingGeneratorType; label: string }> =
  [
    { value: "compression", label: "Compressiekoeling" },
    { value: "absorption", label: "Absorptiekoeling" },
    { value: "free_cooling", label: "Vrije koeling" },
  ];

const BACS_CLASSES: Array<{ value: BacsClassInput; label: string }> = [
  { value: "A", label: "A — high performance" },
  { value: "B", label: "B — advanced" },
  { value: "C", label: "C — standaard (referentie)" },
  { value: "D", label: "D — niet energiezuinig" },
];

// ---------------------------------------------------------------------------
// Kleine invoer-primitieven (styling spiegelt TojuliFull)
// ---------------------------------------------------------------------------

const INPUT_CLASS =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:outline-none focus:ring-1 focus:border-primary focus:ring-primary";

function LabeledField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="font-medium text-on-surface">{label}</span>
      {children}
      {hint && <span className="text-xs text-on-surface-muted">{hint}</span>}
    </label>
  );
}

function NumberField({
  label,
  unit,
  value,
  step,
  placeholder,
  onChange,
  hint,
}: {
  label: string;
  unit?: string;
  value: number | null | undefined;
  step?: number | string;
  placeholder?: string;
  onChange: (v: number | null) => void;
  hint?: string;
}) {
  return (
    <LabeledField
      label={unit ? `${label} [${unit}]` : label}
      hint={hint}
    >
      <input
        type="number"
        step={step ?? "any"}
        value={value ?? ""}
        placeholder={placeholder}
        onChange={(e) =>
          onChange(e.target.value === "" ? null : Number(e.target.value))
        }
        className={INPUT_CLASS}
      />
    </LabeledField>
  );
}

function SelectField<T extends string>({
  label,
  value,
  options,
  onChange,
  hint,
}: {
  label: string;
  value: T;
  options: ReadonlyArray<{ value: T; label: string }>;
  onChange: (v: T) => void;
  hint?: string;
}) {
  return (
    <LabeledField label={label} hint={hint}>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as T)}
        className={INPUT_CLASS}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </LabeledField>
  );
}

/** Card met een aan/uit-schakelaar in de titel voor een optioneel deelsysteem. */
function ToggleCard({
  title,
  enabled,
  onToggle,
  children,
}: {
  title: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  children?: React.ReactNode;
}) {
  return (
    <Card>
      <div className="flex items-center justify-between">
        <h3 className="font-heading text-sm font-medium text-on-surface">
          {title}
        </h3>
        <label className="flex items-center gap-2 text-sm text-on-surface-muted">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => onToggle(e.target.checked)}
            className="h-4 w-4 accent-[var(--oaec-primary,#6d28d9)]"
          />
          <span>{enabled ? "Actief" : "Niet aanwezig"}</span>
        </label>
      </div>
      {enabled && <div className="mt-4">{children}</div>}
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Defaults bij het inschakelen van een deelsysteem
// ---------------------------------------------------------------------------

const DEFAULT_HEATING: HeatingInput = {
  generator: "heat_pump_air",
  cop: 4.0,
  emission: "floor_heating",
  coverage_fraction: 1.0,
};
const DEFAULT_DHW: DhwInput = { generator: "heat_pump", efficiency: 2.8 };
const DEFAULT_VENT: VentilationInput = {
  system: "D",
  wtw_efficiency: 0.85,
};
const DEFAULT_COOLING: CoolingInput = { generator: "compression", seer: 4.0 };

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export function Beng() {
  const { t } = useTranslation();
  const project = useProjectStore((s) => s.project);
  const sharedExtra = useProjectStore((s) => s.sharedExtra);
  const energy = useProjectStore((s) => s.energy);
  const updateEnergy = useProjectStore((s) => s.updateEnergy);
  const setEnergy = useProjectStore((s) => s.setEnergy);

  const [result, setResult] = useState<BengResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [inputHint, setInputHint] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const projectV2: ProjectV2 = useMemo(
    () => buildV2Payload(project, sharedExtra),
    [project, sharedExtra],
  );

  const heating = energy?.heating ?? null;
  const dhw = energy?.dhw ?? null;
  // Normaliseer DWTW: `null` én afwezig betekenen "geen unit" — via `?? undefined`
  // is er geen runtime-verschil tussen de twee (zie DhwInput.dwtw-doc).
  const dwtw = dhw?.dwtw ?? undefined;
  const ventilation = energy?.ventilation ?? null;
  const cooling = energy?.cooling ?? null;
  const pv = energy?.pv ?? [];
  const automation = energy?.automation ?? null;

  const hasAnySystem =
    !!heating ||
    !!dhw ||
    !!ventilation ||
    !!cooling ||
    pv.length > 0 ||
    !!automation;

  // -- Nested-patch helpers (bootstrappen defaults bij inschakelen) --
  const patchHeating = useCallback(
    (partial: Partial<HeatingInput>) =>
      updateEnergy({ heating: { ...(heating ?? DEFAULT_HEATING), ...partial } }),
    [heating, updateEnergy],
  );
  const patchDhw = useCallback(
    (partial: Partial<DhwInput>) =>
      updateEnergy({ dhw: { ...(dhw ?? DEFAULT_DHW), ...partial } }),
    [dhw, updateEnergy],
  );
  const patchVent = useCallback(
    (partial: Partial<VentilationInput>) =>
      updateEnergy({
        ventilation: { ...(ventilation ?? DEFAULT_VENT), ...partial },
      }),
    [ventilation, updateEnergy],
  );
  const patchCooling = useCallback(
    (partial: Partial<CoolingInput>) =>
      updateEnergy({ cooling: { ...(cooling ?? DEFAULT_COOLING), ...partial } }),
    [cooling, updateEnergy],
  );

  const setPv = useCallback(
    (next: PvInput[]) => updateEnergy({ pv: next }),
    [updateEnergy],
  );

  const handleCalculate = useCallback(async () => {
    setBusy(true);
    setError(null);
    setInputHint(null);
    // Stuur `energy` alleen mee als er iets is ingevuld; anders levert de
    // backend bewust een 422 (MissingEnergyInput) → invoer-hint.
    const payload: ProjectV2 = {
      ...projectV2,
      energy: hasAnySystem ? (energy ?? undefined) : undefined,
    };
    try {
      const r = await bengCalculate({ project: payload });
      setResult(r);
    } catch (err) {
      setResult(null);
      if (err instanceof BengInputError) {
        setInputHint(
          t(
            "beng.inputHint",
            "Vul het energie-blok in (minstens één deelsysteem) en zorg dat het project een rekenzone/gebruiksoppervlak heeft.",
          ),
        );
      } else {
        setError(err instanceof Error ? err.message : String(err));
      }
    } finally {
      setBusy(false);
    }
  }, [projectV2, energy, hasAnySystem, t]);

  const handleReset = useCallback(() => {
    setEnergy(null);
    setResult(null);
    setError(null);
    setInputHint(null);
  }, [setEnergy]);

  return (
    <div>
      <PageHeader
        title={t("beng.title", "BENG — NTA 8800 energieprestatie")}
        subtitle={t(
          "beng.subtitle",
          "BENG 1/2/3 + TOjuli + energielabel op basis van shared + geometrie + installaties",
        )}
        actions={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={handleReset}>
              {t("beng.reset", "Leegmaken")}
            </Button>
            <Button onClick={handleCalculate} disabled={busy}>
              {busy
                ? t("beng.calculating", "Bezig…")
                : t("beng.calculate", "Bereken BENG")}
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
        {inputHint && (
          <div className="rounded-md border border-amber-500/30 bg-amber-500/15 px-4 py-3 text-sm text-amber-300">
            {inputHint}
          </div>
        )}

        <Card title={t("beng.contextTitle", "Project-context (read-only)")}>
          <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
            <ContextRow label="Projectnaam" value={projectV2.shared.name || "—"} />
            <ContextRow
              label="Gebouwtype"
              value={`${projectV2.shared.building_type.kind} / ${projectV2.shared.building_type.subtype}`}
            />
            <ContextRow
              label="A_g"
              value={`${projectV2.shared.gross_floor_area_m2?.toFixed(1) ?? "—"} m²`}
            />
            <ContextRow
              label="Spaces"
              value={`${projectV2.geometry.spaces.length}`}
            />
          </div>
          <p className="mt-3 text-xs text-on-surface-muted">
            {t(
              "beng.contextHint",
              "Shared + geometrie komen uit de tabs Algemeen + Modeller. Raam-zonwering/belemmering per raam loopt via de Modeller, niet via dit paneel.",
            )}
          </p>
        </Card>

        {/* -- Verwarming -- */}
        <ToggleCard
          title={t("beng.heating.title", "Verwarming (H.9)")}
          enabled={!!heating}
          onToggle={(on) => updateEnergy({ heating: on ? DEFAULT_HEATING : null })}
        >
          {heating && (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <SelectField
                label={t("beng.heating.generator", "Opwekker")}
                value={heating.generator}
                options={HEAT_GENERATORS}
                onChange={(v) => patchHeating({ generator: v })}
              />
              {(heating.generator === "heat_pump_air" ||
                heating.generator === "heat_pump_ground") && (
                <NumberField
                  label={t("beng.heating.cop", "SCOP")}
                  unit="—"
                  step={0.1}
                  value={heating.cop}
                  onChange={(v) => patchHeating({ cop: v })}
                  hint="Warmtepomp: 3,0–5,5"
                />
              )}
              {heating.generator === "hr_boiler" && (
                <SelectField
                  label={t("beng.heating.hrClass", "HR-klasse")}
                  value={heating.hr_class ?? "hr107"}
                  options={HR_CLASSES}
                  onChange={(v) => patchHeating({ hr_class: v })}
                />
              )}
              {heating.generator === "district_heating" && (
                <NumberField
                  label={t("beng.heating.districtFactor", "Grensvlak-factor")}
                  unit="0..1"
                  step={0.01}
                  value={heating.district_factor}
                  onChange={(v) => patchHeating({ district_factor: v })}
                />
              )}
              <SelectField
                label={t("beng.heating.emission", "Afgifte")}
                value={heating.emission ?? "radiator_high_temp"}
                options={HEAT_EMISSIONS}
                onChange={(v) => patchHeating({ emission: v })}
              />
            </div>
          )}
        </ToggleCard>

        {/* -- Tapwater -- */}
        <ToggleCard
          title={t("beng.dhw.title", "Warm tapwater (H.13)")}
          enabled={!!dhw}
          onToggle={(on) => updateEnergy({ dhw: on ? DEFAULT_DHW : null })}
        >
          {dhw && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                <SelectField
                  label={t("beng.dhw.generator", "Opwekker")}
                  value={dhw.generator}
                  options={DHW_GENERATORS}
                  onChange={(v) => patchDhw({ generator: v })}
                />
                <NumberField
                  label={
                    dhw.generator === "heat_pump"
                      ? t("beng.dhw.scop", "SCOP_W")
                      : t("beng.dhw.efficiency", "η_W;gen")
                  }
                  unit="—"
                  step={0.05}
                  value={dhw.efficiency}
                  onChange={(v) => patchDhw({ efficiency: v })}
                  hint="Leeg = crate-forfait per type"
                />
              </div>
              <label className="flex items-center gap-2 text-sm text-on-surface">
                <input
                  type="checkbox"
                  checked={dwtw != null}
                  onChange={(e) =>
                    patchDhw({
                      dwtw: e.target.checked ? { efficiency: 0.4 } : null,
                    })
                  }
                  className="h-4 w-4 accent-[var(--oaec-primary,#6d28d9)]"
                />
                <span>{t("beng.dhw.dwtw", "Douchewater-WTW (DWTW)")}</span>
              </label>
              {dwtw && (
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                  <NumberField
                    label={t("beng.dhw.dwtwEff", "DWTW-rendement η")}
                    unit="0..1"
                    step={0.05}
                    value={dwtw.efficiency}
                    onChange={(v) =>
                      patchDhw({
                        dwtw: { ...dwtw, efficiency: v ?? 0 },
                      })
                    }
                    hint="Typisch 0,25–0,50"
                  />
                </div>
              )}
            </div>
          )}
        </ToggleCard>

        {/* -- Ventilatie -- */}
        <ToggleCard
          title={t("beng.vent.title", "Ventilatie (H.11)")}
          enabled={!!ventilation}
          onToggle={(on) =>
            updateEnergy({ ventilation: on ? DEFAULT_VENT : null })
          }
        >
          {ventilation && (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <SelectField
                label={t("beng.vent.system", "Systeem")}
                value={ventilation.system}
                options={VENT_SYSTEMS}
                onChange={(v) => patchVent({ system: v })}
              />
              {(ventilation.system === "D" || ventilation.system === "E") && (
                <NumberField
                  label={t("beng.vent.wtw", "WTW-rendement η_hr")}
                  unit="0..1"
                  step={0.05}
                  value={ventilation.wtw_efficiency}
                  onChange={(v) => patchVent({ wtw_efficiency: v })}
                  hint="Typisch 0,75–0,95"
                />
              )}
              <NumberField
                label={t("beng.vent.sfp", "SFP")}
                unit="W/(m³/h)"
                step={0.01}
                value={ventilation.sfp_w_per_m3h}
                onChange={(v) => patchVent({ sfp_w_per_m3h: v })}
                placeholder="Auto (tab 11.23)"
                hint="Leeg = norm-forfait per systeemtype"
              />
              <NumberField
                label={t("beng.vent.supply", "Mech. toevoer")}
                unit="m³/h"
                step={1}
                value={ventilation.mechanical_supply_m3_per_h}
                onChange={(v) => patchVent({ mechanical_supply_m3_per_h: v })}
                placeholder="Auto"
              />
              <NumberField
                label={t("beng.vent.exhaust", "Mech. afvoer")}
                unit="m³/h"
                step={1}
                value={ventilation.mechanical_exhaust_m3_per_h}
                onChange={(v) => patchVent({ mechanical_exhaust_m3_per_h: v })}
                placeholder="Auto"
              />
            </div>
          )}
        </ToggleCard>

        {/* -- Koeling -- */}
        <ToggleCard
          title={t("beng.cooling.title", "Koeling (H.10)")}
          enabled={!!cooling}
          onToggle={(on) =>
            updateEnergy({ cooling: on ? DEFAULT_COOLING : null })
          }
        >
          {cooling && (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <SelectField
                label={t("beng.cooling.generator", "Opwekker")}
                value={cooling.generator}
                options={COOLING_GENERATORS}
                onChange={(v) => patchCooling({ generator: v })}
              />
              {cooling.generator === "compression" && (
                <NumberField
                  label={t("beng.cooling.seer", "SEER")}
                  unit="—"
                  step={0.1}
                  value={cooling.seer}
                  onChange={(v) => patchCooling({ seer: v })}
                  hint="Compressie: 3,0–6,0"
                />
              )}
              {cooling.generator === "absorption" && (
                <NumberField
                  label={t("beng.cooling.cop", "COP")}
                  unit="—"
                  step={0.1}
                  value={cooling.cop}
                  onChange={(v) => patchCooling({ cop: v })}
                  hint="Absorptie: 0,6–1,3"
                />
              )}
              {cooling.generator === "free_cooling" && (
                <NumberField
                  label={t("beng.cooling.freeFraction", "Benuttingsfractie")}
                  unit="0..1"
                  step={0.05}
                  value={cooling.free_cooling_fraction}
                  onChange={(v) => patchCooling({ free_cooling_fraction: v })}
                />
              )}
            </div>
          )}
        </ToggleCard>

        {/* -- PV -- */}
        <Card title={t("beng.pv.title", "PV — zonnestroom (H.16)")}>
          {pv.length === 0 && (
            <p className="text-sm text-on-surface-muted">
              {t("beng.pv.empty", "Geen PV-velden. Voeg er een toe.")}
            </p>
          )}
          <div className="space-y-3">
            {pv.map((field, idx) => (
              <div
                key={idx}
                className="grid grid-cols-1 items-end gap-3 sm:grid-cols-4"
              >
                <NumberField
                  label={t("beng.pv.kwp", "Piekvermogen")}
                  unit="kWp"
                  step={0.1}
                  value={field.peak_power_kwp}
                  onChange={(v) =>
                    setPv(
                      pv.map((p, i) =>
                        i === idx ? { ...p, peak_power_kwp: v ?? 0 } : p,
                      ),
                    )
                  }
                />
                <NumberField
                  label={t("beng.pv.azimuth", "Azimut")}
                  unit="°"
                  step={1}
                  value={field.azimuth_degrees}
                  onChange={(v) =>
                    setPv(
                      pv.map((p, i) =>
                        i === idx ? { ...p, azimuth_degrees: v ?? 0 } : p,
                      ),
                    )
                  }
                  hint="0=N, 90=O, 180=Z, 270=W"
                />
                <NumberField
                  label={t("beng.pv.tilt", "Helling")}
                  unit="°"
                  step={1}
                  value={field.tilt_degrees}
                  onChange={(v) =>
                    setPv(
                      pv.map((p, i) =>
                        i === idx ? { ...p, tilt_degrees: v ?? 0 } : p,
                      ),
                    )
                  }
                  hint="0=plat, 90=verticaal"
                />
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => setPv(pv.filter((_, i) => i !== idx))}
                >
                  {t("beng.pv.remove", "Verwijder")}
                </Button>
              </div>
            ))}
          </div>
          <div className="mt-3">
            <Button
              variant="secondary"
              size="sm"
              onClick={() =>
                setPv([
                  ...pv,
                  { peak_power_kwp: 3.5, azimuth_degrees: 180, tilt_degrees: 35 },
                ])
              }
            >
              {t("beng.pv.add", "+ PV-veld toevoegen")}
            </Button>
          </div>
        </Card>

        {/* -- BACS -- */}
        <ToggleCard
          title={t("beng.bacs.title", "Gebouwautomatisering (H.15)")}
          enabled={!!automation}
          onToggle={(on) =>
            updateEnergy({
              automation: on ? ({ bacs_class: "C" } as AutomationInput) : null,
            })
          }
        >
          {automation && (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <SelectField
                label={t("beng.bacs.class", "BACS-klasse")}
                value={automation.bacs_class}
                options={BACS_CLASSES}
                onChange={(v) => updateEnergy({ automation: { bacs_class: v } })}
              />
            </div>
          )}
        </ToggleCard>

        {/* -- Resultaat -- */}
        {result && <BengResultView result={result} t={t} />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Resultaat-weergave
// ---------------------------------------------------------------------------

function BengResultView({
  result,
  t,
}: {
  result: BengResult;
  t: (key: string, fallback: string) => string;
}) {
  return (
    <Card title={t("beng.result.title", "Resultaat")}>
      <div className="space-y-5">
        {/* Energielabel prominent */}
        <div className="flex items-center gap-4">
          <div className="rounded-md border border-primary/40 bg-primary/10 px-5 py-3">
            <div className="text-xs text-on-surface-muted">
              {t("beng.result.label", "Energielabel")}
            </div>
            <div className="text-3xl font-bold text-primary">
              {result.energy_label}
            </div>
          </div>
          <div className="grid flex-1 grid-cols-2 gap-3 text-sm sm:grid-cols-4">
            <ContextRow
              label="Hernieuwbaar"
              value={`${(result.renewable_share * 100).toFixed(0)} %`}
            />
            <ContextRow
              label="CO₂"
              value={`${result.co2_kg_per_m2.toFixed(1)} kg/m²·jr`}
            />
            <ContextRow label="A_g" value={`${result.a_g_m2.toFixed(1)} m²`} />
            <ContextRow
              label="Vormfactor A_ls/A_g"
              value={result.als_ag_ratio.toFixed(2)}
            />
          </div>
        </div>

        {/* BENG 1/2/3 */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <IndicatorCard
            label="BENG 1"
            sub={t("beng.result.beng1", "Energiebehoefte")}
            unit="kWh/m²·jr"
            report={result.beng1}
          />
          <IndicatorCard
            label="BENG 2"
            sub={t("beng.result.beng2", "Primair fossiel")}
            unit="kWh/m²·jr"
            report={result.beng2}
          />
          <IndicatorCard
            label="BENG 3"
            sub={t("beng.result.beng3", "Hernieuwbaar aandeel")}
            unit="%"
            report={result.beng3}
            higherIsBetter
          />
        </div>

        {/* TOjuli */}
        <div className="rounded-md border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-subtle)] px-4 py-3">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs text-on-surface-muted">
                {t("beng.result.tojuli", "TOjuli — oververhitting")}
              </div>
              <div className="text-lg font-semibold text-on-surface">
                {result.tojuli.max_tojuli_k.toFixed(2)} K
                <span className="text-sm font-normal text-on-surface-muted">
                  {" "}
                  / limiet {result.tojuli.limit_k.toFixed(1)} K
                </span>
              </div>
              <div className="text-xs text-on-surface-muted">
                {t("beng.result.method", "Methode")}:{" "}
                {result.tojuli.method === "actively_cooled"
                  ? t("beng.result.methodCooled", "actief gekoeld (§5.7.2)")
                  : t("beng.result.methodOrient", "per oriëntatie (formule 5.40)")}
              </div>
            </div>
            <PassBadge pass={result.tojuli.pass} />
          </div>
        </div>

        {/* Service-breakdown */}
        <div>
          <h3 className="mb-2 text-sm font-semibold text-on-surface">
            {t("beng.result.breakdown", "Primair energiegebruik per dienst")}
          </h3>
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--oaec-border)] text-on-surface-muted">
                <th className="px-2 py-1 text-left">Dienst</th>
                <th className="px-2 py-1 text-right">kWh/(m²·jr)</th>
              </tr>
            </thead>
            <tbody>
              <BreakdownRow label="Verwarming" value={result.service_breakdown_kwh_m2.heating} />
              <BreakdownRow label="Koeling" value={result.service_breakdown_kwh_m2.cooling} />
              <BreakdownRow label="Warm tapwater" value={result.service_breakdown_kwh_m2.dhw} />
              <BreakdownRow
                label="Ventilator-hulpenergie"
                value={result.service_breakdown_kwh_m2.ventilation_aux}
              />
              <BreakdownRow label="Verlichting" value={result.service_breakdown_kwh_m2.lighting} />
              <BreakdownRow label="PV (opwekking)" value={result.service_breakdown_kwh_m2.pv} />
            </tbody>
          </table>
        </div>

        {/* Notes (transparantie — aannames nooit verbergen) */}
        {result.notes.length > 0 && (
          <div className="border-t border-[var(--oaec-border-subtle)] pt-3">
            <h3 className="mb-2 text-sm font-semibold text-on-surface">
              {t("beng.result.notes", "Aannames & vereenvoudigingen")}
            </h3>
            <ul className="list-disc space-y-1 pl-5 text-xs text-on-surface-muted">
              {result.notes.map((note, i) => (
                <li key={i}>{note}</li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </Card>
  );
}

function IndicatorCard({
  label,
  sub,
  unit,
  report,
  higherIsBetter,
}: {
  label: string;
  sub: string;
  unit: string;
  report: IndicatorReport;
  higherIsBetter?: boolean;
}) {
  return (
    <div className="rounded-md border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-subtle)] px-4 py-3">
      <div className="flex items-center justify-between">
        <div className="text-sm font-semibold text-on-surface">{label}</div>
        <PassBadge pass={report.pass} />
      </div>
      <div className="text-xs text-on-surface-muted">{sub}</div>
      <div className="mt-1 text-2xl font-bold text-on-surface">
        {report.value.toFixed(1)}
        <span className="text-xs font-normal text-on-surface-muted"> {unit}</span>
      </div>
      <div className="text-xs text-on-surface-muted">
        {report.limit != null
          ? `${higherIsBetter ? "eis ≥" : "eis ≤"} ${report.limit.toFixed(1)} ${unit}`
          : "geen grenswaarde (niet-geverifieerd)"}
      </div>
    </div>
  );
}

function PassBadge({ pass }: { pass: boolean | null }) {
  if (pass == null) {
    return (
      <span className="rounded px-2 py-0.5 text-2xs font-medium uppercase tracking-wider text-on-surface-muted">
        n.v.t.
      </span>
    );
  }
  return (
    <span
      className={`rounded px-2 py-0.5 text-2xs font-medium uppercase tracking-wider ${
        pass
          ? "bg-green-600/20 text-green-400"
          : "bg-red-600/20 text-red-400"
      }`}
    >
      {pass ? "voldoet" : "voldoet niet"}
    </span>
  );
}

function BreakdownRow({ label, value }: { label: string; value: number }) {
  return (
    <tr className="border-b border-[var(--oaec-border-subtle)]">
      <td className="px-2 py-1 text-on-surface">{label}</td>
      <td className="px-2 py-1 text-right tabular-nums text-on-surface">
        {value.toFixed(1)}
      </td>
    </tr>
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
