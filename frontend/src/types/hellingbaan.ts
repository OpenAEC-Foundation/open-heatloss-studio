/**
 * Hellingbaan (parkeergarage) — dimensionering volgens NEN 2443.
 *
 * Geport uit de bestaande pyRevit-tool (rekenkern, GEEN Revit/DirectShape-
 * geometrie): `pyrevit/extensions/bouwkunde.extension/Bouwkunde.tab/
 * Bouwbesluit.panel/HellingbaanGenerator.pushbutton/script.py` (regels
 * ~45-151, functie `bereken_hellingbaan`). Frontend-only rekenmodel, zelfde
 * patroon als de HWA-tool (`types/hwa.ts` + `lib/hwaCalculation.ts`):
 * puur-TS, state-loos, elke normconstante met een {@link SourcedValue}
 * bronannotatie.
 *
 * **Bronstatus:** de pyRevit-bron citeert NEN 2443 voor de garagetype-
 * grenzen en zone-indeling, maar de normtekst zelf is bij deze port niet
 * ingezien — vandaar `"pyRevit-referentie"` op alle constantes. Voor de
 * overgangslengtes (voet/top) is er een bekende discrepantie: de pyRevit-
 * tool rekent met de volle/halve wielbasis (2770/1385 mm), het rekenblad
 * van de eigenaar gebruikt 2720/1360 mm — zie `lib/hellingbaanCalculation.ts`
 * voor de exacte toelichting en het test-bestand voor het effect op de
 * referentiecase.
 */

/**
 * Herkomst van een normwaarde in dit rekenmodel:
 * - `"pyRevit-referentie"` — 1-op-1 overgenomen uit de bestaande pyRevit-
 *   tool (`HellingbaanGenerator.pushbutton/script.py`); die tool citeert
 *   NEN 2443, maar de normtekst zelf is bij deze port niet ingezien.
 * - `"rekenblad-eigenaar"` — enkel ter documentatie aangehaalde,
 *   afwijkende waarde uit het interne rekenblad van de eigenaar; wordt
 *   NIET gebruikt in de berekening.
 */
export type HellingbaanSource = "pyRevit-referentie" | "rekenblad-eigenaar";

/** Eén normwaarde met herkomst en referentie, zodat de bron altijd zichtbaar blijft. */
export interface SourcedValue<T> {
  value: T;
  source: HellingbaanSource;
  reference: string;
}

// ---------------------------------------------------------------------------
// Garagetypes
// ---------------------------------------------------------------------------

/** Garagetype-identificatie, zelfde vier categorieën als de pyRevit-tool. */
export type HellingbaanGarageTypeId =
  | "openbaar"
  | "openbaar_dhumy"
  | "niet_openbaar"
  | "stalling";

/** Eén garagetype met de bijbehorende helling- en breedtegrenzen (NEN 2443). */
export interface HellingbaanGarageType {
  id: HellingbaanGarageTypeId;
  /** Maximale helling in % (kortste toelaatbare hellingbaan). */
  maxHellingPercent: number;
  /** Minimale helling in % (langste toelaatbare hellingbaan). */
  minHellingPercent: number;
  /** Minimale rijbaanbreedte in mm. */
  breedteMinMm: number;
}

// ---------------------------------------------------------------------------
// Invoer
// ---------------------------------------------------------------------------

/** Zone waarin de helling is bepaald — bepaalt hoe `hellingBerekendPercent` tot stand kwam. */
export type HellingbaanZone = "vast" | "kort" | "midden" | "lang" | "simpel";

/** Eén segment van de hellingbaan (overgang onder/boven of hoofdhelling). */
export type HellingbaanSegmentType = "overgang_onder" | "hoofd" | "overgang_boven" | "enkel";

/** Volledige invoer voor een hellingbaan-dimensioneringsberekening. */
export interface HellingbaanInput {
  /** Te overbruggen hoogteverschil in mm. */
  hoogteMm: number;
  garageTypeId: HellingbaanGarageTypeId;
  /** Met overgangshellingen (voet/top op halve helling) of één rechte helling. */
  metOvergang: boolean;
  /** Rijbaanbreedte in mm — getoetst tegen `breedteMinMm` van het gekozen type. */
  breedteMm: number;
  /** Handmatige helling-override in %; `undefined` = gebruik de norm-berekende helling. */
  hellingOverridePercent?: number;
}

// ---------------------------------------------------------------------------
// Resultaat
// ---------------------------------------------------------------------------

/** Eén segment van de hellingbaan met lengte, helling en overbrugde hoogte. */
export interface HellingbaanSegment {
  type: HellingbaanSegmentType;
  lengteMm: number;
  hellingPercent: number;
  hellingGraden: number;
  hoogteMm: number;
}

/** Rekenresultaat van een hellingbaan-dimensioneringsberekening. */
export interface HellingbaanResult {
  /** Gebruikte helling in % — override indien gezet, anders `hellingBerekendPercent`. */
  hellingPercent: number;
  /** Norm-berekende helling in % (ongeacht override) — altijd binnen [min, max] van het type. */
  hellingBerekendPercent: number;
  isOverride: boolean;
  zone: HellingbaanZone;
  segments: HellingbaanSegment[];
  /** Totale lengte van de hellingbaan in mm (som van alle segmenten). */
  lengteTotaalMm: number;
  breedteMm: number;
  /** `true` als `breedteMm` onder de norm-minimumbreedte van het gekozen type ligt. */
  isBreedteOnderMinimum: boolean;
  /** `true` als een override buiten de norm-toegestane bandbreedte [min%, max%] van het type valt. */
  isOverrideBuitenZone: boolean;
  warnings: string[];
}

/** Resultaat van de vergelijkingsberekening "zonder optimalisatie" (vaste max-helling van het type). */
export interface HellingbaanReferentieResult {
  hellingPercent: number;
  lengteTotaalMm: number;
}
