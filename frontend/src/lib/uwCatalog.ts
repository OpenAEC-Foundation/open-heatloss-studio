/**
 * uwCatalog — typed loader voor de U_w-profiel- en glascatalogus.
 *
 * Leest de statische referentiedata uit `uwProfileCatalog.json` (profiel-U_f)
 * en `uwGlazingCatalog.json` (glas-U_g), beide frontend-bundled via
 * `resolveJsonModule`. Géén netwerk-/backend-call: de catalogi zijn vaste
 * bestanden in de repo. Een entry toevoegen kan puur door de JSON te
 * bewerken — zie `_meta.how_to_add_a_profile` / `how_to_add_a_glazing`.
 *
 * Twee secties:
 *  - Profielsystemen → `getUwProfiles()`  — koppelt aan het U_f-veld van de calculator
 *  - Glasopbouwen    → `getUwGlazings()`   — koppelt aan het U_g-veld van de calculator
 *
 * Mirror van `productCatalog.ts` (Feature D). De seed-waarden zijn publiek
 * bekende richtwaarden; verifieer bij de fabrikant/leverancier vóór
 * norm-rapportage (zie `_meta.disclaimer` in beide JSON-bestanden).
 */
import glazingJson from "./uwGlazingCatalog.json";
import profileJson from "./uwProfileCatalog.json";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Profielmateriaal — informatief, gebruikt voor groepering in de UI. */
export type UwProfileMaterial = "aluminium" | "kunststof" | "hout" | "staal";

/** Sentinel-id voor de "Handmatig"-optie in de selectors. Wordt nooit als
 *  catalogus-entry gebruikt. */
export const MANUAL_UW_ID = "manual" as const;

/** Eén profielsysteem uit de U_w-profielcatalogus. */
export interface UwProfile {
  /** Stabiele, unieke sleutel (gebruikt als dropdown-value). */
  id: string;
  manufacturer: string;
  system: string;
  /** Materiaalgroep — informatief, voor groepering. */
  material: UwProfileMaterial;
  /** Representatieve profiel-U-waarde U_f in W/(m²·K). */
  u_f: number;
  /** Vrije toelichting (optioneel). */
  note?: string;
}

/** Eén glasopbouw uit de U_w-glascatalogus. */
export interface UwGlazing {
  /** Stabiele, unieke sleutel (gebruikt als dropdown-value). */
  id: string;
  name: string;
  /** Aantal glasbladen. */
  panes: number;
  /** Representatieve centrum-glas-U-waarde U_g in W/(m²·K). */
  u_g: number;
  /** Vrije toelichting (optioneel). */
  note?: string;
}

interface ProfileCatalog {
  profiles: UwProfile[];
}

interface GlazingCatalog {
  glazings: UwGlazing[];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

// `resolveJsonModule` leidt het type van de JSON af uit de inhoud: numerieke
// velden worden `number`, tekstvelden `string`. Het `material`-veld komt zo
// als brede `string` binnen i.p.v. de smalle `UwProfileMaterial`-union, en de
// optionele `note`/`_meta`-sleutels variëren per bestand. We mappen daarom
// elke entry expliciet naar het doeltype: de mapping dwingt de compiler om elk
// veld te controleren tegen `UwProfile`/`UwGlazing` (een hernoemd of ontbrekend
// JSON-veld geeft een compile-fout) en `material` wordt runtime gevalideerd.

/** Geldige `material`-waarden — runtime guard voor de JSON-import. */
const PROFILE_MATERIALS: readonly UwProfileMaterial[] = [
  "aluminium",
  "kunststof",
  "hout",
  "staal",
];

/** Narrow een brede `material`-string naar `UwProfileMaterial`. */
function toProfileMaterial(value: string): UwProfileMaterial {
  if ((PROFILE_MATERIALS as readonly string[]).includes(value)) {
    return value as UwProfileMaterial;
  }
  throw new Error(
    `uwProfileCatalog.json: ongeldige material-waarde "${value}" — ` +
      `verwacht ${PROFILE_MATERIALS.join(" | ")}.`,
  );
}

// De `_meta`-sleutel (instructie-/disclaimertekst) wordt bewust genegeerd;
// alleen `profiles` / `glazings` worden door de mapping geconsumeerd.
const profileCatalog: ProfileCatalog = {
  profiles: profileJson.profiles.map((p) => ({
    id: p.id,
    manufacturer: p.manufacturer,
    system: p.system,
    material: toProfileMaterial(p.material),
    u_f: p.u_f,
    ...(p.note !== undefined ? { note: p.note } : {}),
  })),
};

const glazingCatalog: GlazingCatalog = {
  glazings: glazingJson.glazings.map((g) => ({
    id: g.id,
    name: g.name,
    panes: g.panes,
    u_g: g.u_g,
    ...(g.note !== undefined ? { note: g.note } : {}),
  })),
};

/** Alle profielsystemen uit de catalogus, op fabrikant + systeem gesorteerd. */
export function getUwProfiles(): UwProfile[] {
  return [...profileCatalog.profiles].sort(compareProfiles);
}

/** Alle glasopbouwen uit de catalogus, op U_g aflopend gesorteerd. */
export function getUwGlazings(): UwGlazing[] {
  // Aflopende U_g: enkel glas bovenaan, triple HR+++ onderaan — volgt de
  // intuïtieve volgorde van "slechtst isolerend" naar "best isolerend".
  return [...glazingCatalog.glazings].sort((a, b) => b.u_g - a.u_g);
}

/** Zoek één profielsysteem op id. `undefined` wanneer niet gevonden. */
export function findUwProfile(id: string): UwProfile | undefined {
  return profileCatalog.profiles.find((p) => p.id === id);
}

/** Zoek één glasopbouw op id. `undefined` wanneer niet gevonden. */
export function findUwGlazing(id: string): UwGlazing | undefined {
  return glazingCatalog.glazings.find((g) => g.id === id);
}

function compareProfiles(a: UwProfile, b: UwProfile): number {
  const byManufacturer = a.manufacturer.localeCompare(b.manufacturer, "nl");
  return byManufacturer !== 0
    ? byManufacturer
    : a.system.localeCompare(b.system, "nl");
}
