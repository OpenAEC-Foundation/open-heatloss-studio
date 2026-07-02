/**
 * Ψ_g-spacertabel — engineering-richtwaarden voor de samengestelde
 * raam-U-waarde (U_w).
 *
 * De lineaire warmtedoorgangscoëfficiënt van de beglazingsrand (Ψ_g, de
 * "glazing edge"-bijdrage) hangt af van het type randafstandhouder tussen de
 * glasbladen.
 *
 * Deze tabel is **geen** mirror (meer) van de NTA 8800 bijlage L Rust-tabel
 * `nta8800-tables::glazing_edge`. Die Rust-tabel hoort bij een eigen
 * norm-context (NTA 8800 / TO-juli) en is voor de U_w-berekening hier te hoog.
 *
 * Belangrijke simplificatie: deze tabel geeft één Ψ_g-waarde per spacer-type.
 * In werkelijkheid hangt Ψ_g óók af van het glastype (dubbel/triple) en het
 * kozijnmateriaal (hout/kunststof/metaal) — EN-ISO 10077-1 Annex E geeft
 * daarvoor een tabellenkader. De waarden hieronder zijn bewuste
 * engineering-richtwaarden, geijkt op het gangbare geval: HR++ dubbelglas in
 * een courant kozijn. Voor afwijkende combinaties gebruikt de gebruiker de
 * handmatige Ψ_g-override in de calculator.
 *
 * Pure data + lookup — geen React, geen store.
 */

import type { Spacer } from "../types/project";

/**
 * Ψ_g-waarde per randafstandhouder-type in W/(m·K) — engineering-richtwaarden
 * voor HR++ dubbelglas, geïnspireerd op het EN-ISO 10077-1 Annex E-kader.
 *
 * - `aluminium` — conventionele aluminium/metalen afstandhouder. EN-ISO
 *   10077-1 Annex E geeft voor een niet-thermisch-onderbroken metalen spacer
 *   bij dubbelglas ≈ 0,08 W/(m·K); dat is de norm-conservatieve richtwaarde
 *   voor het meest voorkomende geval.
 * - `stainless` — RVS afstandhouder; iets lagere geleiding dan aluminium,
 *   richtwaarde 0,06. Voor een fijnmaziger onderscheid: de handmatige
 *   Ψ_g-override.
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
