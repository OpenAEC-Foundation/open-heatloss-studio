// Generated from schemas/v1/project.schema.json
// Re-generate: npm run generate-types

export type BoundaryType =
  | "exterior"
  | "unheated_space"
  | "adjacent_room"
  | "adjacent_building"
  | "ground"
  | "water";

export type BuildingType =
  | "detached"
  | "semi_detached"
  | "terraced"
  | "end_of_terrace"
  | "porch"
  | "gallery"
  | "stacked";

export type SecurityClass = "a" | "b" | "c";

export type RoomFunction =
  | "living_room"
  | "kitchen"
  | "bedroom"
  | "bathroom"
  | "toilet"
  | "hallway"
  | "landing"
  | "storage"
  | "attic"
  | "custom";

export type HeatingSystem =
  | "local_gas_heater"
  | "ir_panel_wall"
  | "ir_panel_ceiling"
  | "radiator_ht"
  | "radiator_lt"
  | "ceiling_heating"
  | "wall_heating"
  | "plinth_heating"
  | "floor_heating_with_radiator_ht"
  | "floor_heating_with_radiator_lt"
  | "floor_heating_main_high"
  | "floor_heating_main_low"
  | "floor_and_wall_heating"
  | "fan_convector";

export type VentilationSystemType =
  | "system_a"
  | "system_b"
  | "system_c"
  | "system_d"
  | "system_e";

export type FrostProtectionType =
  | "unknown"
  | "central_reduced_speed"
  | "central_enthalpy"
  | "central_preheating"
  | "decentral_reduced_speed"
  | "decentral_enthalpy"
  | "decentral_preheating"
  | "electric_preheating";

export type MaterialType = "masonry" | "non_masonry";

export type VerticalPosition = "floor" | "ceiling" | "wall";

export interface GroundParameters {
  u_equivalent: number;
  ground_water_factor?: number;
  fg2?: number;
}

export interface ConstructionElementLayer {
  materialId: string;
  /** Laagdikte in mm. */
  thickness: number;
  /**
   * Optionele lambda override [W/(m·K)]. Gebruikt door de thermal import
   * wanneer de Revit exporter een lambda meegeeft die niet via de material
   * database te matchen is. Priority in Rc-berekening:
   *   rdFixed (spouw) > lambdaOverride > material.lambda.
   */
  lambdaOverride?: number;
  /** Stijl/keper configuratie voor inhomogene lagen. */
  stud?: {
    materialId: string;
    width: number;
    spacing: number;
  };
}

export interface ConstructionElement {
  id: string;
  description: string;
  area: number;
  u_value: number;
  boundary_type: BoundaryType;
  material_type: MaterialType;
  temperature_factor?: number | null;
  adjacent_room_id?: string | null;
  adjacent_temperature?: number | null;
  vertical_position?: VerticalPosition;
  use_forfaitaire_thermal_bridge?: boolean;
  custom_delta_u_tb?: number | null;
  ground_params?: GroundParameters | null;
  has_embedded_heating?: boolean;
  /** Optioneel: laag-opbouw voor Rc/U berekening. Niet naar Rust core gestuurd. */
  layers?: ConstructionElementLayer[];
  /** Verwijzing naar ProjectConstruction in modellerStore. Niet naar Rust core gestuurd. */
  project_construction_id?: string;
  /** Verwijzing naar een CatalogEntry uit de thermal import (None voor openings/handmatige elementen). */
  catalog_ref?: string | null;
}

export interface Room {
  id: string;
  name: string;
  function: RoomFunction;
  custom_temperature?: number | null;
  floor_area: number;
  height?: number;
  constructions: ConstructionElement[];
  heating_system: HeatingSystem;
  ventilation_rate?: number | null;
  has_mechanical_exhaust?: boolean;
  has_mechanical_supply?: boolean;
  fraction_outside_air?: number;
  supply_air_temperature?: number | null;
  internal_air_temperature?: number | null;
  clamp_positive?: boolean;
}

export interface Building {
  building_type: BuildingType;
  qv10: number;
  total_floor_area: number;
  security_class: SecurityClass;
  has_night_setback?: boolean;
  warmup_time?: number;
  building_height?: number | null;
  num_floors?: number;
  /**
   * Project-brede standaard verwarmingssysteem. Wordt gebruikt bij het
   * aanmaken van nieuwe ruimten (via createRoom) en kan met één klik op
   * alle bestaande ruimten worden toegepast. Optioneel voor backward
   * compat met oude projecten; frontend valt terug op "radiator_ht".
   *
   * NOTE: Dit veld is HANDMATIG toegevoegd buiten de JSON-schema generatie
   * om (zie header comment bovenaan). Bij de volgende
   * `npm run generate-types` moet dit veld óók in het Rust `Building`
   * struct + schema landen, anders overschrijft de generator deze regel.
   * TODO: propagate default_heating_system naar Rust crates/isso51-core/src/model.
   */
  default_heating_system?: HeatingSystem;
}

export interface DesignConditions {
  theta_e?: number;
  theta_b_residential?: number;
  theta_b_non_residential?: number;
  wind_factor?: number;
  /**
   * Ontwerp-watertemperatuur voor grensvlakken aan water (°C). Geen norm-waarde;
   * engineering-aanname. Default 5 °C voor Nederlandse binnenwateren onder
   * winterconditie. Optioneel voor backward-compat met oude projecten.
   */
  theta_water?: number;
}

export interface VentilationConfig {
  system_type: VentilationSystemType;
  has_heat_recovery?: boolean;
  heat_recovery_efficiency?: number | null;
  frost_protection?: FrostProtectionType | null;
  supply_temperature?: number | null;
  has_preheating?: boolean;
  preheating_temperature?: number | null;
}

export interface CoverImage {
  /** Raw base64 encoded image data (zonder data: prefix). */
  data: string;
  /** MIME type — alleen PNG en JPEG ondersteund. */
  media_type: "image/png" | "image/jpeg";
  /** Originele bestandsnaam (optioneel, voor UX). */
  filename?: string;
}

export interface ProjectInfo {
  name: string;
  project_number?: string | null;
  address?: string | null;
  client?: string | null;
  date?: string | null;
  engineer?: string | null;
  notes?: string | null;
  cover_image?: CoverImage | null;
}

export interface Project {
  info: ProjectInfo;
  building: Building;
  climate: DesignConditions;
  ventilation: VentilationConfig;
  rooms: Room[];
  /**
   * Optionele project-brede override voor de U-waarde van kozijnen
   * (openings: categorie `kozijnen_vullingen`). Wanneer gezet (en > 0)
   * vervangt dit in de berekening de individuele `u_value` van alle
   * gekoppelde kozijn-elementen. De onderliggende per-element waarde
   * blijft in de store staan; de override wordt alleen in de rekenkern
   * toegepast via `prepareProjectForCalculation` en is daar via
   * `getEffectiveFrameUValue` uitleesbaar.
   *
   * Eenheid: W/(m²·K). Leeg / undefined = geen override (individuele
   * waarden per element).
   *
   * Niet naar Rust core gestuurd als veld: de override wordt al
   * toegepast op `u_value` voordat het project naar het backend gaat.
   */
  frameUValueOverride?: number;
}
