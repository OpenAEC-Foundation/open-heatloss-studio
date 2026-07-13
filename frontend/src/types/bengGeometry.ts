/**
 * Handgeschreven TypeScript-spiegel van het gevel-georiënteerde
 * `beng_geometry`-invoerblok (F6).
 *
 * Bron (serde-casing is normatief — spiegel exact):
 *  - `crates/openaec-project-shared/src/beng_geometry.rs`
 *    (`BengGeometry`, `OpaqueConstructionDef`, `RcOrU`, `WindowDef`,
 *    `KozijnType`, `BengZone`, `BengBoundary`, `VlakType`, `BengAdjacency`,
 *    `BengWindowPlacement`).
 *  - `crates/nta8800-model/src/location.rs` (`Orientation`) en
 *    `crates/nta8800-model/src/geometry/window.rs`
 *    (`Obstruction`, `MovableSunShading`, `ShadingControl`).
 *
 * Casing-conventies uit de Rust `#[serde(rename_all = ...)]`-attributen:
 *  - Struct-velden: snake_case (serde-default) — bv. `bruto_buiten_opp_m2`,
 *    `u_w_per_m2k`, `constructie_ref`, `omtrek_p_m`.
 *  - Alle unit-enums: snake_case — `vloer`, `paneel_in_kozijn`, `noord_oost`.
 *  - Externally-tagged enums (`RcOrU`, `BengAdjacency`): een unit-variant
 *    serialiseert als kale string, een data-variant als `{ tag: { ...velden } }`.
 *    Voorbeelden uit de Rust-tests:
 *      RcOrU::Rc(3.7)                              → `{"rc":3.7}`
 *      BengAdjacency::VloerOpMaaiveldBovenGrond    → `"vloer_op_maaiveld_boven_grond"`
 *      BengAdjacency::Buitenlucht { Noord }        → `{"buitenlucht":{"orientatie":"noord"}}`
 *      BengAdjacency::AosForfaitair { None }       → `{"aos_forfaitair":{}}`
 *
 * Optionele velden (`Option<T>` + `skip_serializing_if`) → `field?: T | null`;
 * ze mogen bij het verzenden ontbreken. NIET via `npm run generate-types`
 * regenereren (kapotte pipeline) — dit bestand blijft handmatig.
 */

// ---------------------------------------------------------------------------
// Gedeelde nta8800-model-enums (spiegel location.rs / geometry/window.rs)
// ---------------------------------------------------------------------------

/** 8-punts kompas + horizontaal (serde snake_case). NTA 8800: 0° = noord. */
export type Orientation =
  | "noord"
  | "noord_oost"
  | "oost"
  | "zuid_oost"
  | "zuid"
  | "zuid_west"
  | "west"
  | "noord_west"
  | "horizontaal";

/** Externe belemmering/beschaduwing (serde snake_case). V1: alleen twee uiteinden. */
export type Obstruction = "none" | "minimal";

/** Bedieningsregime beweegbare zonwering (serde snake_case). */
export type ShadingControl = "manual_residential" | "automatic";

/** Beweegbare zonwering op een raam (NTA 8800 §7.6.6.1.4). */
export interface MovableSunShading {
  /** Reductiefactor F_c (0..=1), forfaitair uit tabel 7.5/7.6. */
  f_c: number;
  /** Bedieningsregime (bepaalt f_sh;with-profiel). */
  control: ShadingControl;
}

// ---------------------------------------------------------------------------
// Bibliotheek — opake constructies
// ---------------------------------------------------------------------------

/** Vlak-type van een begrenzing/constructie-definitie (Uniec `BEGR_VLAK`). */
export type VlakType =
  | "vloer"
  | "vloer_boven_buitenlucht"
  | "gevel"
  | "dak"
  | "kelderwand"
  | "bodem";

/**
 * Thermische kwaliteit van een opake constructie — Rc óf U (externally tagged).
 * Uniec voert Rc in; een direct gegeven U is de alternatieve invoer.
 */
export type RcOrU = { rc: number } | { u: number };

/** Opake constructie-definitie in de bouwkundige bibliotheek (Uniec `LIBCONSTRD_*`). */
export interface OpaqueConstructionDef {
  /** Unieke id (referentiedoel van `BengBoundary.constructie_ref`). */
  id: string;
  /** Mens-leesbare omschrijving (bv. "Wand"). */
  omschrijving?: string;
  /** Vlak-type (Uniec `LIBCONSTRD_TYPE`). */
  kind: VlakType;
  /** Rc (m²·K/W) of U (W/(m²·K)). */
  thermal: RcOrU;
}

// ---------------------------------------------------------------------------
// Bibliotheek — kozijnmerken
// ---------------------------------------------------------------------------

/** Type kozijn-vulling (Uniec `LIBCONSTRT_TYPE`). */
export type KozijnType = "raam" | "deur" | "paneel_in_kozijn";

/** Kozijnmerk-definitie (Uniec `LIBCONSTRT_*`, oppervlakte per merk). */
export interface WindowDef {
  /** Unieke id (referentiedoel van `BengWindowPlacement.kozijn_ref`). */
  id: string;
  /** Kozijnmerk-omschrijving (bv. "A", "dakraam"). */
  omschrijving?: string;
  /** Type transparante/opake vulling. */
  kind: KozijnType;
  /** Samengestelde U-waarde in W/(m²·K) (glas + kozijn). */
  u_w_per_m2k: number;
  /** Zonnewarmtedoorlatingsfactor g (0..=1); afwezig/0 voor een opake deur. */
  ggl?: number | null;
  /** Oppervlakte per exemplaar in m². Bijdrage = area_m2 · aantal. */
  area_m2: number;
}

// ---------------------------------------------------------------------------
// Begrenzing — grenst-aan (BengAdjacency, externally tagged)
// ---------------------------------------------------------------------------

/**
 * De oriëntatieloze `grenst_aan`-varianten (serialiseren als kale string).
 * Zie `BengAdjacency`.
 */
export type BengAdjacencyUnit =
  | "vloer_op_maaiveld_boven_kruipruimte"
  | "vloer_op_maaiveld_boven_grond"
  | "vloer_op_maaiveld_boven_onverwarmde_kelder"
  | "vloer_onder_maaiveld_boven_kruipruimte"
  | "vloer_onder_maaiveld_boven_grond"
  | "vloer_onder_maaiveld_boven_onverwarmde_kelder"
  | "sterk_geventileerd"
  | "water"
  | "aangrenzende_verwarmde_ruimte"
  | "aor_forfaitair";

/**
 * Waar een begrenzingsvlak aan grenst (Uniec referentie-enums §2). Externally
 * tagged: unit-varianten als string, `buitenlucht`/`aos_forfaitair` als object
 * met (voor AOS optionele) oriëntatie.
 */
export type BengAdjacency =
  | BengAdjacencyUnit
  | { buitenlucht: { orientatie: Orientation } }
  | { aos_forfaitair: { orientatie?: Orientation | null } };

/**
 * Vlakke keuze-sleutel voor de UI: de unit-varianten plus `buitenlucht` en
 * `aos_forfaitair`. De oriëntatie wordt apart bijgehouden.
 */
export type BengAdjacencyKind =
  | BengAdjacencyUnit
  | "buitenlucht"
  | "aos_forfaitair";

/** Lees de UI-keuze-sleutel uit een `BengAdjacency`. */
export function adjacencyKind(a: BengAdjacency): BengAdjacencyKind {
  if (typeof a === "string") return a;
  if ("buitenlucht" in a) return "buitenlucht";
  return "aos_forfaitair";
}

/** Lees de oriëntatie uit een `BengAdjacency` (buitenlucht/AOS), anders `null`. */
export function adjacencyOrientation(a: BengAdjacency): Orientation | null {
  if (typeof a === "string") return null;
  if ("buitenlucht" in a) return a.buitenlucht.orientatie;
  return a.aos_forfaitair.orientatie ?? null;
}

/** `true` voor de keuzes die een oriëntatie(-select) tonen. */
export function adjacencyHasOrientation(kind: BengAdjacencyKind): boolean {
  return kind === "buitenlucht" || kind === "aos_forfaitair";
}

/**
 * `true` voor de vloer-op-grond-varianten die de P/A-methode gebruiken en dus
 * een omtrek P vereisen (spiegelt `BengAdjacency::requires_omtrek`).
 */
export function adjacencyRequiresOmtrek(a: BengAdjacency): boolean {
  const kind = adjacencyKind(a);
  return (
    kind === "vloer_op_maaiveld_boven_grond" ||
    kind === "vloer_onder_maaiveld_boven_grond"
  );
}

/**
 * Bouw een `BengAdjacency` uit een UI-keuze + (optionele) oriëntatie.
 * `buitenlucht` valt terug op `horizontaal` wanneer geen richting gegeven is
 * (Uniecs `HOR` voor een plat dak); `aos_forfaitair` laat `null` toe (vloer-AOS
 * zonder richting).
 */
export function makeAdjacency(
  kind: BengAdjacencyKind,
  orientatie?: Orientation | null,
): BengAdjacency {
  if (kind === "buitenlucht") {
    return { buitenlucht: { orientatie: orientatie ?? "horizontaal" } };
  }
  if (kind === "aos_forfaitair") {
    return orientatie
      ? { aos_forfaitair: { orientatie } }
      : { aos_forfaitair: {} };
  }
  return kind;
}

// ---------------------------------------------------------------------------
// Begrenzing — kozijn-plaatsing
// ---------------------------------------------------------------------------

/** Een kozijn-plaatsing op een gevel (Uniec `CONSTRT_LIB` + belemmering/zonwering). */
export interface BengWindowPlacement {
  /** Referentie naar het kozijnmerk in `BengGeometry.window_defs`. */
  kozijn_ref: string;
  /** Aantal identieke exemplaren op deze gevel; Rust-default 1. */
  aantal?: number;
  /** Externe belemmering (§17.3); afwezig = `"none"`. */
  belemmering?: Obstruction;
  /** Beweegbare zonwering; afwezig/null = geen zonwering. */
  zonwering?: MovableSunShading | null;
  /** Zomernachtventilatie via dit raam aanwezig; afwezig = `false`. */
  zomernachtventilatie?: boolean;
}

// ---------------------------------------------------------------------------
// Begrenzing (gevel)
// ---------------------------------------------------------------------------

/** Eén begrenzingsvlak van de thermische schil (Uniec `Begrenzing` + `Constructies`). */
export interface BengBoundary {
  /** Unieke id binnen de zone. */
  id: string;
  /** Vlak-omschrijving (bv. "Wand", "Dak"). */
  omschrijving?: string;
  /** Vlak-type (Uniec `BEGR_VLAK`). */
  vlak_type: VlakType;
  /** Waar het vlak aan grenst (Uniec `BEGR_*`). */
  grenst_aan: BengAdjacency;
  /** Bruto BUITEN-oppervlak in m² (kern-heroriëntatie). */
  bruto_buiten_opp_m2: number;
  /** Helling in graden t.o.v. horizontaal (90 = gevel, 15 = hellend dak). */
  helling_deg?: number | null;
  /** Omtrek P van het vloerveld in m; verplicht bij vloer-op-grond. */
  omtrek_p_m?: number | null;
  /** Referentie naar de opake constructie in `BengGeometry.opaque_defs`. */
  constructie_ref: string;
  /** Ramen/deuren op dit vlak. */
  ramen?: BengWindowPlacement[];
}

// ---------------------------------------------------------------------------
// Rekenzone
// ---------------------------------------------------------------------------

/** Rekenzone (Uniec `RZ`, NTA 8800 §6.2). */
export interface BengZone {
  /** Unieke id binnen `BengGeometry.zones`. */
  id: string;
  /** Rekenzone-omschrijving (bv. "woning"). */
  naam?: string;
  /** Gebruiksoppervlakte A_g in m² (noemer van de BENG-indicatoren). */
  a_g_m2: number;
  /** Bouwwijze vloer (thermische massa, Uniec-code; vrij-veld). */
  bouwwijze_vloer?: string | null;
  /** Bouwwijze wand (Uniec-code; vrij-veld). */
  bouwwijze_wand?: string | null;
  /** Woningtype (Uniec `UNIT_TYPEWON`; vrij-veld). */
  woningtype?: string | null;
  /** De begrenzingsvlakken die de thermische schil van deze zone vormen. */
  gevels?: BengBoundary[];
}

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

/**
 * Gevel-georiënteerde BENG-geometrie-invoer (F6). Additief op `ProjectV2`
 * náást `SharedGeometry`. Alle lijsten optioneel; een leeg blok (`{}`) is
 * geldig en serialiseert byte-identiek terug.
 */
export interface BengGeometry {
  /** Bouwkundige bibliotheek — opake constructie-definities. */
  opaque_defs?: OpaqueConstructionDef[];
  /** Kozijn-bibliotheek — kozijnmerk-definities. */
  window_defs?: WindowDef[];
  /** Rekenzones (meestal één voor een grondgebonden woning). */
  zones?: BengZone[];
}
