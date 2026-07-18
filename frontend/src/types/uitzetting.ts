/**
 * Uitzetting — thermische lengte-uitzetting + vochtzwelling plaatmateriaal.
 *
 * Frontend-only rekenmodel (géén Rust/API), zelfde patroon als de HWA-/
 * hellingbaan-tools (`types/hwa.ts`, `types/hellingbaan.ts`): puur-TS,
 * state-loos, elke normconstante met een {@link SourcedValue}-bronannotatie.
 * Twee losstaande deelmodellen op één route (`/tools/uitzetting`):
 *
 * - **Thermische uitzetting** (`Δl = α·ΔT·l₀`) — 1-op-1 het "uitzetting"-
 *   tabblad van het rekenblad van de eigenaar. `α` komt uit de
 *   materialenbibliotheek (`lib/materialsDatabase.ts`, veld `alpha`,
 *   eenheid 10⁻⁶/K) of wordt handmatig overschreven.
 * - **Vochtzwelling plaatmateriaal** (EN 318) — het "uitzetting hout"-
 *   tabblad: lineaire zwelling per %RV-verandering, default-coëfficiënt
 *   voor OSB klasse O2.
 */

/**
 * Herkomst van een normwaarde in dit rekenmodel:
 * - `"rekenblad-eigenaar"` — overgenomen uit het interne bronrekenblad van
 *   de eigenaar (referentie-/min-/max-temperatuur, RV-defaults).
 * - `"EN 318"` — de Europese testnorm voor lineaire zwelling van
 *   plaatmateriaal bij vochtopname; de default-zwellingscoëfficiënt is een
 *   gangbare klasse-O2-OSB-waarde uit die norm.
 */
export type UitzettingSource = "rekenblad-eigenaar" | "EN 318";

/** Eén normwaarde met herkomst en referentie, zodat de bron altijd zichtbaar blijft. */
export interface SourcedValue<T> {
  value: T;
  source: UitzettingSource;
  reference: string;
}

// ---------------------------------------------------------------------------
// A. Thermische uitzetting
// ---------------------------------------------------------------------------

/** Invoer voor de thermische-uitzettingsberekening van één lengtemaat. */
export interface ThermalExpansionInput {
  /**
   * Lineaire uitzettingscoëfficiënt α in 10⁻⁶/K. `null` = materiaal zonder
   * zinvolle α (isolatie/folie/spouw uit de bibliotheek) — de berekening
   * geeft dan een nette waarschuwing terug in plaats van een resultaat.
   */
  alphaPer1e6PerK: number | null;
  /** Uitgangslengte l₀ in meter. */
  lengthM: number;
  /** Referentietemperatuur (montage/opname) in °C. */
  refTempC: number;
  /** Minimumtemperatuur (voor de krimpberekening) in °C. */
  minTempC: number;
  /** Maximumtemperatuur (voor de uitzettingsberekening) in °C. */
  maxTempC: number;
}

/** Resultaat van de thermische-uitzettingsberekening. */
export interface ThermalExpansionResult {
  /** ΔT tussen referentie- en minimumtemperatuur (K), gebruikt voor de krimp. */
  deltaTKrimpK: number;
  /** ΔT tussen referentie- en maximumtemperatuur (K), gebruikt voor de vergroting. */
  deltaTUitzettingK: number;
  /** Krimp bij afkoeling naar de minimumtemperatuur, in mm. */
  krimpMm: number;
  /** Vergroting bij opwarming naar de maximumtemperatuur, in mm. */
  vergrotingMm: number;
  /** Krimp per strekkende meter, in mm/m — onafhankelijk van l₀. */
  krimpMmPerM: number;
  /** Vergroting per strekkende meter, in mm/m — onafhankelijk van l₀. */
  vergrotingMmPerM: number;
  warnings: string[];
}

// ---------------------------------------------------------------------------
// B. Vochtzwelling plaatmateriaal (EN 318)
// ---------------------------------------------------------------------------

/** Invoer voor de vochtzwellingsberekening van plaatmateriaal. */
export interface MoistureSwellingInput {
  /** Lengte in meter. */
  lengthM: number;
  /** Relatieve luchtvochtigheid bij installatie, in %. */
  rvInstallPercent: number;
  /** Maximale relatieve luchtvochtigheid in het gebruiksklimaat, in %. */
  rvMaxPercent: number;
  /** Minimale relatieve luchtvochtigheid in het gebruiksklimaat, in %. */
  rvMinPercent: number;
  /** Lineaire zwelling in mm per strekkende meter per %RV — default OSB O2 (EN 318). */
  swellingMmPerMPerPercent: number;
}

/** Resultaat van de vochtzwellingsberekening. */
export interface MoistureSwellingResult {
  /** ΔRV tussen installatie en maximum (%), gebruikt voor de toename. */
  deltaRvMaxPercent: number;
  /** ΔRV tussen installatie en minimum (%), gebruikt voor de krimp. */
  deltaRvMinPercent: number;
  /** Toename bij RV-stijging naar het maximum, in mm. */
  toenameMm: number;
  /** Krimp bij RV-daling naar het minimum, in mm. */
  krimpMm: number;
  warnings: string[];
}
