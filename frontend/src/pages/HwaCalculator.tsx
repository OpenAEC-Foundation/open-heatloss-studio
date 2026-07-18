/**
 * HWA (hemelwaterafvoer) — losse tool (route `/tools/hwa`).
 *
 * Dimensioneert dakafvoeren per dakvlak op basis van de rekenkern in
 * `lib/hwaCalculation.ts` (frontend-only, géén Rust/API). State leeft in
 * `store/hwaStore.ts` (persisted, zodat een refresh de ingevoerde
 * dakvlakken niet wegvaagt — vgl. `recentFilesStore.ts`), dus de tool werkt
 * ook zonder geopend project.
 *
 * Structuur volgt `DoorGapCalculator.tsx`: `PageHeader` + `Card`s, invoer
 * boven/links, resultaat rechts/onder, bronvoetnoot onderaan. De
 * bronvoetnoot haalt de `SourcedValue.reference`-teksten rechtstreeks uit
 * de rekenkern-constanten — niet hardcoded, zodat de tool nooit een andere
 * bronstatus toont dan de rekenkern zelf claimt.
 */
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { Card } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { PageHeader } from "../components/layout/PageHeader";
import { formatDecimals } from "../lib/formatNumber";
import {
  DEFAULT_RAIN_INTENSITY_LP_MIN_M2,
  DESIGN_SLOPE_MM_PER_M,
  DOWNPIPE_CAPACITY_TABLE,
  FLAT_ROOF_FACTORS,
  PITCH_REDUCTION_TABLE,
  calculateHwa,
} from "../lib/hwaCalculation";
import { useHwaStore } from "../store/hwaStore";
import type {
  HwaAreaInputMode,
  HwaFlatRoofFinish,
  HwaRoofSurface,
  HwaSurfaceResult,
  HwaSystemMode,
} from "../types/hwa";

const inputClass =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-2 py-1 text-sm text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

const selectClass = inputClass;

export function HwaCalculator() {
  const { t } = useTranslation();

  const surfaces = useHwaStore((s) => s.surfaces);
  const rainIntensityLpMinM2 = useHwaStore((s) => s.rainIntensityLpMinM2);
  const systemMode = useHwaStore((s) => s.systemMode);
  const uvSystemCapacityLpMin = useHwaStore((s) => s.uvSystemCapacityLpMin);
  const addSurface = useHwaStore((s) => s.addSurface);
  const updateSurface = useHwaStore((s) => s.updateSurface);
  const removeSurface = useHwaStore((s) => s.removeSurface);
  const setRainIntensity = useHwaStore((s) => s.setRainIntensity);
  const setSystemMode = useHwaStore((s) => s.setSystemMode);
  const setUvSystemCapacity = useHwaStore((s) => s.setUvSystemCapacity);

  const result = useMemo(
    () =>
      calculateHwa({
        surfaces,
        rainIntensityLpMinM2,
        systemMode,
        uvSystemCapacityLpMin,
      }),
    [surfaces, rainIntensityLpMinM2, systemMode, uvSystemCapacityLpMin],
  );

  const resultBySurfaceId = useMemo(() => {
    const map = new Map<string, HwaSurfaceResult>();
    for (const r of result.surfaceResults) map.set(r.surfaceId, r);
    return map;
  }, [result.surfaceResults]);

  return (
    <div>
      <PageHeader title={t("hwa.title")} subtitle={t("hwa.subtitle")} />

      <div className="mx-auto max-w-6xl space-y-4 p-6">
        {/* Systeemmodus + regenintensiteit */}
        <Card title={t("hwa.systemTitle")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <fieldset className="flex flex-col gap-1 text-sm">
              <legend className="font-medium text-on-surface">
                {t("hwa.systemMode")}
              </legend>
              <div className="mt-1 flex flex-col gap-1.5">
                {(
                  [
                    ["traditioneel", t("hwa.systemModeTraditioneel")],
                    ["uv", t("hwa.systemModeUv")],
                  ] as const
                ).map(([mode, label]: readonly [HwaSystemMode, string]) => (
                  <label
                    key={mode}
                    className="flex items-center gap-2 text-on-surface-secondary"
                  >
                    <input
                      type="radio"
                      name="hwa-system-mode"
                      checked={systemMode === mode}
                      onChange={() => setSystemMode(mode)}
                      className="accent-[var(--oaec-primary,#2563eb)]"
                    />
                    {label}
                  </label>
                ))}
              </div>
            </fieldset>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("hwa.rainIntensity")}{" "}
                <span className="font-normal text-on-surface-muted">
                  [l/(min·m²)]
                </span>
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={rainIntensityLpMinM2}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  setRainIntensity(Number.isFinite(n) && n >= 0 ? n : 0);
                }}
                className={inputClass}
              />
              <span className="text-xs text-on-surface-muted">
                {t("hwa.rainIntensityDefault", {
                  value: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
                })}
              </span>
            </label>

            {systemMode === "uv" && (
              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("hwa.uvCapacity")}{" "}
                  <span className="font-normal text-on-surface-muted">
                    [l/min]
                  </span>
                </span>
                <input
                  type="number"
                  min={0}
                  step="any"
                  value={uvSystemCapacityLpMin ?? ""}
                  onChange={(e) => {
                    const n = parseFloat(e.target.value);
                    setUvSystemCapacity(
                      Number.isFinite(n) && n >= 0 ? n : undefined,
                    );
                  }}
                  className={inputClass}
                />
              </label>
            )}
          </div>
        </Card>

        {/* Dakvlakken */}
        <Card title={t("hwa.surfacesTitle")}>
          <div className="space-y-3">
            {surfaces.length === 0 && (
              <p className="text-sm text-on-surface-muted">
                {t("hwa.surfacesEmpty")}
              </p>
            )}
            {surfaces.map((surface, idx) => (
              <SurfaceRow
                key={surface.id}
                index={idx}
                surface={surface}
                result={resultBySurfaceId.get(surface.id) ?? null}
                onUpdate={(partial) => updateSurface(surface.id, partial)}
                onRemove={() => removeSurface(surface.id)}
              />
            ))}
            <Button variant="secondary" size="sm" onClick={addSurface}>
              + {t("hwa.addSurface")}
            </Button>
          </div>
        </Card>

        {/* Totaal + UV-toets */}
        <Card title={t("hwa.totalTitle")}>
          <div className="space-y-2 text-sm">
            <ResultRow
              label={t("hwa.totalEffectiveArea")}
              value={`${formatDecimals(result.totaalEffectiveAreaM2, 2)} m²`}
            />
            <ResultRow
              label={t("hwa.totalFlow")}
              value={`${formatDecimals(result.totaalFlowLpMin, 1)} l/min`}
              emphasized
            />

            {result.uvToets && (
              <div
                className={`mt-2 rounded-md border px-3 py-2 text-xs font-medium ${
                  result.uvToets.pass
                    ? "border-green-200 bg-green-50 text-green-700"
                    : "border-red-300 bg-red-50 text-red-700"
                }`}
              >
                {result.uvToets.pass
                  ? t("hwa.uvToetsPass", {
                      flow: formatDecimals(result.uvToets.totaalFlowLpMin, 1),
                      capacity: formatDecimals(result.uvToets.capaciteitLpMin, 1),
                    })
                  : t("hwa.uvToetsFail", {
                      flow: formatDecimals(result.uvToets.totaalFlowLpMin, 1),
                      capacity: formatDecimals(result.uvToets.capaciteitLpMin, 1),
                    })}
              </div>
            )}

            {result.warnings.length > 0 && (
              <ul className="mt-2 space-y-1 text-xs oa-warning-text">
                {result.warnings.map((w, i) => (
                  <li key={i}>⚠ {w}</li>
                ))}
              </ul>
            )}
          </div>
        </Card>

        {/* Bronvoetnoot — teksten rechtstreeks uit de rekenkern-constanten */}
        <div className="space-y-1 text-xs text-on-surface-muted">
          <p>{t("hwa.sourceIntro")}</p>
          <ul className="list-inside list-disc space-y-0.5">
            <li>
              {t("hwa.sourceRain")}: {DEFAULT_RAIN_INTENSITY_LP_MIN_M2.reference}
            </li>
            <li>
              {t("hwa.sourcePitch")}: {PITCH_REDUCTION_TABLE.reference}
            </li>
            <li>
              {t("hwa.sourceFlatRoof")}: {FLAT_ROOF_FACTORS.reference}
            </li>
            <li>
              {t("hwa.sourceCapacity")}: {DOWNPIPE_CAPACITY_TABLE.reference}
            </li>
            <li>
              {t("hwa.sourceSlope")}: {DESIGN_SLOPE_MM_PER_M.reference}
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}

function SurfaceRow({
  index,
  surface,
  result,
  onUpdate,
  onRemove,
}: {
  index: number;
  surface: HwaRoofSurface;
  result: HwaSurfaceResult | null;
  onUpdate: (partial: Partial<HwaRoofSurface>) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="rounded-md border border-[var(--oaec-border-subtle)] p-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <input
          type="text"
          value={surface.name}
          onChange={(e) => onUpdate({ name: e.target.value })}
          className={`${inputClass} w-48 font-medium`}
          placeholder={t("hwa.surfaceNamePlaceholder", { n: index + 1 })}
        />
        <button
          type="button"
          onClick={onRemove}
          className="rounded-md px-2 py-1 text-xs text-red-500 hover:bg-red-50"
        >
          {t("hwa.removeSurface")}
        </button>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {/* Invoermodus toggle */}
        <fieldset className="flex flex-col gap-1 text-xs">
          <legend className="font-medium text-on-surface-secondary">
            {t("hwa.areaInputMode")}
          </legend>
          <div className="flex gap-3">
            {(
              [
                ["lxb", t("hwa.areaInputModeLxb")],
                ["vrij", t("hwa.areaInputModeVrij")],
              ] as const
            ).map(([mode, label]: readonly [HwaAreaInputMode, string]) => (
              <label
                key={mode}
                className="flex items-center gap-1.5 text-on-surface-secondary"
              >
                <input
                  type="radio"
                  name={`hwa-area-mode-${surface.id}`}
                  checked={surface.areaInputMode === mode}
                  onChange={() => onUpdate({ areaInputMode: mode })}
                  className="accent-[var(--oaec-primary,#2563eb)]"
                />
                {label}
              </label>
            ))}
          </div>
        </fieldset>

        {surface.areaInputMode === "lxb" ? (
          <>
            <label className="flex flex-col gap-1 text-xs">
              <span className="font-medium text-on-surface-secondary">
                {t("hwa.lengthM")} [m]
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={surface.lengthM ?? ""}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  onUpdate({ lengthM: Number.isFinite(n) && n >= 0 ? n : undefined });
                }}
                className={inputClass}
              />
            </label>
            <label className="flex flex-col gap-1 text-xs">
              <span className="font-medium text-on-surface-secondary">
                {t("hwa.widthM")} [m]
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={surface.widthM ?? ""}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  onUpdate({ widthM: Number.isFinite(n) && n >= 0 ? n : undefined });
                }}
                className={inputClass}
              />
            </label>
          </>
        ) : (
          <label className="flex flex-col gap-1 text-xs sm:col-span-2">
            <span className="font-medium text-on-surface-secondary">
              {t("hwa.areaM2")} [m²]
            </span>
            <input
              type="number"
              min={0}
              step="any"
              value={surface.areaM2 ?? ""}
              onChange={(e) => {
                const n = parseFloat(e.target.value);
                onUpdate({ areaM2: Number.isFinite(n) && n >= 0 ? n : undefined });
              }}
              className={inputClass}
            />
          </label>
        )}

        <label className="flex flex-col gap-1 text-xs">
          <span className="font-medium text-on-surface-secondary">
            {t("hwa.pitchDeg")} [°]
          </span>
          <input
            type="number"
            min={0}
            max={90}
            step="any"
            value={surface.pitchDeg}
            onChange={(e) => {
              const n = parseFloat(e.target.value);
              onUpdate({ pitchDeg: Number.isFinite(n) ? n : 0 });
            }}
            className={inputClass}
          />
        </label>

        {surface.pitchDeg === 0 && (
          <label className="flex flex-col gap-1 text-xs">
            <span className="font-medium text-on-surface-secondary">
              {t("hwa.flatRoofFinish")}
            </span>
            <select
              value={surface.flatRoofFinish ?? ""}
              onChange={(e) => {
                const v = e.target.value as HwaFlatRoofFinish | "";
                onUpdate({ flatRoofFinish: v === "" ? null : v });
              }}
              className={selectClass}
            >
              <option value="">{t("hwa.flatRoofFinishNone")}</option>
              <option value="grind">{t("hwa.flatRoofFinishGrind")}</option>
              <option value="plat">{t("hwa.flatRoofFinishPlat")}</option>
            </select>
          </label>
        )}

        {surface.pitchDeg === 0 && (
          <label className="flex flex-col gap-1 text-xs">
            <span className="font-medium text-on-surface-secondary">
              {t("hwa.afschotMmPerM")} [mm/m]
            </span>
            <input
              type="number"
              min={0}
              step="any"
              value={surface.afschotMmPerM ?? ""}
              onChange={(e) => {
                const n = parseFloat(e.target.value);
                onUpdate({
                  afschotMmPerM: Number.isFinite(n) && n >= 0 ? n : undefined,
                });
              }}
              className={inputClass}
            />
          </label>
        )}

        <label className="flex flex-col gap-1 text-xs">
          <span className="font-medium text-on-surface-secondary">
            {t("hwa.facadeContribution")} [m²]
          </span>
          <input
            type="number"
            min={0}
            step="any"
            value={surface.facadeContributionM2}
            onChange={(e) => {
              const n = parseFloat(e.target.value);
              onUpdate({ facadeContributionM2: Number.isFinite(n) && n >= 0 ? n : 0 });
            }}
            className={inputClass}
          />
        </label>

        <label className="flex flex-col gap-1 text-xs">
          <span className="font-medium text-on-surface-secondary">
            {t("hwa.downpipeCount")}
          </span>
          <input
            type="number"
            min={1}
            step={1}
            value={surface.downpipeCount}
            onChange={(e) => {
              const n = parseInt(e.target.value, 10);
              onUpdate({ downpipeCount: Number.isFinite(n) && n >= 1 ? n : 1 });
            }}
            className={inputClass}
          />
        </label>
      </div>

      {result && (
        <div className="mt-3 rounded-md bg-surface-alt p-2 text-xs">
          <div className="grid grid-cols-2 gap-x-4 gap-y-1 sm:grid-cols-4">
            <ResultRow
              label={t("hwa.effectiveArea")}
              value={`${formatDecimals(result.effectiveAreaM2, 2)} m²`}
            />
            <ResultRow
              label={t("hwa.flow")}
              value={`${formatDecimals(result.flowLpMin, 1)} l/min`}
            />
            <ResultRow
              label={t("hwa.flowPerPipe")}
              value={`${formatDecimals(result.flowPerPipeLpMin, 1)} l/min`}
            />
            <ResultRow
              label={t("hwa.advice")}
              value={
                result.adviesdiameterMm !== null
                  ? `Ø${result.adviesdiameterMm} mm`
                  : t("hwa.adviceNone")
              }
              emphasized
            />
            {surface.pitchDeg === 0 && (
              <ResultRow
                label={t("hwa.afschotMmPerM")}
                value={
                  result.afschotMmPerM !== null
                    ? `${formatDecimals(result.afschotMmPerM, 1)} mm/m`
                    : t("hwa.afschotNone")
                }
              />
            )}
          </div>
          {result.alternatief && (
            <p className="mt-1 text-on-surface-muted">
              {t("hwa.alternative", {
                count: result.alternatief.downpipeCount,
                diameter: result.alternatief.diameterMm,
                flow: formatDecimals(result.alternatief.flowPerPipeLpMin, 1),
              })}
            </p>
          )}
          {result.warnings.length > 0 && (
            <ul className="mt-1 space-y-0.5 oa-warning-text">
              {result.warnings.map((w, i) => (
                <li key={i}>⚠ {w}</li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

/** Label + waarde-regel (tabular-nums voor uitlijning) — zelfde patroon als DoorGapCalculator. */
function ResultRow({
  label,
  value,
  emphasized = false,
}: {
  label: string;
  value: string;
  emphasized?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-on-surface-muted">{label}</span>
      <span
        className={`tabular-nums ${emphasized ? "font-semibold text-on-surface" : "text-on-surface-secondary"}`}
      >
        {value}
      </span>
    </div>
  );
}
