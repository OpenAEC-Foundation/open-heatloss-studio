/**
 * Uitzetting-calculator — losse tool (route `/tools/uitzetting`).
 *
 * Twee onafhankelijke rekensecties op één pagina (géén tabs — zelfde
 * stacked-`Card`-structuur als `HwaCalculator.tsx`/`HellingbaanCalculator.tsx`,
 * want beide secties zijn kort genoeg om zonder tab-navigatie leesbaar te
 * blijven):
 *
 * - **A. Thermische uitzetting** (`Δl = α·ΔT·l₀`) — materiaal via de
 *   bestaande `MaterialPicker` (α uit de bibliotheek, handmatig
 *   overschrijfbaar) of direct een α-waarde intypen.
 * - **B. Vochtzwelling plaatmateriaal** (EN 318) — lineaire zwelling per
 *   %RV-verandering, default OSB klasse O2.
 *
 * State is bewust lokaal (`useState`, geen store) — zelfde overweging als
 * `HoekenCalculator.tsx`/`DoorGapCalculator.tsx`: de invoer is vluchtig, er
 * is niets projectgebonden om tussen sessies te bewaren.
 */
import { useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { MaterialPicker } from "../components/construction/MaterialPicker";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { formatDecimals } from "../lib/formatNumber";
import type { Material } from "../lib/materialsDatabase";
import {
  DEFAULT_MAX_TEMP_C,
  DEFAULT_MIN_TEMP_C,
  DEFAULT_REF_TEMP_C,
  DEFAULT_RV_INSTALL_PERCENT,
  DEFAULT_RV_MAX_PERCENT,
  DEFAULT_RV_MIN_PERCENT,
  DEFAULT_SWELLING_MM_PER_M_PER_PERCENT,
  DILATATIE_WARNING_THRESHOLD_MM,
  THICKNESS_SWELLING_NOTE_OSB_O2,
  calculateMoistureSwelling,
  calculateThermalExpansion,
} from "../lib/uitzettingCalculation";

const inputClass =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-2 py-1 text-sm text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

/** Label + waarde-regel (tabular-nums) — zelfde patroon als `HwaCalculator.tsx`. */
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

export function UitzettingCalculator() {
  const { t } = useTranslation();

  // ---------- A. Thermische uitzetting ----------
  const [selectedMaterial, setSelectedMaterial] = useState<Material | null>(null);
  const [alphaInput, setAlphaInput] = useState("12");
  const [lengthInput, setLengthInput] = useState("1");
  const [refTempInput, setRefTempInput] = useState(String(DEFAULT_REF_TEMP_C.value));
  const [minTempInput, setMinTempInput] = useState(String(DEFAULT_MIN_TEMP_C.value));
  const [maxTempInput, setMaxTempInput] = useState(String(DEFAULT_MAX_TEMP_C.value));

  const [pickerOpen, setPickerOpen] = useState(false);
  const [pickerRect, setPickerRect] = useState<DOMRect | null>(null);
  const materialBtnRef = useRef<HTMLButtonElement | null>(null);

  const handleOpenPicker = () => {
    if (materialBtnRef.current) {
      setPickerRect(materialBtnRef.current.getBoundingClientRect());
    }
    setPickerOpen(true);
  };

  const handleSelectMaterial = (material: Material) => {
    setSelectedMaterial(material);
    setAlphaInput(material.alpha != null ? String(material.alpha) : "");
    setPickerOpen(false);
    setPickerRect(null);
  };

  const parsedAlpha = alphaInput.trim() === "" ? null : parseFloat(alphaInput.replace(",", "."));
  const alphaValid = parsedAlpha === null || Number.isFinite(parsedAlpha);

  const parsedLength = parseFloat(lengthInput.replace(",", ".")) || 0;
  const parsedRefTemp = parseFloat(refTempInput.replace(",", ".")) || 0;
  const parsedMinTemp = parseFloat(minTempInput.replace(",", ".")) || 0;
  const parsedMaxTemp = parseFloat(maxTempInput.replace(",", ".")) || 0;

  const thermalResult = useMemo(
    () =>
      calculateThermalExpansion({
        alphaPer1e6PerK: alphaValid ? parsedAlpha : null,
        lengthM: parsedLength,
        refTempC: parsedRefTemp,
        minTempC: parsedMinTemp,
        maxTempC: parsedMaxTemp,
      }),
    [alphaValid, parsedAlpha, parsedLength, parsedRefTemp, parsedMinTemp, parsedMaxTemp],
  );

  // ---------- B. Vochtzwelling plaatmateriaal ----------
  const [moistureLengthInput, setMoistureLengthInput] = useState("0.8");
  const [rvInstallInput, setRvInstallInput] = useState(String(DEFAULT_RV_INSTALL_PERCENT.value));
  const [rvMaxInput, setRvMaxInput] = useState(String(DEFAULT_RV_MAX_PERCENT.value));
  const [rvMinInput, setRvMinInput] = useState(String(DEFAULT_RV_MIN_PERCENT.value));
  const [swellingInput, setSwellingInput] = useState(
    String(DEFAULT_SWELLING_MM_PER_M_PER_PERCENT.value),
  );

  const moistureResult = useMemo(
    () =>
      calculateMoistureSwelling({
        lengthM: parseFloat(moistureLengthInput.replace(",", ".")) || 0,
        rvInstallPercent: parseFloat(rvInstallInput.replace(",", ".")) || 0,
        rvMaxPercent: parseFloat(rvMaxInput.replace(",", ".")) || 0,
        rvMinPercent: parseFloat(rvMinInput.replace(",", ".")) || 0,
        swellingMmPerMPerPercent: parseFloat(swellingInput.replace(",", ".")) || 0,
      }),
    [moistureLengthInput, rvInstallInput, rvMaxInput, rvMinInput, swellingInput],
  );

  return (
    <div>
      <PageHeader title={t("uitzetting.title")} subtitle={t("uitzetting.subtitle")} />

      <div className="mx-auto max-w-4xl space-y-4 p-6">
        {/* A. Thermische uitzetting */}
        <Card title={t("uitzetting.thermalTitle")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">{t("uitzetting.material")}</span>
              <button
                ref={materialBtnRef}
                type="button"
                onClick={handleOpenPicker}
                className="flex-1 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-2 py-1.5 text-left text-sm hover:bg-[var(--oaec-hover)]"
              >
                {selectedMaterial ? (
                  <span className="text-on-surface-secondary">{selectedMaterial.name}</span>
                ) : (
                  <span className="text-on-surface-muted">{t("uitzetting.materialPick")}</span>
                )}
              </button>
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.alpha")}{" "}
                <span className="font-normal text-on-surface-muted">[10{"⁻⁶"}/K]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={alphaInput}
                onChange={(e) => setAlphaInput(e.target.value)}
                placeholder={t("uitzetting.alphaUnknown")}
                className={inputClass}
              />
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.length")} <span className="font-normal text-on-surface-muted">[m]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={lengthInput}
                onChange={(e) => setLengthInput(e.target.value)}
                className={inputClass}
              />
            </label>

          </div>

          {/*
            Losse, volle-breedte rij (i.p.v. genest in de 2-koloms-grid
            hierboven): de labels ("Referentietemperatuur" e.d.) zijn lange,
            onafbreekbare NL-samenstellingen die in een halve-kaartbreedte
            3-koloms-grid over elkaar heen overliepen (geen spatie om op te
            wrappen) — zie screenshot-verificatie.
          */}
          <div className="mt-4 grid grid-cols-3 gap-3">
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.refTemp")} <span className="font-normal text-on-surface-muted">[°C]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={refTempInput}
                onChange={(e) => setRefTempInput(e.target.value)}
                className={inputClass}
              />
            </label>
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.minTemp")} <span className="font-normal text-on-surface-muted">[°C]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={minTempInput}
                onChange={(e) => setMinTempInput(e.target.value)}
                className={inputClass}
              />
            </label>
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.maxTemp")} <span className="font-normal text-on-surface-muted">[°C]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={maxTempInput}
                onChange={(e) => setMaxTempInput(e.target.value)}
                className={inputClass}
              />
            </label>
          </div>

          <div className="mt-4 rounded-md bg-surface-alt p-3 text-sm">
            <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 sm:grid-cols-4">
              <ResultRow
                label={t("uitzetting.krimp")}
                value={`${formatDecimals(thermalResult.krimpMm, 3)} mm`}
                emphasized
              />
              <ResultRow
                label={t("uitzetting.vergroting")}
                value={`${formatDecimals(thermalResult.vergrotingMm, 3)} mm`}
                emphasized
              />
              <ResultRow
                label={t("uitzetting.krimpPerM")}
                value={`${formatDecimals(thermalResult.krimpMmPerM, 3)} mm/m`}
              />
              <ResultRow
                label={t("uitzetting.vergrotingPerM")}
                value={`${formatDecimals(thermalResult.vergrotingMmPerM, 3)} mm/m`}
              />
            </div>
            {thermalResult.warnings.length > 0 && (
              <ul className="mt-2 space-y-1 text-xs oa-warning-text">
                {thermalResult.warnings.map((w, i) => (
                  <li key={i}>⚠ {w}</li>
                ))}
              </ul>
            )}
          </div>

          <p className="mt-2 text-xs text-on-surface-muted">
            {t("uitzetting.thermalDisclaimer", { threshold: DILATATIE_WARNING_THRESHOLD_MM })}
          </p>
        </Card>

        {/* B. Vochtzwelling plaatmateriaal */}
        <Card title={t("uitzetting.moistureTitle")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.length")} <span className="font-normal text-on-surface-muted">[m]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={moistureLengthInput}
                onChange={(e) => setMoistureLengthInput(e.target.value)}
                className={inputClass}
              />
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("uitzetting.swelling")}{" "}
                <span className="font-normal text-on-surface-muted">[mm/m per %RV]</span>
              </span>
              <input
                type="text"
                inputMode="decimal"
                value={swellingInput}
                onChange={(e) => setSwellingInput(e.target.value)}
                className={inputClass}
              />
              <span className="text-xs text-on-surface-muted">{t("uitzetting.swellingPreset")}</span>
            </label>

            <div className="grid grid-cols-3 gap-2 sm:col-span-2">
              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("uitzetting.rvInstall")} <span className="font-normal text-on-surface-muted">[%]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={rvInstallInput}
                  onChange={(e) => setRvInstallInput(e.target.value)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("uitzetting.rvMax")} <span className="font-normal text-on-surface-muted">[%]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={rvMaxInput}
                  onChange={(e) => setRvMaxInput(e.target.value)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("uitzetting.rvMin")} <span className="font-normal text-on-surface-muted">[%]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={rvMinInput}
                  onChange={(e) => setRvMinInput(e.target.value)}
                  className={inputClass}
                />
              </label>
            </div>
          </div>

          <div className="mt-4 rounded-md bg-surface-alt p-3 text-sm">
            <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
              <ResultRow
                label={t("uitzetting.toename")}
                value={`${formatDecimals(moistureResult.toenameMm, 3)} mm`}
                emphasized
              />
              <ResultRow
                label={t("uitzetting.krimp")}
                value={`${formatDecimals(moistureResult.krimpMm, 3)} mm`}
                emphasized
              />
            </div>
            {moistureResult.warnings.length > 0 && (
              <ul className="mt-2 space-y-1 text-xs oa-warning-text">
                {moistureResult.warnings.map((w, i) => (
                  <li key={i}>⚠ {w}</li>
                ))}
              </ul>
            )}
          </div>

          <p className="mt-2 text-xs text-on-surface-muted">{THICKNESS_SWELLING_NOTE_OSB_O2}</p>
        </Card>

        {/* Bronvoetnoot */}
        <div className="space-y-1 text-xs text-on-surface-muted">
          <p>{t("uitzetting.sourceIntro")}</p>
          <ul className="list-inside list-disc space-y-0.5">
            <li>
              {t("uitzetting.sourceTemps")}: {DEFAULT_REF_TEMP_C.reference}
            </li>
            <li>
              {t("uitzetting.sourceRv")}: {DEFAULT_RV_INSTALL_PERCENT.reference}
            </li>
            <li>
              {t("uitzetting.sourceSwelling")}: {DEFAULT_SWELLING_MM_PER_M_PER_PERCENT.reference}
            </li>
          </ul>
        </div>
      </div>

      {/* MaterialPicker portal */}
      {pickerOpen && (
        <MaterialPicker
          anchorRect={pickerRect}
          onSelect={handleSelectMaterial}
          onClose={() => {
            setPickerOpen(false);
            setPickerRect(null);
          }}
        />
      )}
    </div>
  );
}
