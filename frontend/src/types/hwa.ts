/**
 * Hemelwaterafvoer (HWA) — dimensionering van dakafvoeren.
 *
 * Frontend-only rekenmodel (géén Rust, géén API-calls), zelfde patroon als
 * de losse deurspleet-tool (`lib/doorGap.ts` + `pages/DoorGapCalculator.tsx`):
 * puur-TS rekenkern met normconstanten gedocumenteerd op de plek van
 * gebruik, testbestand ernaast. Fase 1 = alleen rekenkern + types, geen UI.
 *
 * **Bronstatus van de constantes:** dit rekenmodel is opgebouwd vanuit een
 * intern rekenblad ("rekenblad-eigenaar"), deels inmiddels geverifieerd
 * tegen NTR 3216-gebaseerde fabrikanten-documentatie (NedZink/Dyka) —
 * `"fabrikanten-doc"`. De formule-structuur is daarbij bevestigd:
 * `Qh = α · i · β · F` (α = platdakfactor, i = regenintensiteit,
 * β = hellingreductie-/gevelfactor, F = oppervlak). De norm zelf
 * (NEN 3215 / NTR 3216) is niet ingezien. Elke normconstante draagt daarom
 * een {@link SourcedValue} met expliciet bronlabel — zie
 * `lib/hwaCalculation.ts`.
 */

/**
 * Herkomst van een normwaarde:
 * - `"rekenblad-eigenaar"` — overgenomen uit het interne bronrekenblad,
 *   nog niet geverifieerd tegen de norm of fabrikanten-documentatie.
 * - `"fabrikanten-doc"` — geverifieerd tegen NTR 3216-gebaseerde
 *   fabrikanten-documentatie (NedZink/Dyka); de norm zelf is niet ingezien.
 * - `"norm-geverifieerd"` — gecontroleerd tegen de aangehaalde normtekst.
 * - `"vuistregel"` — vuistregel/richtlijn uit vakliteratuur (bv. een
 *   Vakrichtlijn), GEEN normbron en NIET tegen NEN 3215/NTR 3216
 *   geverifieerd; puur informatief/controle, beïnvloedt de berekening niet.
 */
export type HwaSource =
  | "rekenblad-eigenaar"
  | "fabrikanten-doc"
  | "norm-geverifieerd"
  | "vuistregel";

/** Eén normwaarde met herkomst en referentie, zodat de bron altijd zichtbaar blijft. */
export interface SourcedValue<T> {
  value: T;
  source: HwaSource;
  reference: string;
}

// ---------------------------------------------------------------------------
// Invoer
// ---------------------------------------------------------------------------

/** Invoermodus voor het dakvlak-oppervlak. */
export type HwaAreaInputMode = "lxb" | "vrij";

/** Platdak-afwerking — bepaalt de reductiefactor bij pitchDeg 0. Niet relevant bij hellende daken. */
export type HwaFlatRoofFinish = "grind" | "plat" | null;

/** Eén dakvlak dat op de HWA-afvoer(en) wordt aangesloten. */
export interface HwaRoofSurface {
  id: string;
  name: string;
  /** `"lxb"` = lengte × breedte, `"vrij"` = direct oppervlak invoeren. */
  areaInputMode: HwaAreaInputMode;
  lengthM?: number;
  widthM?: number;
  areaM2?: number;
  /** Dakhelling in graden; 0 = plat dak. */
  pitchDeg: number;
  /** Alleen relevant bij `pitchDeg === 0`. */
  flatRoofFinish: HwaFlatRoofFinish;
  /** Bijdragend gevel-/opstandoppervlak in m² (default 0). */
  facadeContributionM2: number;
  /** Aantal afvoeren op dit vlak (≥ 1). */
  downpipeCount: number;
  /**
   * Afschot in mm/m — alleen relevant bij `pitchDeg === 0` (plat dak).
   * Puur controle/documentatie: beïnvloedt de berekende afvoercapaciteit
   * NIET (geen normbron voor invloed op de reductiefactor), zie
   * `DESIGN_SLOPE_MM_PER_M` in `lib/hwaCalculation.ts`.
   */
  afschotMmPerM?: number;
}

/** Systeemtype: zwaartekracht ("traditioneel") of onderdruk-/vacuümsysteem ("uv"). */
export type HwaSystemMode = "traditioneel" | "uv";

/** Volledige invoer voor een HWA-dimensioneringsberekening. */
export interface HwaInput {
  surfaces: HwaRoofSurface[];
  /** Regenintensiteit in l/(min·m²) — default via {@link import("../lib/hwaCalculation").DEFAULT_RAIN_INTENSITY_LP_MIN_M2}. */
  rainIntensityLpMinM2: number;
  systemMode: HwaSystemMode;
  /** Capaciteit van het UV-systeem in l/min — verplicht bij `systemMode === "uv"`. */
  uvSystemCapacityLpMin?: number;
}

// ---------------------------------------------------------------------------
// Resultaat
// ---------------------------------------------------------------------------

/** Alternatief advies met één extra afvoer (kleinere diameter per afvoer). */
export interface HwaDownpipeAlternative {
  downpipeCount: number;
  diameterMm: number;
  flowPerPipeLpMin: number;
}

/** Rekenresultaat voor één dakvlak. */
export interface HwaSurfaceResult {
  surfaceId: string;
  /** Effectief oppervlak (basis × reductiefactor + gevelbijdrage) in m². */
  effectiveAreaM2: number;
  /** Totaal debiet voor dit vlak in l/min. */
  flowLpMin: number;
  /** Debiet per afvoer in l/min. */
  flowPerPipeLpMin: number;
  /** Kleinste passende diameter (mm) uit de capaciteitstabel; null als zelfs de grootste diameter niet volstaat. */
  adviesdiameterMm: number | null;
  /** Alternatief met downpipeCount + 1, indien dat een kleinere diameter oplevert. */
  alternatief: HwaDownpipeAlternative | null;
  /** Doorgegeven vanuit de invoer (`HwaRoofSurface.afschotMmPerM`), `null` als niet ingevuld — puur documentatie, zie aldaar. */
  afschotMmPerM: number | null;
  warnings: string[];
}

/** Totaalresultaat van een HWA-dimensioneringsberekening. */
export interface HwaResult {
  surfaceResults: HwaSurfaceResult[];
  totaalEffectiveAreaM2: number;
  totaalFlowLpMin: number;
  /** Alleen gevuld bij `systemMode === "uv"`. */
  uvToets: { pass: boolean; totaalFlowLpMin: number; capaciteitLpMin: number } | null;
  warnings: string[];
}
