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

export interface TojuliInputs {
  quick_check?: Record<string, unknown> | null;
}

export interface Calcs {
  isso51?: Iso51Inputs | null;
  tojuli?: TojuliInputs | null;
}

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
}

export const DEFAULT_SHARED_EXTRA: SharedExtra = {
  postcode: null,
  location: null,
  notes: null,
  construction_year: null,
  num_storeys: null,
  building_type: null,
};
