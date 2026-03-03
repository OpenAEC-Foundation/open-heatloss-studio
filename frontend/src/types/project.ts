// Generated from schemas/v1/project.schema.json
// Re-generate: npm run generate-types

export type BoundaryType =
  | "exterior"
  | "unheated_space"
  | "adjacent_room"
  | "adjacent_building"
  | "ground";

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
  ventilation_rate: number;
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
}

export interface DesignConditions {
  theta_e?: number;
  theta_b_residential?: number;
  theta_b_non_residential?: number;
  wind_factor?: number;
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

export interface ProjectInfo {
  name: string;
  project_number?: string | null;
  address?: string | null;
  client?: string | null;
  date?: string | null;
  engineer?: string | null;
  notes?: string | null;
}

export interface Project {
  info: ProjectInfo;
  building: Building;
  climate: DesignConditions;
  ventilation: VentilationConfig;
  rooms: Room[];
}
