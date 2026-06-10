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
  dm3sToM3h,
  VENTILATION_SYSTEMS,
  type BblFunctionKey,
  type VentilationSystemKey,
} from "../../types/ventilation";
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

/** "12.5 dm³/s" — dm³/s primair. */
export function flowLabel(dm3s: number): string {
  return `${formatDecimals(dm3s, 1)} dm³/s`;
}

/** "45 m³/h" — secundaire weergave. */
export function m3hLabel(dm3s: number): string {
  return `${formatDecimals(dm3sToM3h(dm3s), 0)} m³/h`;
}

// ---------------------------------------------------------------------------
// Status-badge per vertrek (✓ / tekort / natuurlijk / geen eis)
// ---------------------------------------------------------------------------

export function StatusBadge({
  isSupply,
  isExhaust,
  mechanical,
  deficit,
}: {
  isSupply: boolean;
  isExhaust: boolean;
  mechanical: boolean;
  deficit: number;
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
        title={`Tekort: ${formatDecimals(deficit, 1)} dm³/s`}
      >
        tekort {formatDecimals(deficit, 1)}
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
}: {
  label: string;
  dm3s: number;
  muted?: boolean;
}) {
  return (
    <div className="flex items-center justify-between py-0.5">
      <span className="text-on-surface-muted">{label}</span>
      <span
        className={`tabular-nums ${muted ? "text-scaffold-gray" : "font-medium text-on-surface"}`}
      >
        {flowLabel(dm3s)}{" "}
        <span className="font-normal text-scaffold-gray">
          ({m3hLabel(dm3s)})
        </span>
      </span>
    </div>
  );
}

/** Totalen (eis + aanwezig per richting) + balans-indicator. */
export function BuildingBalanceSummary({
  balance,
}: {
  balance: BuildingVentilationBalance;
}) {
  const sys = balance.system;
  return (
    <div className="text-xs">
      <BalanceRow label="Toevoer-eis" dm3s={balance.totalRequiredSupplyDm3s} />
      <BalanceRow label="Afvoer-eis" dm3s={balance.totalRequiredExhaustDm3s} />
      <BalanceRow
        label={
          sys.supplyMechanical
            ? "Aanwezig toevoer"
            : "Aanwezig toevoer (gevelroosters)"
        }
        dm3s={balance.totalPresentSupplyDm3s}
        muted={!sys.supplyMechanical}
      />
      <BalanceRow
        label={
          sys.exhaustMechanical
            ? "Aanwezig afvoer"
            : "Aanwezig afvoer (natuurlijk)"
        }
        dm3s={balance.totalPresentExhaustDm3s}
        muted={!sys.exhaustMechanical}
      />
      <div className="mt-2 flex items-center justify-between">
        <span className="text-on-surface-muted">Balans eis</span>
        {balance.balanced ? (
          <span className="font-semibold text-green-600">✓ In balans</span>
        ) : (
          <span className="font-semibold text-amber-600">
            {balance.imbalanceDm3s > 0 ? "Overdruk +" : "Onderdruk "}
            {formatDecimals(balance.imbalanceDm3s, 1)} dm³/s
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
export function UnitCapacitySummary({ check }: { check: UnitCapacityCheck }) {
  const { t } = useTranslation();
  if (!check.applicable || check.assignedCount === 0) return null;
  return (
    <div className="mt-2 border-t border-[var(--oaec-border-subtle)] pt-2 text-xs">
      <div className="flex items-center justify-between py-0.5">
        <span className="text-on-surface-muted">
          {t("ventilation.units.capacityAssigned")}
        </span>
        <span className="font-medium tabular-nums text-on-surface">
          {flowLabel(check.totalCapacityDm3s)}{" "}
          <span className="font-normal text-scaffold-gray">
            ({formatDecimals(check.totalCapacityM3h, 0)} m³/h)
          </span>
        </span>
      </div>
      <div className="flex items-center justify-between py-0.5">
        <span className="text-on-surface-muted">
          {t("ventilation.units.capacityRequired")}
        </span>
        <span className="font-medium tabular-nums text-on-surface">
          {flowLabel(check.requiredDm3s)}{" "}
          <span className="font-normal text-scaffold-gray">
            ({m3hLabel(check.requiredDm3s)})
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
            {formatDecimals(check.shortfallDm3s, 1)} dm³/s (
            {formatDecimals(check.marginPct, 0)}%)
          </span>
        )}
      </div>
    </div>
  );
}
