/**
 * U_w-berekening — samengestelde raam-U-waarde conform NEN-EN-ISO 10077-1.
 *
 *   U_w = (ΣA_g·U_g + ΣA_f·U_f + Σl_g·Ψ_g) / (ΣA_g + ΣA_f)
 *
 * Standaard-detailniveau: uniform kozijn — één U_g voor alle ruiten, één U_f,
 * uniforme profielbreedte (buitenkozijn + identieke tussenprofielen). De
 * ruit-indeling is een regelmatig c×r-raster van identieke ruiten.
 *
 * Pure functies — geen React, geen store, geen side-effects.
 */

import type { Spacer, UwBreakdown } from "../types/project";

import { spacerPsiG } from "./spacerTable";

// ---------- Types ----------

/**
 * Invoer voor de U_w-berekening. Spiegelt de invoer-velden van `UwBreakdown`;
 * de afgeleide velden (`a_g_m2`, `a_f_m2`, `l_g_m`, `u_w`) worden door
 * `calculateUw` berekend en zijn hier afwezig.
 */
export interface UwInput {
  /** Raambreedte buitenwerks in mm. */
  width_mm: number;
  /** Raamhoogte buitenwerks in mm. */
  height_mm: number;
  /** Uniforme profielbreedte (buitenkozijn + tussenprofielen) in mm. */
  frame_width_mm: number;
  /** Aantal ruit-kolommen, ≥ 1. */
  pane_columns: number;
  /** Aantal ruit-rijen, ≥ 1. */
  pane_rows: number;
  /** Glas-U-waarde U_g in W/(m²·K). */
  u_g: number;
  /** Profiel-U-waarde U_f in W/(m²·K). */
  u_f: number;
  /** Randafstandhouder-type; `null` = volledig handmatige Ψ_g. */
  spacer: Spacer | null;
  /** Effectieve Ψ_g in W/(m·K) — relevant zodra `psi_g_is_manual` true is. */
  psi_g: number;
  /** `true` = `psi_g` is een handmatige override op de spacer-tabelwaarde. */
  psi_g_is_manual: boolean;
}

/** Afgeleide geometrie van een uniform kozijn met c×r ruit-raster. */
export interface UwGeometry {
  /** Totale raamoppervlakte A_w in m². */
  a_w_m2: number;
  /** Totale glasoppervlakte ΣA_g in m². */
  a_g_m2: number;
  /** Totale profieloppervlakte ΣA_f in m². */
  a_f_m2: number;
  /** Totale zichtbare glasrand-omtrek Σl_g in m. */
  l_g_m: number;
  /** Breedte van één ruit in mm. */
  pane_width_mm: number;
  /** Hoogte van één ruit in mm. */
  pane_height_mm: number;
}

/** Volledig resultaat van een U_w-berekening. */
export interface UwResult {
  geometry: UwGeometry;
  /** Effectief gebruikte Ψ_g in W/(m·K). */
  psi_g: number;
  /** Samengestelde raam-U-waarde U_w in W/(m²·K). */
  u_w: number;
}

/** Eén invoervalidatiefout. */
export interface UwValidationError {
  /** Veld waarop de fout slaat (`"width_mm"`, `"general"`, …). */
  field: string;
  /** Mensleesbare melding (Nederlands). */
  message: string;
}

// ---------- Constanten ----------

/** Aantal decimalen waarop U_w in UI en opslag wordt afgerond. */
const UW_DECIMALS = 3;
/** Aantal decimalen voor afgeleide oppervlakten/lengtes. */
const GEOMETRY_DECIMALS = 4;

function round(value: number, decimals: number): number {
  const factor = 10 ** decimals;
  return Math.round(value * factor) / factor;
}

// ---------- Ψ_g-resolutie ----------

/**
 * Effectieve Ψ_g [W/(m·K)] op basis van spacer-keuze en handmatige override.
 *
 * - `psi_g_is_manual` true → de meegegeven `psi_g` wint (handmatige invoer).
 * - anders, met een `spacer` → de spacer-tabelwaarde.
 * - anders (geen spacer, geen override) → de meegegeven `psi_g` als fallback.
 */
export function resolvePsiG(input: UwInput): number {
  if (input.psi_g_is_manual) return input.psi_g;
  const fromTable = spacerPsiG(input.spacer);
  return fromTable ?? input.psi_g;
}

// ---------- Geometrie ----------

/**
 * Bereken de kozijngeometrie voor een uniform c×r ruit-raster.
 *
 * Met W=breedte, H=hoogte, f=profielbreedte, c=kolommen, r=rijen:
 * - profiel bestaat uit het buitenkozijn + (c−1) verticale en (r−1)
 *   horizontale tussenprofielen, alle met breedte f → totale glas-aftrek
 *   is (c+1)·f horizontaal en (r+1)·f verticaal.
 * - `A_g = (W−(c+1)f)·(H−(r+1)f)` is de gesommeerde glasoppervlakte; voor
 *   identieke ruiten valt de raster-som samen tot dit product.
 * - `l_g` is de volle zichtbare glasrand-omtrek per ruit, gesommeerd:
 *   `2·( r·(W−(c+1)f) + c·(H−(r+1)f) )`.
 *
 * Verwacht gevalideerde invoer (`validateUwInput` groen); bij een te breed
 * profiel worden glas-dimensies op 0 geklemd zodat de functie niet crasht.
 */
export function computeGeometry(input: UwInput): UwGeometry {
  const w = input.width_mm;
  const h = input.height_mm;
  const f = input.frame_width_mm;
  const c = input.pane_columns;
  const r = input.pane_rows;

  const glassWidth = Math.max(0, w - (c + 1) * f);
  const glassHeight = Math.max(0, h - (r + 1) * f);

  const a_w_m2 = (w * h) / 1e6;
  const a_g_m2 = (glassWidth * glassHeight) / 1e6;
  const a_f_m2 = Math.max(0, a_w_m2 - a_g_m2);
  const l_g_m = (2 * (r * glassWidth + c * glassHeight)) / 1000;

  const pane_width_mm = c > 0 ? glassWidth / c : 0;
  const pane_height_mm = r > 0 ? glassHeight / r : 0;

  return {
    a_w_m2: round(a_w_m2, GEOMETRY_DECIMALS),
    a_g_m2: round(a_g_m2, GEOMETRY_DECIMALS),
    a_f_m2: round(a_f_m2, GEOMETRY_DECIMALS),
    l_g_m: round(l_g_m, GEOMETRY_DECIMALS),
    pane_width_mm,
    pane_height_mm,
  };
}

// ---------- Validatie ----------

/**
 * Valideer U_w-invoer. Lege array = geldig.
 *
 * Controleert positieve afmetingen, geheel ruit-raster ≥ 1, niet-negatieve
 * U-waarden en — de norm-randvoorwaarde — dat het profiel niet breder is dan
 * het raam: `W > (c+1)·f` én `H > (r+1)·f`, anders blijft er geen glas over.
 */
export function validateUwInput(input: UwInput): UwValidationError[] {
  const errors: UwValidationError[] = [];

  if (!(input.width_mm > 0)) {
    errors.push({ field: "width_mm", message: "Breedte moet groter zijn dan 0." });
  }
  if (!(input.height_mm > 0)) {
    errors.push({ field: "height_mm", message: "Hoogte moet groter zijn dan 0." });
  }
  if (!(input.frame_width_mm > 0)) {
    errors.push({
      field: "frame_width_mm",
      message: "Profielbreedte moet groter zijn dan 0.",
    });
  }
  if (!Number.isInteger(input.pane_columns) || input.pane_columns < 1) {
    errors.push({
      field: "pane_columns",
      message: "Aantal ruit-kolommen moet een geheel getal ≥ 1 zijn.",
    });
  }
  if (!Number.isInteger(input.pane_rows) || input.pane_rows < 1) {
    errors.push({
      field: "pane_rows",
      message: "Aantal ruit-rijen moet een geheel getal ≥ 1 zijn.",
    });
  }
  if (!(input.u_g > 0)) {
    errors.push({ field: "u_g", message: "U_g moet groter zijn dan 0." });
  }
  if (!(input.u_f > 0)) {
    errors.push({ field: "u_f", message: "U_f moet groter zijn dan 0." });
  }

  const psi = resolvePsiG(input);
  if (!(psi >= 0)) {
    errors.push({
      field: "psi_g",
      message: "Ψ_g moet een geldig getal ≥ 0 zijn.",
    });
  }

  // Norm-randvoorwaarde: profiel mag het glas niet wegdrukken.
  // Alleen evalueren wanneer de betrokken velden afzonderlijk geldig zijn,
  // anders dubbele/misleidende meldingen.
  if (
    input.width_mm > 0 &&
    input.frame_width_mm > 0 &&
    Number.isInteger(input.pane_columns) &&
    input.pane_columns >= 1 &&
    input.width_mm <= (input.pane_columns + 1) * input.frame_width_mm
  ) {
    errors.push({
      field: "frame_width_mm",
      message:
        "Profiel te breed: de breedte is te klein voor het profiel + tussenprofielen.",
    });
  }
  if (
    input.height_mm > 0 &&
    input.frame_width_mm > 0 &&
    Number.isInteger(input.pane_rows) &&
    input.pane_rows >= 1 &&
    input.height_mm <= (input.pane_rows + 1) * input.frame_width_mm
  ) {
    errors.push({
      field: "frame_width_mm",
      message:
        "Profiel te breed: de hoogte is te klein voor het profiel + tussenprofielen.",
    });
  }

  return errors;
}

// ---------- U_w-berekening ----------

/**
 * Bereken de samengestelde raam-U-waarde U_w conform NEN-EN-ISO 10077-1.
 *
 * Gooit een `Error` wanneer de invoer ongeldig is — roep eerst
 * `validateUwInput` aan en toon de meldingen in de UI in plaats van
 * `calculateUw` te laten gooien.
 */
export function calculateUw(input: UwInput): UwResult {
  const errors = validateUwInput(input);
  if (errors.length > 0) {
    throw new Error(`Ongeldige U_w-invoer: ${errors[0]!.message}`);
  }

  const geometry = computeGeometry(input);
  const psi_g = resolvePsiG(input);

  const numerator =
    geometry.a_g_m2 * input.u_g +
    geometry.a_f_m2 * input.u_f +
    geometry.l_g_m * psi_g;
  // `validateUwInput` garandeert positieve afmetingen → `a_w_m2 > 0`. Een
  // deling door 0 hier zou een onmogelijke staat zijn; geen stille fallback.
  const u_w = numerator / geometry.a_w_m2;

  return {
    geometry,
    psi_g,
    u_w: round(u_w, UW_DECIMALS),
  };
}

// ---------- UwBreakdown-brug ----------

/**
 * Herkomst-labels van U_g en U_f — vrije tekst van de gekozen catalogus-entry.
 * Pure metadata: geen reken-input, alleen voor weergave in rapport en UI.
 * Een veld is `undefined` zolang de bijbehorende waarde handmatig is ingevoerd.
 */
export interface UwSources {
  /** Vrije-tekst label van de gekozen glasopbouw, of `undefined` bij handmatig. */
  u_g_source?: string;
  /** Vrije-tekst label van het gekozen profielsysteem, of `undefined` bij handmatig. */
  u_f_source?: string;
}

/**
 * Stel een persistent `UwBreakdown`-record samen uit invoer + resultaat.
 * De afgeleide velden worden gecachet op het record (herberekenbaar via
 * `calculateUw`). Bedoeld voor Fase 3 (opslaan op het kozijn-element).
 *
 * `sources` is optionele herkomst-metadata (catalogus-labels van U_g/U_f);
 * lege of afwezige labels worden niet weggeschreven. Een lege string telt
 * via de truthy-filter als "afwezig" — alleen niet-lege labels belanden op
 * het `UwBreakdown`-record, zodat een leeg label geen kale veld-key oplevert.
 */
export function toUwBreakdown(
  input: UwInput,
  result: UwResult,
  sources?: UwSources,
): UwBreakdown {
  const breakdown: UwBreakdown = {
    width_mm: input.width_mm,
    height_mm: input.height_mm,
    frame_width_mm: input.frame_width_mm,
    pane_columns: input.pane_columns,
    pane_rows: input.pane_rows,
    u_g: input.u_g,
    u_f: input.u_f,
    spacer: input.spacer,
    psi_g: result.psi_g,
    psi_g_is_manual: input.psi_g_is_manual,
    a_g_m2: result.geometry.a_g_m2,
    a_f_m2: result.geometry.a_f_m2,
    l_g_m: result.geometry.l_g_m,
    u_w: result.u_w,
  };
  if (sources?.u_g_source) breakdown.u_g_source = sources.u_g_source;
  if (sources?.u_f_source) breakdown.u_f_source = sources.u_f_source;
  return breakdown;
}

/** Lees een opgeslagen `UwBreakdown` terug in als `UwInput` (Fase 3). */
export function fromUwBreakdown(b: UwBreakdown): UwInput {
  return {
    width_mm: b.width_mm,
    height_mm: b.height_mm,
    frame_width_mm: b.frame_width_mm,
    pane_columns: b.pane_columns,
    pane_rows: b.pane_rows,
    u_g: b.u_g,
    u_f: b.u_f,
    spacer: b.spacer ?? null,
    psi_g: b.psi_g,
    psi_g_is_manual: b.psi_g_is_manual,
  };
}

/** Lees de herkomst-labels (U_g/U_f) terug uit een opgeslagen `UwBreakdown`. */
export function sourcesFromUwBreakdown(b: UwBreakdown): UwSources {
  return {
    u_g_source: b.u_g_source,
    u_f_source: b.u_f_source,
  };
}
