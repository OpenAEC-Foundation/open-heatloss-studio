/**
 * Gedeelde UI-bouwstenen voor de ventilatiebalans — gebruikt door zowel het
 * Modeller-zijpaneel (`components/modeller/VentilationBalancePanel.tsx`) als
 * de volwaardige tab (`pages/VentilationBalance.tsx`). Eén bron van waarheid
 * voor labels, status-logica, de systeem A–D-selector en de gebouwbalans.
 *
 * **Eenheden:** dm³/s intern; m³/h alleen als afgeleide weergave.
 */

import { useTranslation } from "react-i18next";

import {
  BBL_REQUIREMENTS,
  FLOW_UNIT_DECIMALS,
  FLOW_UNIT_LABELS,
  flowToDisplay,
  isBblDemandIndicative,
  otherFlowUnit,
  VENTILATION_SYSTEMS,
  type BblFunctionKey,
  type FlowDisplayUnit,
  type VentilationSystemKey,
} from "../../types/ventilation";
import { useVentilationUiStore } from "./ventilationUiStore";
import type { BuildingVentilationBalance } from "../../lib/ventilationBalance";
import type { UnitCapacityCheck } from "../../lib/ventilationUnits";
import { formatDecimals } from "../../lib/formatNumber";

// ---------------------------------------------------------------------------
// Labels & formatters
// ---------------------------------------------------------------------------

/** Alle BBL-gebruiksfunctie-sleutels (dropdown-opties). */
export const FUNCTION_OPTIONS = Object.keys(
  BBL_REQUIREMENTS,
) as BblFunctionKey[];

/** Korte systeem-omschrijving onder de selector-knoppen. */
export const SYSTEM_SHORT: Record<VentilationSystemKey, string> = {
  A: "Natuurlijk",
  B: "Mech. toevoer",
  C: "Mech. afvoer",
  D: "Gebalanceerd",
};

/**
 * Store-waarde (dm³/s) → weergave-string in de gekozen eenheid, inclusief
 * eenheid-label. Afronding alleen hier (weergave): dm³/s op 1 decimaal,
 * m³/h op hele getallen ({@link FLOW_UNIT_DECIMALS}).
 */
export function flowDisplayLabel(dm3s: number, unit: FlowDisplayUnit): string {
  return `${formatDecimals(flowToDisplay(dm3s, unit), FLOW_UNIT_DECIMALS[unit])} ${FLOW_UNIT_LABELS[unit]}`;
}

/** Weergave-string in de *andere* eenheid (secundair, tussen haakjes). */
export function flowSecondaryLabel(
  dm3s: number,
  unit: FlowDisplayUnit,
): string {
  return flowDisplayLabel(dm3s, otherFlowUnit(unit));
}

/**
 * "12.5 dm³/s" — dm³/s primair (vaste eenheid). Gebruikt door contexten die
 * NIET met de UI-toggle meeschakelen: het rapport
 * (`lib/ventilationReportBuilder.ts`, genormeerd op dm³/s) en het
 * Modeller-zijpaneel.
 */
export function flowLabel(dm3s: number): string {
  return flowDisplayLabel(dm3s, "dm3s");
}

/** "45 m³/h" — vaste m³/h-weergave (zie {@link flowLabel} voor de context). */
export function m3hLabel(dm3s: number): string {
  return flowDisplayLabel(dm3s, "m3h");
}

// ---------------------------------------------------------------------------
// Eenheden-toggle dm³/s ↔ m³/h (persistent via ventilationUiStore)
// ---------------------------------------------------------------------------

/**
 * Segmented toggle voor de debiet-weergave-eenheid. Leest/schrijft de
 * persistente UI-voorkeur (`ohs-ventilation-ui`); puur weergave — de store
 * blijft dm³/s (zie `types/ventilation.ts`).
 */
export function FlowUnitToggle() {
  const flowUnit = useVentilationUiStore((s) => s.flowUnit);
  const setFlowUnit = useVentilationUiStore((s) => s.setFlowUnit);
  return (
    <div
      className="inline-flex overflow-hidden rounded-md border border-primary/20"
      role="group"
      aria-label="Eenheid debiet-weergave"
      title="Weergave-eenheid voor debieten (opslag blijft dm³/s)"
    >
      {(["dm3s", "m3h"] as const).map((unit) => {
        const active = flowUnit === unit;
        return (
          <button
            key={unit}
            type="button"
            aria-pressed={active}
            onClick={() => setFlowUnit(unit)}
            className={`px-2 py-1 text-xs font-medium transition-colors ${
              active
                ? "bg-primary/10 text-deep-forge"
                : "bg-surface text-deep-forge/60 hover:bg-primary/5"
            }`}
          >
            {FLOW_UNIT_LABELS[unit]}
          </button>
        );
      })}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Indicatief-markering — persoon-gebaseerde functie zonder bezetting
// ---------------------------------------------------------------------------

/**
 * Zichtbare markering "indicatief — bezetting invullen" voor een
 * persoon-gebaseerde gebruiksfunctie (Bbl 4.122 lid 2) zonder ingevulde
 * bezetting: de getoonde eis is dan een m²-benadering i.p.v. de wettelijke
 * per-persoon-eis. Rendert niets wanneer de eis niet indicatief is.
 * Gebruikt door het zijpaneel én de tab (zelfde bron: `isBblDemandIndicative`).
 */
export function IndicativeOccupancyBadge({
  fn,
  occupancy,
}: {
  fn: BblFunctionKey;
  occupancy?: number;
}) {
  const { t } = useTranslation();
  if (!isBblDemandIndicative(fn, occupancy)) return null;
  return (
    <span
      className="ml-1 rounded-full oa-badge-warning px-1.5 py-0.5 text-[9px] font-semibold"
      title={t("ventilation.indicativeHint")}
    >
      {t("ventilation.indicativeBadge")}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Status-badge per vertrek (✓ / tekort / natuurlijk / geen eis)
// ---------------------------------------------------------------------------

export function StatusBadge({
  isSupply,
  isExhaust,
  mechanical,
  deficit,
  unit = "dm3s",
}: {
  isSupply: boolean;
  isExhaust: boolean;
  mechanical: boolean;
  deficit: number;
  /** Weergave-eenheid (default dm³/s — zijpaneel/rapport-conventie). */
  unit?: FlowDisplayUnit;
}) {
  if (!isSupply && !isExhaust) {
    return (
      <span className="rounded-full bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold text-scaffold-gray">
        geen eis
      </span>
    );
  }
  if (!mechanical) {
    return (
      <span
        className="rounded-full bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold text-scaffold-gray"
        title={
          isSupply
            ? "Natuurlijke toevoer — getoetst via gevelroosters"
            : "Natuurlijke afvoer — geen ventiel-toetsing"
        }
      >
        natuurlijk
      </span>
    );
  }
  if (deficit > 0) {
    return (
      <span
        className="rounded-full bg-red-100 px-1.5 py-0.5 text-[9px] font-semibold text-red-600"
        title={`Tekort: ${flowDisplayLabel(deficit, unit)}`}
      >
        tekort{" "}
        {formatDecimals(flowToDisplay(deficit, unit), FLOW_UNIT_DECIMALS[unit])}
      </span>
    );
  }
  return (
    <span className="rounded-full bg-green-100 px-1.5 py-0.5 text-[9px] font-semibold text-green-700">
      ✓
    </span>
  );
}

// ---------------------------------------------------------------------------
// Systeem A–D-selector
// ---------------------------------------------------------------------------

export function SystemSelector({
  value,
  onChange,
  showDescription = true,
}: {
  /** Effectieve systeemsleutel (na default-fallback). */
  value: VentilationSystemKey;
  onChange: (system: VentilationSystemKey) => void;
  /** Toon de tekstuele toelichting onder de knoppen (default aan). */
  showDescription?: boolean;
}) {
  const sys = VENTILATION_SYSTEMS[value];
  return (
    <div>
      <div className="grid grid-cols-4 gap-1.5">
        {(Object.keys(VENTILATION_SYSTEMS) as VentilationSystemKey[]).map(
          (key) => {
            const active = value === key;
            return (
              <button
                key={key}
                onClick={() => onChange(key)}
                className={`rounded-md border px-1 py-1.5 text-center transition-colors ${
                  active
                    ? "border-primary bg-primary/10 text-deep-forge"
                    : "border-primary/20 bg-surface text-deep-forge/60 hover:bg-primary/5"
                }`}
                title={VENTILATION_SYSTEMS[key].label}
              >
                <div className="text-base font-bold leading-none">{key}</div>
                <div className="mt-0.5 text-[9px] text-scaffold-gray">
                  {SYSTEM_SHORT[key]}
                </div>
              </button>
            );
          },
        )}
      </div>
      {showDescription && (
        <p className="mt-2 text-[10px] leading-snug text-on-surface-muted">
          {sys.label}.{" "}
          {sys.supplyMechanical
            ? "Toevoer wordt getoetst op ventielen."
            : "Toevoer via gevelroosters (natuurlijk) — geen ventiel-tekort op toevoer."}{" "}
          {sys.exhaustMechanical
            ? "Afvoer wordt getoetst op ventielen."
            : "Afvoer natuurlijk — geen ventiel-tekort op afvoer."}
        </p>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Gebouwbalans-blok (totalen + indicator)
// ---------------------------------------------------------------------------

function BalanceRow({
  label,
  dm3s,
  muted = false,
  unit = "dm3s",
}: {
  label: string;
  dm3s: number;
  muted?: boolean;
  unit?: FlowDisplayUnit;
}) {
  return (
    <div className="flex items-center justify-between py-0.5">
      <span className="text-on-surface-muted">{label}</span>
      <span
        className={`tabular-nums ${muted ? "text-scaffold-gray" : "font-medium text-on-surface"}`}
      >
        {flowDisplayLabel(dm3s, unit)}{" "}
        <span className="font-normal text-scaffold-gray">
          ({flowSecondaryLabel(dm3s, unit)})
        </span>
      </span>
    </div>
  );
}

/** Totalen (eis + aanwezig per richting) + balans-indicator. */
export function BuildingBalanceSummary({
  balance,
  unit = "dm3s",
}: {
  balance: BuildingVentilationBalance;
  /** Weergave-eenheid (default dm³/s — zijpaneel/rapport-conventie). */
  unit?: FlowDisplayUnit;
}) {
  const sys = balance.system;
  return (
    <div className="text-xs">
      <BalanceRow
        label="Toevoer-eis"
        dm3s={balance.totalRequiredSupplyDm3s}
        unit={unit}
      />
      <BalanceRow
        label="Afvoer-eis"
        dm3s={balance.totalRequiredExhaustDm3s}
        unit={unit}
      />
      <BalanceRow
        label={
          sys.supplyMechanical
            ? "Aanwezig toevoer"
            : "Aanwezig toevoer (gevelroosters)"
        }
        dm3s={balance.totalPresentSupplyDm3s}
        muted={!sys.supplyMechanical}
        unit={unit}
      />
      <BalanceRow
        label={
          sys.exhaustMechanical
            ? "Aanwezig afvoer"
            : "Aanwezig afvoer (natuurlijk)"
        }
        dm3s={balance.totalPresentExhaustDm3s}
        muted={!sys.exhaustMechanical}
        unit={unit}
      />
      <div className="mt-2 flex items-center justify-between">
        <span className="text-on-surface-muted">Balans eis</span>
        {balance.balanced ? (
          <span className="font-semibold text-green-600">✓ In balans</span>
        ) : (
          <span className="font-semibold oa-warning-text">
            {balance.imbalanceDm3s > 0 ? "Overdruk +" : "Onderdruk "}
            {flowDisplayLabel(balance.imbalanceDm3s, unit)}
          </span>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Compacte capaciteitstoets WTW/MV-units (zijpaneel + tab)
// ---------------------------------------------------------------------------

/**
 * Compact resultaat van de unit-capaciteitstoets: capaciteit vs. gecombineerde
 * eis + ✓/tekort-oordeel met marge%. Rendert niets bij systeem A
 * (`applicable: false`) of wanneer er geen units toegewezen zijn — de toets is
 * dan niet zinvol.
 */
export function UnitCapacitySummary({
  check,
  unit = "dm3s",
}: {
  check: UnitCapacityCheck;
  /** Weergave-eenheid (default dm³/s — zijpaneel/rapport-conventie). */
  unit?: FlowDisplayUnit;
}) {
  const { t } = useTranslation();
  if (!check.applicable || check.assignedCount === 0) return null;
  return (
    <div className="mt-2 border-t border-[var(--oaec-border-subtle)] pt-2 text-xs">
      <div className="flex items-center justify-between py-0.5">
        <span className="text-on-surface-muted">
          {t("ventilation.units.capacityAssigned")}
        </span>
        <span className="font-medium tabular-nums text-on-surface">
          {flowDisplayLabel(check.totalCapacityDm3s, unit)}{" "}
          <span className="font-normal text-scaffold-gray">
            ({flowSecondaryLabel(check.totalCapacityDm3s, unit)})
          </span>
        </span>
      </div>
      <div className="flex items-center justify-between py-0.5">
        <span className="text-on-surface-muted">
          {t("ventilation.units.capacityRequired")}
        </span>
        <span className="font-medium tabular-nums text-on-surface">
          {flowDisplayLabel(check.requiredDm3s, unit)}{" "}
          <span className="font-normal text-scaffold-gray">
            ({flowSecondaryLabel(check.requiredDm3s, unit)})
          </span>
        </span>
      </div>
      <div className="mt-1 flex items-center justify-between">
        <span className="text-on-surface-muted">
          {t("ventilation.units.capacityCheck")}
        </span>
        {check.satisfied ? (
          <span className="font-semibold text-green-600">
            ✓ {t("ventilation.units.capacityOk")} (+
            {formatDecimals(check.marginPct, 0)}%)
          </span>
        ) : (
          <span className="font-semibold text-red-600">
            {t("ventilation.units.capacityShortfall")}{" "}
            {flowDisplayLabel(check.shortfallDm3s, unit)} (
            {formatDecimals(check.marginPct, 0)}%)
          </span>
        )}
      </div>
    </div>
  );
}
