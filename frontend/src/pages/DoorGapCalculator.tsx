/**
 * Deurspleet-calculator — losse tool (route `/tools/deurspleet`).
 *
 * Berekent de benodigde vrije doorlaat en spleethoogte onder de deur voor een
 * overstroomdebiet (NEN 1087:2001 §5.1.3.2), via het herbruikbare rekenmodel
 * `lib/doorGap.ts`. Volledig state-loos: inputs leven in lokale
 * component-state, er wordt níets in de project-envelope of een store
 * gepersisteerd — de tool werkt dus ook zonder geopend project.
 *
 * **Eenheden:** dm³/s intern (project-conventie); de debiet-invoer schakelt
 * mee met de persistente weergave-toggle dm³/s ↔ m³/h (`FlowUnitToggle` /
 * `ventilationUiStore`) — conversie alleen aan de UI-rand via
 * `flowToDisplay`/`flowFromDisplay`.
 *
 * Testbaarheid: optionele `initial`-prop voor de begin-invoer, naar het
 * patroon van `Help.initialSection` (`pages/Help.tsx`).
 */
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { FlowUnitToggle } from "../components/ventilation/shared";
import { useVentilationUiStore } from "../components/ventilation/ventilationUiStore";
import { formatDecimals } from "../lib/formatNumber";
import {
  DOOR_GAP_DELTA_P_OFFICE_PA,
  DOOR_GAP_DELTA_P_PA,
  DOOR_GAP_GRILLE_THRESHOLD_MM,
  doorGapAdvice,
  gapHeightMm,
  proposeDoorGrille,
  RULE_OF_THUMB_CM2_PER_DM3S,
  ruleOfThumbAreaCm2,
} from "../lib/doorGap";
import {
  FLOW_UNIT_DECIMALS,
  FLOW_UNIT_LABELS,
  flowFromDisplay,
  flowToDisplay,
} from "../types/ventilation";

/** Δp-preset: woonfunctie (1 Pa) of kantoor (2 Pa) — NEN 1087 §5.1.3.2.7. */
type DeltaPPreset = "residential" | "office";

const DELTA_P_BY_PRESET: Record<DeltaPPreset, number> = {
  residential: DOOR_GAP_DELTA_P_PA,
  office: DOOR_GAP_DELTA_P_OFFICE_PA,
};

/** Begin-invoer (overschrijfbaar via de `initial`-prop, o.a. voor tests). */
interface DoorGapInitial {
  /** Overstroomdebiet in dm³/s (default 7 — BBL-minimum toiletruimte). */
  flowDm3s?: number;
  /** Deurbreedte in mm (default 880 — gangbare binnendeur). */
  doorWidthMm?: number;
  /** Δp-preset (default woonfunctie, 1 Pa). */
  deltaPPreset?: DeltaPPreset;
  /** Aantal deuren waarover het debiet gelijk verdeeld wordt (default 1). */
  doorCount?: number;
  /** Geluidswerende uitvoering (default false) — altijd rooster-advies. */
  acoustic?: boolean;
}

export function DoorGapCalculator({ initial }: { initial?: DoorGapInitial } = {}) {
  const { t } = useTranslation();
  // Weergave-eenheid (persistente UI-voorkeur) — puur display; de lokale
  // state en het rekenmodel blijven dm³/s.
  const flowUnit = useVentilationUiStore((s) => s.flowUnit);

  const [flowDm3s, setFlowDm3s] = useState(initial?.flowDm3s ?? 7);
  const [doorWidthMm, setDoorWidthMm] = useState(initial?.doorWidthMm ?? 880);
  const [deltaPPreset, setDeltaPPreset] = useState<DeltaPPreset>(
    initial?.deltaPPreset ?? "residential",
  );
  const [doorCount, setDoorCount] = useState(initial?.doorCount ?? 1);
  const [acoustic, setAcoustic] = useState(initial?.acoustic ?? false);

  // Pure berekening — geen memo nodig, dit is goedkoop.
  const safeDoorCount = Math.max(1, Math.floor(doorCount));
  const flowPerDoorDm3s = flowDm3s / safeDoorCount;
  const deltaPPa = DELTA_P_BY_PRESET[deltaPPreset];
  const result = gapHeightMm({
    flowDm3s: flowPerDoorDm3s,
    doorWidthMm,
    deltaPPa,
  });
  const ruleAreaCm2 = ruleOfThumbAreaCm2(flowPerDoorDm3s);
  const advice = doorGapAdvice(result.heightMm, acoustic);
  // Rooster-voorstel (per deur) — zelfde benodigde doorlaat als de spleet.
  const grille = advice === "grille" ? proposeDoorGrille(result.areaCm2, acoustic) : null;

  const inputClass =
    "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

  return (
    <div>
      <PageHeader
        title={t("doorGap.title")}
        subtitle={t("doorGap.subtitle")}
        actions={<FlowUnitToggle />}
      />

      <div className="mx-auto max-w-3xl space-y-4 p-6">
        <Card title={t("doorGap.inputTitle")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            {/* Overstroomdebiet — invoer in de gekozen weergave-eenheid;
                conversie naar dm³/s aan de UI-rand (flowFromDisplay). */}
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("doorGap.flow")}{" "}
                <span className="font-normal text-on-surface-muted">
                  [{FLOW_UNIT_LABELS[flowUnit]}]
                </span>
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={Number(
                  flowToDisplay(flowDm3s, flowUnit).toFixed(
                    FLOW_UNIT_DECIMALS[flowUnit] + 1,
                  ),
                )}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  setFlowDm3s(
                    Number.isFinite(n) && n >= 0
                      ? flowFromDisplay(n, flowUnit)
                      : 0,
                  );
                }}
                className={inputClass}
              />
            </label>

            {/* Deurbreedte */}
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("doorGap.doorWidth")}{" "}
                <span className="font-normal text-on-surface-muted">[mm]</span>
              </span>
              <input
                type="number"
                min={1}
                step={10}
                value={doorWidthMm}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  setDoorWidthMm(Number.isFinite(n) && n > 0 ? n : 0);
                }}
                className={inputClass}
              />
            </label>

            {/* Δp-preset (radio) — NEN 1087 §5.1.3.2.7 */}
            <fieldset className="flex flex-col gap-1 text-sm">
              <legend className="font-medium text-on-surface">
                {t("doorGap.deltaP")}
              </legend>
              <div className="mt-1 flex flex-col gap-1.5">
                {(
                  [
                    ["residential", t("doorGap.deltaPResidential")],
                    ["office", t("doorGap.deltaPOffice")],
                  ] as const
                ).map(([preset, label]) => (
                  <label
                    key={preset}
                    className="flex items-center gap-2 text-on-surface-secondary"
                  >
                    <input
                      type="radio"
                      name="doorgap-deltap"
                      checked={deltaPPreset === preset}
                      onChange={() => setDeltaPPreset(preset)}
                      className="accent-[var(--oaec-primary,#2563eb)]"
                    />
                    {label}
                  </label>
                ))}
              </div>
            </fieldset>

            {/* Aantal deuren */}
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("doorGap.doorCount")}
              </span>
              <input
                type="number"
                min={1}
                step={1}
                value={safeDoorCount}
                onChange={(e) => {
                  const n = parseInt(e.target.value, 10);
                  setDoorCount(Number.isFinite(n) && n >= 1 ? n : 1);
                }}
                className={inputClass}
              />
              <span className="text-xs text-on-surface-muted">
                {t("doorGap.doorCountHint")}
              </span>
            </label>

            {/* Geluidswerend — altijd rooster-advies (open spleet is
                akoestisch ongewenst) */}
            <label className="flex flex-col gap-1 text-sm sm:col-span-2">
              <span className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={acoustic}
                  onChange={(e) => setAcoustic(e.target.checked)}
                  className="accent-[var(--oaec-primary,#2563eb)]"
                />
                <span className="font-medium text-on-surface">
                  {t("doorGap.acoustic")}
                </span>
              </span>
              <span className="text-xs text-on-surface-muted">
                {t("doorGap.acousticHint")}
              </span>
            </label>
          </div>
        </Card>

        <Card title={t("doorGap.resultTitle")}>
          <div className="space-y-2 text-sm">
            {safeDoorCount > 1 && (
              <ResultRow
                label={t("doorGap.flowPerDoor")}
                value={`${formatDecimals(flowToDisplay(flowPerDoorDm3s, flowUnit), FLOW_UNIT_DECIMALS[flowUnit])} ${FLOW_UNIT_LABELS[flowUnit]}`}
              />
            )}
            <ResultRow
              label={t("doorGap.requiredArea")}
              value={`${formatDecimals(result.areaCm2, 1)} cm²`}
              emphasized
            />
            <ResultRow
              label={t("doorGap.gapHeight")}
              value={`${result.heightMm} mm`}
              emphasized
            />
            <ResultRow
              label={t("doorGap.ruleOfThumb", {
                rule: RULE_OF_THUMB_CM2_PER_DM3S,
              })}
              value={`${formatDecimals(ruleAreaCm2, 1)} cm²`}
            />
            <p className="pt-1 text-xs text-on-surface-muted">
              {t("doorGap.ruleOfThumbNote")}
            </p>

            {/* Advies: spleet uitvoerbaar of deurrooster toepassen */}
            {advice === "grille" ? (
              <div className="mt-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs font-medium text-amber-700">
                {acoustic
                  ? t("doorGap.adviceAcoustic")
                  : t("doorGap.adviceGrille", {
                      threshold: DOOR_GAP_GRILLE_THRESHOLD_MM,
                    })}
              </div>
            ) : (
              <div className="mt-2 rounded-md border border-green-200 bg-green-50 px-3 py-2 text-xs font-medium text-green-700">
                {t("doorGap.adviceOk")}
              </div>
            )}
          </div>
        </Card>

        {/* Rooster-voorstel — alleen bij rooster-advies; indicatieve seed,
            geen fabrikantdata (zelfde patroon als de ventilatie-units-seed) */}
        {grille && (
          <Card title={t("doorGap.grilleTitle")}>
            <div className="space-y-2 text-sm">
              <ResultRow
                label={t("doorGap.grilleRequiredNet")}
                value={`${formatDecimals(result.areaCm2, 1)} cm²`}
                emphasized
              />
              <ResultRow
                label={t("doorGap.grilleSuggestion")}
                value={t("doorGap.grilleSuggestionValue", {
                  // Bewust `n` en niet `count` — `count` triggert i18next-
                  // pluralisatie (aparte _one/_other-keys).
                  n: grille.count,
                  width: grille.size.widthMm,
                  height: grille.size.heightMm,
                  net: formatDecimals(grille.netAreaCm2PerGrille, 1),
                })}
                emphasized
              />
              <ResultRow
                label={t("doorGap.grilleTotalNet")}
                value={`${formatDecimals(grille.totalNetAreaCm2, 1)} cm²`}
              />
              {acoustic && (
                <p className="pt-1 text-xs text-on-surface-muted">
                  {t("doorGap.grilleAcousticNote")}
                </p>
              )}
              <div className="mt-2 rounded-md border border-[var(--oaec-border)] bg-surface-alt px-3 py-2 text-xs text-on-surface-muted">
                {t("doorGap.grilleDisclaimer")}
              </div>
            </div>
          </Card>
        )}

        <p className="text-xs text-on-surface-muted">{t("doorGap.normRef")}</p>
      </div>
    </div>
  );
}

/** Label + waarde-regel in de resultaat-kaart (tabular-nums voor uitlijning). */
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
