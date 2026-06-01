/**
 * Display-formatting helpers voor numerieke waarden in de UI.
 *
 * Deze functies zijn bedoeld voor weergave — niet voor berekeningen.
 * De onderliggende store-waarden blijven volledige precisie behouden.
 */

/** Aantal decimalen voor oppervlakte-weergave (m²). */
const AREA_DECIMALS = 2;

/** Aantal decimalen voor U-waarde-weergave (W/(m²·K)) — engineering-conventie. */
const U_VALUE_DECIMALS = 3;

/** Placeholder voor ontbrekende / ongeldige waarden. */
const MISSING_PLACEHOLDER = "-";

/**
 * Formatteer een oppervlakte-waarde (m²) op 2 decimalen.
 *
 * Geeft een placeholder terug wanneer de waarde null, undefined of geen
 * eindig getal is. Gebruikt `toFixed` zonder locale-formattering zodat de
 * output geschikt is voor tabellen met tabular-nums styling.
 */
export function formatArea(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return MISSING_PLACEHOLDER;
  }
  return value.toFixed(AREA_DECIMALS);
}

/**
 * Formatteer een oppervlakte-waarde met "m²" suffix.
 *
 * Voorbeeld: `12.3456` -> `"12.35 m²"`, `null` -> `"-"`.
 */
export function formatAreaM2(value: number | null | undefined): string {
  const formatted = formatArea(value);
  if (formatted === MISSING_PLACEHOLDER) {
    return MISSING_PLACEHOLDER;
  }
  return `${formatted} m\u00B2`;
}

/**
 * Formatteer een generiek getal op maximaal N decimalen (default 2) en strip
 * overbodige trailing nullen.
 *
 * Bedoeld als generieke display-cap voor invoer-/resultaatvelden die anders
 * de rauwe float zouden tonen (bv. een ge\u00EFmporteerde `0.15333333`). Anders dan
 * `formatArea` (vaste 2 decimalen) houdt deze helper de weergave compact:
 * `20 -> "20"`, `20.5 -> "20.5"`, `0.15333 -> "0.15"`.
 *
 * Geeft een placeholder terug wanneer de waarde null, undefined of geen eindig
 * getal is. Voor U-waarden gebruik je `formatUValue` (3 decimalen) \u2014 niet deze.
 */
export function formatDecimals(
  value: number | null | undefined,
  maxDecimals = 2,
): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return MISSING_PLACEHOLDER;
  }
  // toFixed rondt af op N decimalen; Number() strip de overbodige nullen.
  return String(Number(value.toFixed(maxDecimals)));
}

/**
 * Formatteer een U-waarde (W/(m\u00B2\u00B7K)) op 3 decimalen.
 *
 * U-waarden tonen we bewust met 3 decimalen (engineering-conventie, consistent
 * met de RcCalculator/UwCalculator-weergaven en het rapport). Vaste weergave
 * via `toFixed(3)` zodat `0.15 -> "0.150"` \u2014 geen trailing-null-strip, zodat de
 * precisie visueel herkenbaar blijft.
 *
 * Geeft een placeholder terug bij null/undefined/niet-eindig.
 */
export function formatUValue(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return MISSING_PLACEHOLDER;
  }
  return value.toFixed(U_VALUE_DECIMALS);
}
