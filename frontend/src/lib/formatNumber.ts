/**
 * Display-formatting helpers voor numerieke waarden in de UI.
 *
 * Deze functies zijn bedoeld voor weergave — niet voor berekeningen.
 * De onderliggende store-waarden blijven volledige precisie behouden.
 */

/** Aantal decimalen voor oppervlakte-weergave (m²). */
const AREA_DECIMALS = 2;

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
