/**
 * TO-juli — NTA 8800 bijlage AA vereenvoudigde koelbehoefte.
 *
 * **Status:** Expert mode / "Snelle check (woningen)" — niet norm-volledig
 * voor utiliteit. Volgens ADR-002 (`docs/ADR-002-multi-calc-project.md`) is
 * dit pad behouden onder `/tojuli/quick` als snelle woning-check; het
 * volledige TO-juli (H.10 + utiliteit + project-koppeling) komt op
 * `/tojuli` via fasering F4-F7.
 *
 * Form-driven calculator: 12 inputs in drie groepen (gebied / lucht /
 * thermische lasten) → Tauri command `simplified_cooling` → result Card.
 */
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";

/** Mirrors `SimplifiedCoolingRequest` in src-tauri/commands.rs. */
interface SimplifiedCoolingRequest {
  living_area_m2: number;
  other_area_m2: number;
  dwelling_count: number;
  persons_per_dwelling: number;
  infiltration_m3_per_h: number;
  natural_ventilation_m3_per_h: number;
  mechanical_supply_m3_per_h: number;
  peak_hour: number;
  construction_year: number;
  opaque_area_m2: number;
  solar_load_w: number;
  glazing_transmission_w: number;
}

/** Mirrors `SimplifiedCoolingResult` in nta8800-cooling. */
interface SimplifiedCoolingResult {
  minimum_capacity_w: number;
  internal_load_w: number;
  outdoor_load_w: number;
  opaque_transmission_w: number;
  solar_load_w: number;
  glazing_transmission_w: number;
  peak_cooling_load_w: number;
  maatgevende_koelbehoefte_w_per_m2: number;
}

const DEFAULT_INPUT: SimplifiedCoolingRequest = {
  living_area_m2: 80,
  other_area_m2: 40,
  dwelling_count: 1,
  persons_per_dwelling: 2.5,
  infiltration_m3_per_h: 100,
  natural_ventilation_m3_per_h: 0,
  mechanical_supply_m3_per_h: 150,
  peak_hour: 17,
  construction_year: 2020,
  opaque_area_m2: 100,
  solar_load_w: 4400,
  glazing_transmission_w: 286,
};

/**
 * Validatieregels per veld. Geen externe lib — kleine set, inline kan.
 * Faalmelding gaat naar `errors` map; submit-knop is disabled bij ≥1 error.
 */
function validate(input: SimplifiedCoolingRequest): Record<string, string> {
  const errs: Record<string, string> = {};
  if (input.living_area_m2 < 0) errs.living_area_m2 = "≥ 0";
  if (input.other_area_m2 < 0) errs.other_area_m2 = "≥ 0";
  if (input.living_area_m2 + input.other_area_m2 <= 0)
    errs.living_area_m2 = "som > 0";
  if (input.dwelling_count < 1) errs.dwelling_count = "≥ 1";
  if (input.persons_per_dwelling <= 0)
    errs.persons_per_dwelling = "> 0";
  if (input.infiltration_m3_per_h < 0)
    errs.infiltration_m3_per_h = "≥ 0";
  if (input.natural_ventilation_m3_per_h < 0)
    errs.natural_ventilation_m3_per_h = "≥ 0";
  if (input.mechanical_supply_m3_per_h < 0)
    errs.mechanical_supply_m3_per_h = "≥ 0";
  if (input.peak_hour < 9 || input.peak_hour > 21)
    errs.peak_hour = "9..21";
  if (input.construction_year < 1900 || input.construction_year > 2100)
    errs.construction_year = "1900..2100";
  if (input.opaque_area_m2 < 0) errs.opaque_area_m2 = "≥ 0";
  if (input.solar_load_w < 0) errs.solar_load_w = "≥ 0";
  if (input.glazing_transmission_w < 0)
    errs.glazing_transmission_w = "≥ 0";
  return errs;
}

interface NumberFieldProps {
  label: string;
  unit: string;
  value: number;
  step?: number;
  onChange: (v: number) => void;
  error?: string;
  hint?: string;
}

function NumberField({
  label,
  unit,
  value,
  step,
  onChange,
  error,
  hint,
}: NumberFieldProps) {
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
        onChange={(e) => {
          const n = parseFloat(e.target.value);
          onChange(Number.isFinite(n) ? n : 0);
        }}
        className={`rounded-md border bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:outline-none focus:ring-1 ${
          error
            ? "border-red-500 focus:border-red-500 focus:ring-red-500"
            : "border-[var(--oaec-border)] focus:border-primary focus:ring-primary"
        }`}
      />
      {error && <span className="text-xs text-red-400">{error}</span>}
      {hint && !error && (
        <span className="text-xs text-on-surface-muted">{hint}</span>
      )}
    </label>
  );
}

export function Tojuli() {
  const { t } = useTranslation();
  const [input, setInput] =
    useState<SimplifiedCoolingRequest>(DEFAULT_INPUT);
  const [result, setResult] = useState<SimplifiedCoolingResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const errors = validate(input);
  const hasErrors = Object.keys(errors).length > 0;

  const setField = useCallback(
    <K extends keyof SimplifiedCoolingRequest>(
      key: K,
      value: SimplifiedCoolingRequest[K],
    ) => {
      setInput((prev) => ({ ...prev, [key]: value }));
    },
    [],
  );

  const handleCalculate = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const r = await invoke<SimplifiedCoolingResult>(
        "simplified_cooling",
        { req: input },
      );
      setResult(r);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setResult(null);
    } finally {
      setBusy(false);
    }
  }, [input]);

  const handleReset = useCallback(() => {
    setInput(DEFAULT_INPUT);
    setResult(null);
    setError(null);
  }, []);

  return (
    <div>
      <PageHeader
        title={t("tojuli.title")}
        subtitle={t("tojuli.subtitle")}
        breadcrumbs={[{ label: t("tojuli.title") }]}
        actions={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={handleReset}>
              {t("tojuli.reset")}
            </Button>
            <Button onClick={handleCalculate} disabled={hasErrors || busy}>
              {busy ? t("tojuli.calculating") : t("tojuli.calculate")}
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

        <Card title={t("tojuli.groupArea")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <NumberField
              label={t("tojuli.fields.living_area_m2")}
              unit="m²"
              value={input.living_area_m2}
              onChange={(v) => setField("living_area_m2", v)}
              error={errors.living_area_m2}
              hint={t("tojuli.hints.living_area_m2")}
            />
            <NumberField
              label={t("tojuli.fields.other_area_m2")}
              unit="m²"
              value={input.other_area_m2}
              onChange={(v) => setField("other_area_m2", v)}
              error={errors.other_area_m2}
            />
            <NumberField
              label={t("tojuli.fields.dwelling_count")}
              unit="—"
              step={1}
              value={input.dwelling_count}
              onChange={(v) => setField("dwelling_count", Math.round(v))}
              error={errors.dwelling_count}
            />
            <NumberField
              label={t("tojuli.fields.persons_per_dwelling")}
              unit="—"
              step={0.1}
              value={input.persons_per_dwelling}
              onChange={(v) => setField("persons_per_dwelling", v)}
              error={errors.persons_per_dwelling}
              hint={t("tojuli.hints.persons_per_dwelling")}
            />
          </div>
        </Card>

        <Card title={t("tojuli.groupVentilation")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <NumberField
              label={t("tojuli.fields.infiltration_m3_per_h")}
              unit="m³/h"
              value={input.infiltration_m3_per_h}
              onChange={(v) => setField("infiltration_m3_per_h", v)}
              error={errors.infiltration_m3_per_h}
            />
            <NumberField
              label={t("tojuli.fields.natural_ventilation_m3_per_h")}
              unit="m³/h"
              value={input.natural_ventilation_m3_per_h}
              onChange={(v) =>
                setField("natural_ventilation_m3_per_h", v)
              }
              error={errors.natural_ventilation_m3_per_h}
            />
            <NumberField
              label={t("tojuli.fields.mechanical_supply_m3_per_h")}
              unit="m³/h"
              value={input.mechanical_supply_m3_per_h}
              onChange={(v) =>
                setField("mechanical_supply_m3_per_h", v)
              }
              error={errors.mechanical_supply_m3_per_h}
            />
          </div>
        </Card>

        <Card title={t("tojuli.groupLoads")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <NumberField
              label={t("tojuli.fields.peak_hour")}
              unit="h"
              step={1}
              value={input.peak_hour}
              onChange={(v) => setField("peak_hour", Math.round(v))}
              error={errors.peak_hour}
              hint={t("tojuli.hints.peak_hour")}
            />
            <NumberField
              label={t("tojuli.fields.construction_year")}
              unit="—"
              step={1}
              value={input.construction_year}
              onChange={(v) => setField("construction_year", Math.round(v))}
              error={errors.construction_year}
              hint={t("tojuli.hints.construction_year")}
            />
            <NumberField
              label={t("tojuli.fields.opaque_area_m2")}
              unit="m²"
              value={input.opaque_area_m2}
              onChange={(v) => setField("opaque_area_m2", v)}
              error={errors.opaque_area_m2}
              hint={t("tojuli.hints.opaque_area_m2")}
            />
            <NumberField
              label={t("tojuli.fields.solar_load_w")}
              unit="W"
              value={input.solar_load_w}
              onChange={(v) => setField("solar_load_w", v)}
              error={errors.solar_load_w}
              hint={t("tojuli.hints.solar_load_w")}
            />
            <NumberField
              label={t("tojuli.fields.glazing_transmission_w")}
              unit="W"
              value={input.glazing_transmission_w}
              onChange={(v) => setField("glazing_transmission_w", v)}
              error={errors.glazing_transmission_w}
              hint={t("tojuli.hints.glazing_transmission_w")}
            />
          </div>
        </Card>

        {result && (
          <Card title={t("tojuli.resultTitle")}>
            <div className="space-y-4">
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <ResultRow
                  label={t("tojuli.result.minimum_capacity")}
                  value={`${(result.minimum_capacity_w / 1000).toFixed(2)} kW`}
                  highlight
                />
                <ResultRow
                  label={t("tojuli.result.maatgevende_koelbehoefte")}
                  value={`${result.maatgevende_koelbehoefte_w_per_m2.toFixed(1)} W/m²`}
                  highlight
                />
              </div>

              <div className="border-t border-[var(--oaec-border-subtle)] pt-3">
                <h3 className="mb-2 text-sm font-semibold text-on-surface">
                  {t("tojuli.result.breakdown")}
                </h3>
                <table className="w-full text-sm">
                  <tbody>
                    <ResultTableRow
                      label={t("tojuli.result.internal_load")}
                      value={result.internal_load_w}
                      unit="W"
                    />
                    <ResultTableRow
                      label={t("tojuli.result.outdoor_load")}
                      value={result.outdoor_load_w}
                      unit="W"
                    />
                    <ResultTableRow
                      label={t("tojuli.result.opaque_transmission")}
                      value={result.opaque_transmission_w}
                      unit="W"
                    />
                    <ResultTableRow
                      label={t("tojuli.result.solar_load")}
                      value={result.solar_load_w}
                      unit="W"
                    />
                    <ResultTableRow
                      label={t("tojuli.result.glazing_transmission")}
                      value={result.glazing_transmission_w}
                      unit="W"
                    />
                    <ResultTableRow
                      label={t("tojuli.result.peak_total")}
                      value={result.peak_cooling_load_w}
                      unit="W"
                      bold
                    />
                  </tbody>
                </table>
              </div>

              <p className="text-xs text-on-surface-muted">
                {t("tojuli.normRef")}
              </p>
            </div>
          </Card>
        )}
      </div>
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

function ResultTableRow({
  label,
  value,
  unit,
  bold,
}: {
  label: string;
  value: number;
  unit: string;
  bold?: boolean;
}) {
  return (
    <tr
      className={`border-b border-[var(--oaec-border-subtle)] last:border-0 ${
        bold ? "font-semibold" : ""
      }`}
    >
      <td className="px-3 py-1.5 text-on-surface">{label}</td>
      <td className="px-3 py-1.5 text-right tabular-nums text-on-surface">
        {value.toFixed(0)} {unit}
      </td>
    </tr>
  );
}
