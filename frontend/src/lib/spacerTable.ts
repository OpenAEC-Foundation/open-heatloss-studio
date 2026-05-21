/**
 * Ψ_g-spacertabel — TS-mirror van `nta8800-tables::glazing_edge::SpacerKind`.
 *
 * De lineaire warmtedoorgangscoëfficiënt van de beglazingsrand (Ψ_g, de
 * "glazing edge"-bijdrage) hangt af van het type randafstandhouder tussen de
 * glasbladen. Vier representatieve waarden conform NEN-EN-ISO 10077-1; een
 * Rust-bridge voor deze vier getallen zou disproportioneel zijn.
 *
 * Pure data + lookup — geen React, geen store.
 */

import type { Spacer } from "../types/project";

/**
 * Ψ_g-waarde per randafstandhouder-type in W/(m·K).
 *
 * - `aluminium` — conventionele aluminium afstandhouder (hoogste warmtelek).
 * - `stainless` — RVS afstandhouder.
 * - `warm_edge_polymer` — kunststof "warm edge" afstandhouder.
 * - `warm_edge_foam` — schuim "warm edge" afstandhouder (laagste warmtelek).
 */
export const SPACER_PSI_G: Record<Spacer, number> = {
  aluminium: 0.08,
  stainless: 0.06,
  warm_edge_polymer: 0.04,
  warm_edge_foam: 0.02,
};

/** Nederlandse labels voor de spacer-dropdown in de UI. */
export const SPACER_LABELS_NL: Record<Spacer, string> = {
  aluminium: "Aluminium",
  stainless: "RVS",
  warm_edge_polymer: "Warm edge — kunststof",
  warm_edge_foam: "Warm edge — schuim",
};

/** Geordende lijst van alle spacer-types — stabiele dropdown-volgorde. */
export const SPACER_ORDER: readonly Spacer[] = [
  "aluminium",
  "stainless",
  "warm_edge_polymer",
  "warm_edge_foam",
];

/**
 * Tabel-Ψ_g voor een gegeven spacer-type [W/(m·K)].
 * `null` (volledig handmatige Ψ_g) → `undefined`.
 */
export function spacerPsiG(spacer: Spacer | null | undefined): number | undefined {
  if (spacer === null || spacer === undefined) return undefined;
  return SPACER_PSI_G[spacer];
}
