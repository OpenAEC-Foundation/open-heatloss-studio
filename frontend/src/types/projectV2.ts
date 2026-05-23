/**
 * Handgeschreven TypeScript equivalenten van `openaec-project-shared`
 * (`crates/openaec-project-shared/src/{project,shared,geometry,calcs}.rs`).
 *
 * Deze types beschrijven het V2 multi-calc project model zoals
 * vastgelegd in ADR-002. Ze worden in F3 gebruikt door de tabbed
 * ProjectSetup UI; volledige schema-gen volgt in een latere fase.
 *
 * **Backward-compat:** Frontend houdt momenteel een V1 `Project`
 * (`types/project.ts`) als bron-van-waarheid in `projectStore`.
 * V2-only velden (postcode, location, notes, num_storeys,
 * construction_year, building_type kind/subtype) leven sidecar in de
 * store onder `sharedExtra` totdat backend V2 native serveert.
 */

import type { Project } from "./project";

export const SCHEMA_VERSION_V2 = 2 as const;

// ---------------------------------------------------------------------------
// SharedProject — ventilation types (gespiegeld van VentilationSystemKind in Rust)
// ---------------------------------------------------------------------------

/** V2 ventilatiesysteem soorten — snake_case JSON keys (Rust serde rename_all). */
export type VentilationSystemKind =
  | "mech_balanced"
  | "mech_supply"
  | "mech_exhaust"
  | "natural";

/** Warmteterugwinning (WTW/WRG) configuratie. */
export interface HeatRecovery {
  /** Rendement (0.0–1.0). */
  efficiency: number;
  /** Heeft vorstbeveiliging. */
  frost_protection: boolean;
  /** Toevoertemperatuur na WTW in °C (optioneel). */
  supply_temperature?: number;
}

// ---------------------------------------------------------------------------
// SharedProject
// ---------------------------------------------------------------------------

export type ResidentialType =
  | "detached"
  | "semi_detached"
  | "terraced"
  | "end_of_terrace"
  | "porch"
  | "gallery"
  | "stacked";

export type UtilityType =
  | "office"
  | "education"
  | "assembly"
  | "healthcare"
  | "lodging"
  | "sport"
  | "retail"
  | "industrial"
  | "other";

/** Discriminated union — `kind` is tag in Rust `#[serde(tag = "kind")]`. */
export type BuildingTypeShared =
  | { kind: "woning"; subtype: ResidentialType }
  | { kind: "utiliteit"; subtype: UtilityType };

export interface SharedProject {
  name: string;
  project_number?: string | null;
  address?: string | null;
  postcode?: string | null;
  location?: string | null;
  client?: string | null;
  date?: string | null;
  engineer?: string | null;
  notes?: string | null;
  building_type: BuildingTypeShared;
  construction_year?: number | null;
  gross_floor_area_m2?: number | null;
  num_storeys?: number | null;
  ventilation_system?: VentilationSystemKind;
  heat_recovery?: HeatRecovery;
  infiltration_m3_per_h?: number;
  /**
   * Mechanische toevoer in m³/h (NTA 8800 tab 11.23).
   * Optioneel — Rust spiegel is `Option<f64>` met `#[serde(default)]`.
   * Backend valt terug op `default_ach` in tojuli.rs als undefined.
   */
  mechanical_supply_m3_per_h?: number;
  /**
   * Mechanische afvoer in m³/h (NTA 8800 tab 11.23).
   * Optioneel — Rust spiegel is `Option<f64>` met `#[serde(default)]`.
   * Backend valt terug op `default_ach` in tojuli.rs als undefined.
   */
  mechanical_exhaust_m3_per_h?: number;
}

// ---------------------------------------------------------------------------
// SharedGeometry — minimal types (placeholder in F3)
// ---------------------------------------------------------------------------

export type ConstructionKind = "wall" | "floor" | "ceiling" | "roof";

export type BoundaryKind =
  | "exterior"
  | "unheated_space"
  | "adjacent_room"
  | "adjacent_building"
  | "ground"
  | "open_water";

export type OpeningKind = "window" | "door";

export interface Opening {
  id: string;
  kind: OpeningKind;
  area_m2: number;
  u_value: number;
  g_value?: number | null;
  frame_fraction?: number | null;
}

export interface ConstructionLayer {
  material: string;
  thickness_mm: number;
  lambda_w_per_mk?: number;
  r_m2k_per_w?: number | null;
}

export interface Construction {
  id: string;
  description: string;
  kind: ConstructionKind;
  boundary: BoundaryKind;
  area_m2: number;
  u_value: number;
  orientation_deg?: number | null;
  slope_deg?: number | null;
  openings?: Opening[];
  layers?: ConstructionLayer[];
  adjacent_space_id?: string | null;
  psi_thermal_bridge?: number | null;
}

export interface Space {
  id: string;
  name: string;
  function?: string | null;
  floor_area_m2: number;
  height_m: number;
  constructions?: Construction[];
  theta_i_winter_c?: number | null;
  theta_i_summer_c?: number | null;
}

export interface SharedGeometry {
  spaces: Space[];
}

// ---------------------------------------------------------------------------
// Calcs — per-norm inputs
// ---------------------------------------------------------------------------

/**
 * V2-placeholder: contains the legacy V1 Project JSON inline (flattened in
 * Rust, here held as the typed Project). The view-mapper in
 * `openaec-project-shared::view` re-constructs an isso51_core::Project from
 * this blob.
 */
export interface Iso51Inputs {
  legacy_v1: Project | Record<string, unknown>;
}

/**
 * ISSO 53 utility-warmteverlies inputs. Parallel aan `Iso51Inputs` — bevat
 * (transitional) een volledige ISSO 53 project-JSON onder `legacy`, identiek
 * patroon als bij ISSO 51. Velden worden in latere fasen uitgesplitst.
 *
 * Rust spiegel: `openaec_project_shared::calcs::Iso53Inputs` met
 * `#[serde(flatten)] pub legacy: serde_json::Value`.
 */
export interface Iso53Inputs {
  legacy: Record<string, unknown>;
}

export interface TojuliInputs {
  quick_check?: Record<string, unknown> | null;
}

export interface Calcs {
  isso51?: Iso51Inputs | null;
  isso53?: Iso53Inputs | null;
  tojuli?: TojuliInputs | null;
}

/**
 * Actieve norm voor een ProjectV2. Spiegel van de Rust enum
 * `ActiveNorm` (`#[serde(rename_all = "camelCase")]`). Bepaalt welke
 * berekening primair is in UI, CLI, en PDF-generator.
 */
export type ActiveNorm = "isso51" | "isso53";

// ---------------------------------------------------------------------------
// ISSO 53 UI sidecar-state (fase 3)
// ---------------------------------------------------------------------------

/**
 * ISSO 53 BuildingShape (tabel 4.9) — gebruikt voor de
 * infiltratie-vormfactor. Camel-case JSON keys (spiegel van Rust
 * `isso53_core::model::enums::BuildingShape`).
 */
export type Isso53BuildingShape =
  | "meerlaags"
  | "eenLaagMetKap"
  | "eenLaagMetPlatDak"
  | "eenLaagMetHalfPlatDak";

/**
 * ISSO 53 GebouwTypePositie (tabel 4.8) — positie binnen het complex
 * voor de infiltratiebepaling. Spiegel van Rust
 * `isso53_core::model::enums::GebouwTypePositie`.
 */
export type Isso53BuildingPosition =
  | "enkellaagsTussen"
  | "enkellaagsKop"
  | "enkellaagsVrijstaand"
  | "meerlaagsGeheel"
  | "meerlaagsTop"
  | "meerlaagsTussen"
  | "meerlaagsOnder";

/**
 * ISSO 53 GebouwTypeWinddruk (tabel 4.6) — winddrukverdelingsfactor
 * f_type. Andere indeling dan BuildingShape (4.9) en GebouwTypePositie
 * (4.8). Spiegel van Rust `GebouwTypeWinddruk`.
 */
export type Isso53WindPressureType =
  | "eenlaagsMetKap"
  | "eenlaagsMetPlatDak"
  | "meerlaagsStandaard"
  | "meerlaagsVolgevelBinnengalerij"
  | "meerlaagsDubbeleHuidOnderbroken"
  | "meerlaagsDubbeleHuidDoorlopend";

/** ISSO 53 Thermische massa (tabel 2.4). */
export type Isso53ThermalMass = "licht" | "gemiddeld" | "zwaar";

/** ISSO 53 Ventilatiesysteemtype (tabel 4.7) — let op: andere namespace dan V1. */
export type Isso53VentilationSystem =
  | "systemA"
  | "systemB"
  | "systemC"
  | "systemD"
  | "systemE";

/**
 * ISSO 53 GebruiksFunctie (Bouwbesluit; ISSO 53 tabel 2.2). Spiegel
 * van Rust `isso53_core::model::enums::GebruiksFunctie`.
 */
export type Isso53GebruiksFunctie =
  | "kantoor"
  | "onderwijs"
  | "gezondheidszorg"
  | "bijeenkomst"
  | "logies"
  | "sport"
  | "winkel"
  | "cel"
  | "industrie";

/**
 * ISSO 53 RuimteType (tabel 2.2). Spiegel van Rust
 * `isso53_core::model::enums::RuimteType`. De UI biedt het volledige
 * vlakke menu aan — de norm wijst per combi (gebruiksFunctie,
 * ruimteType) de getallen toe.
 */
export type Isso53RuimteType =
  | "verblijfsruimte"
  | "verblijfsgebied"
  | "badruimte"
  | "toiletruimte"
  | "verkeersruimte"
  | "technischeRuimte"
  | "bergruimte"
  | "onbenoemdeRuimte"
  | "stallingsruimte"
  | "garage"
  | "kantoorruimte"
  | "receptie"
  | "lesruimte"
  | "collegezaal"
  | "werkplaats"
  | "bureauruimte"
  | "patientenkamer"
  | "operatiekamer"
  | "onderzoekruimte"
  | "eetruimte"
  | "restaurant"
  | "kantine"
  | "vergaderruimte"
  | "hotelkamer"
  | "sportzaal"
  | "verkoopruimte"
  | "supermarkt"
  | "warenhuis";

/**
 * ISSO 53 building-niveau invoer die niet in V1 `Building` past.
 * Wordt sidecar opgeslagen in `projectStore` en is alleen actief
 * wanneer `norm === "isso53"`. Bij norm-wissel (fase 4) wordt deze
 * sidecar geconverteerd of leeg gelaten.
 */
export interface Isso53BuildingState {
  buildingShape: Isso53BuildingShape;
  buildingPosition: Isso53BuildingPosition;
  windPressureType: Isso53WindPressureType;
  thermalMass: Isso53ThermalMass;
  ventilationSystem: Isso53VentilationSystem;
  constructionYear: number | null;
}

export const DEFAULT_ISSO53_BUILDING: Isso53BuildingState = {
  buildingShape: "meerlaags",
  buildingPosition: "meerlaagsTussen",
  windPressureType: "meerlaagsStandaard",
  thermalMass: "gemiddeld",
  ventilationSystem: "systemD",
  constructionYear: null,
};

/**
 * Per-ruimte ISSO 53 sidecar-state (gebruiksFunctie + ruimteType).
 * Key = `room.id` uit V1 `Project.rooms[]`. Wordt alleen gerenderd
 * en gepersisteerd wanneer `norm === "isso53"`.
 */
export interface Isso53RoomState {
  gebruiksFunctie: Isso53GebruiksFunctie;
  ruimteType: Isso53RuimteType;
}

export const DEFAULT_ISSO53_ROOM: Isso53RoomState = {
  gebruiksFunctie: "kantoor",
  ruimteType: "verblijfsruimte",
};

// ---------------------------------------------------------------------------
// ProjectV2 root
// ---------------------------------------------------------------------------

export interface ProjectV2 {
  schema_version: typeof SCHEMA_VERSION_V2;
  shared: SharedProject;
  geometry: SharedGeometry;
  calcs: Calcs;
}

// ---------------------------------------------------------------------------
// SharedExtra — sidecar voor V2-only velden die nog niet in V1-store passen
// ---------------------------------------------------------------------------

/**
 * V2-only fields die niet in V1 `Project`/`ProjectInfo`/`Building` passen
 * en momenteel niet naar de backend gaan. Worden lokaal opgeslagen
 * (persist) en samengevoegd in `SharedProject` bij V2-export.
 *
 * Bij backend-upgrade naar V2 verhuist deze data naar `shared` zelf.
 */
export interface SharedExtra {
  postcode?: string | null;
  location?: string | null;
  notes?: string | null;
  construction_year?: number | null;
  num_storeys?: number | null;
  /** Building type met expliciete kind+subtype (V2-uitbreiding). */
  building_type?: BuildingTypeShared | null;
  /**
   * V2-only ventilatieveld: basisinfiltratie in m³/h. Niet in V1
   * `VentilationConfig`. Wordt bij `buildV2Payload` overgenomen in
   * `shared.infiltration_m3_per_h`. Backend valt terug op default_ach in
   * tojuli.rs als undefined.
   */
  infiltration_m3_per_h?: number | null;
  /**
   * V2-only ventilatieveld: mechanische toevoer in m³/h (NTA 8800 tab 11.23).
   * Niet in V1 `VentilationConfig`. Wordt bij `buildV2Payload` overgenomen
   * in `shared.mechanical_supply_m3_per_h`. Backend valt terug op
   * default_ach in tojuli.rs als undefined.
   */
  mechanical_supply_m3_per_h?: number | null;
  /**
   * V2-only ventilatieveld: mechanische afvoer in m³/h (NTA 8800 tab 11.23).
   * Niet in V1 `VentilationConfig`. Wordt bij `buildV2Payload` overgenomen
   * in `shared.mechanical_exhaust_m3_per_h`. Backend valt terug op
   * default_ach in tojuli.rs als undefined.
   */
  mechanical_exhaust_m3_per_h?: number | null;
}

export const DEFAULT_SHARED_EXTRA: SharedExtra = {
  postcode: null,
  location: null,
  notes: null,
  construction_year: null,
  num_storeys: null,
  building_type: null,
  infiltration_m3_per_h: null,
  mechanical_supply_m3_per_h: null,
  mechanical_exhaust_m3_per_h: null,
};
