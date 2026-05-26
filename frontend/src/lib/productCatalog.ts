/**
 * productCatalog — typed loader voor de BCRG-productcatalogus.
 *
 * Leest de statische referentiedata uit `data/productCatalog.json` (frontend-
 * bundled via `resolveJsonModule`). Géén netwerk-/backend-call: de catalogus
 * is een vast bestand in de repo. Een unit toevoegen kan puur door de JSON te
 * bewerken — zie `_meta.how_to_add_a_unit` daarin.
 *
 * Twee secties:
 *  - WTW-units      → `getWtwUnits()`     — koppelt aan V1 `heat_recovery_efficiency`
 *  - Koelunits      → `getCoolingUnits()` — koppelt aan TO-juli `CoolingSystem`
 *
 * Feature D (werkpakket D2). De seed-waarden zijn publiek bekende richtwaarden;
 * verifieer tegen bcrg.nl vóór norm-rapportage (zie `_meta.verification_note`).
 */
import catalogJson from "../data/productCatalog.json";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Koel-systeemtype — mirror van `nta8800-cooling::CoolingSystem` enum. */
export type CoolingUnitKind =
  | "compression_cooling"
  | "absorption_cooling"
  | "free_cooling";

/** Sentinel-id voor de "Handmatig invoeren"-optie in de selectors. Wordt
 *  nooit als catalogus-unit gebruikt. */
export const MANUAL_PRODUCT_ID = "manual" as const;

/** Eén WTW-unit uit de BCRG-catalogus. */
export interface WtwUnit {
  /** Stabiele, unieke sleutel (gebruikt als dropdown-value). */
  id: string;
  brand: string;
  model: string;
  /** Thermisch rendement warmteterugwinning η_hr, fractie 0..1. */
  eta_hr: number;
  /** Nominale luchtdebiet-capaciteit in m³/h. Afgeleid uit modelnaam,
   *  niet officieel BCRG-geverifieerd. Wordt in de UI gebruikt als
   *  drempel om te kleine units uit te grijzen in de selector. */
  q_nominal_m3h: number;
  /** Specifiek ventilatorvermogen f_SFP in W/(dm³/s). Optioneel —
   *  catalogus-data zonder UI-binding zolang er geen SFP-invoerveld is. */
  f_sfp?: number;
  /** BCRG-verklaringnummer; lege string = nog niet geverifieerd. */
  bcrg_declaration_nr: string;
  /** Indicatie bouwjaar/marktintroductie (informatief). */
  year_indication: string;
}

/** Eén koelunit uit de BCRG-catalogus. */
export interface CoolingUnit {
  /** Stabiele, unieke sleutel (gebruikt als dropdown-value). */
  id: string;
  brand: string;
  model: string;
  type: CoolingUnitKind;
  /** Seizoensgebonden koudefactor — gevuld bij `compression_cooling`. */
  scop_cooling?: number;
  /** Koudefactor — gevuld bij `absorption_cooling`. */
  cop?: number;
  /** Benuttingsfractie 0..1 — gevuld bij `free_cooling`. */
  factor?: number;
  /** BCRG-verklaringnummer; lege string = nog niet geverifieerd. */
  bcrg_declaration_nr: string;
  /** Indicatie bouwjaar/marktintroductie (informatief). */
  year_indication: string;
}

interface ProductCatalog {
  wtw_units: WtwUnit[];
  cooling_units: CoolingUnit[];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

// `resolveJsonModule` levert de JSON al getypeerd uit op basis van de inhoud;
// we casten naar het expliciete `ProductCatalog`-contract zodat de helpers een
// stabiel API-oppervlak houden ook als de JSON groeit. De `_meta`-sleutel
// (instructie-/verificatietekst) wordt bewust genegeerd.
const catalog = catalogJson as unknown as ProductCatalog;

/** Alle WTW-units uit de catalogus, op merk + model gesorteerd. */
export function getWtwUnits(): WtwUnit[] {
  return [...catalog.wtw_units].sort(compareByBrandModel);
}

/** Alle koelunits uit de catalogus, op merk + model gesorteerd. */
export function getCoolingUnits(): CoolingUnit[] {
  return [...catalog.cooling_units].sort(compareByBrandModel);
}

/** Zoek één WTW-unit op id. `undefined` wanneer niet gevonden. */
export function findWtwUnit(id: string): WtwUnit | undefined {
  return catalog.wtw_units.find((u) => u.id === id);
}

/** Zoek één koelunit op id. `undefined` wanneer niet gevonden. */
export function findCoolingUnit(id: string): CoolingUnit | undefined {
  return catalog.cooling_units.find((u) => u.id === id);
}

function compareByBrandModel(
  a: { brand: string; model: string },
  b: { brand: string; model: string },
): number {
  const byBrand = a.brand.localeCompare(b.brand, "nl");
  return byBrand !== 0 ? byBrand : a.model.localeCompare(b.model, "nl");
}
