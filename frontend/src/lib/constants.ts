/** API base URL prefix. */
export const API_PREFIX = "/api/v1";

/** Room function labels (NL). */
export const ROOM_FUNCTION_LABELS: Record<string, string> = {
  living_room: "Woonkamer",
  kitchen: "Keuken",
  bedroom: "Slaapkamer",
  bathroom: "Badkamer",
  toilet: "Toilet",
  hallway: "Gang/entree",
  landing: "Overloop",
  storage: "Berging",
  attic: "Zolder",
  custom: "Aangepast",
};

/** Room function default temperatures. */
export const ROOM_FUNCTION_TEMPERATURES: Record<string, number> = {
  living_room: 20,
  kitchen: 20,
  bedroom: 20,
  bathroom: 22,
  toilet: 15,
  hallway: 15,
  landing: 15,
  storage: 5,
  attic: 20,
};

/** Building type labels (NL). */
export const BUILDING_TYPE_LABELS: Record<string, string> = {
  detached: "Vrijstaand",
  semi_detached: "Twee-onder-een-kap",
  terraced: "Tussenwoning",
  end_of_terrace: "Hoekwoning",
  porch: "Portiekwoning",
  gallery: "Galerijwoning",
  stacked: "Gestapeld",
};

/** Ventilation system type labels (NL). */
export const VENTILATION_SYSTEM_LABELS: Record<string, string> = {
  system_a: "Systeem A (natuurlijk)",
  system_b: "Systeem B (mech. toevoer)",
  system_c: "Systeem C (mech. afvoer)",
  system_d: "Systeem D (gebalanceerd)",
  system_e: "Systeem E (combinatie)",
};

/** Security class labels. */
export const SECURITY_CLASS_LABELS: Record<string, string> = {
  a: "Klasse A (c_z = 0)",
  b: "Klasse B (c_z = 0,5)",
  c: "Klasse C (c_z = 1,0)",
};

/** Boundary type labels (NL). */
export const BOUNDARY_TYPE_LABELS: Record<string, string> = {
  exterior: "Buiten",
  unheated_space: "Onverwarmd",
  adjacent_room: "Aangrenzend",
  adjacent_building: "Naburig gebouw",
  ground: "Grond",
  water: "Water",
};

/** Boundary type color keys for Tailwind classes. */
export const BOUNDARY_COLORS: Record<string, string> = {
  exterior: "blue",
  unheated_space: "purple",
  adjacent_room: "green",
  adjacent_building: "amber",
  ground: "stone",
  water: "teal",
};

/**
 * Default engineering-aanname voor ontwerp-watertemperatuur (°C).
 * Geen norm-waarde; conservatief voor Nederlandse binnenwateren in winterconditie.
 */
export const DEFAULT_THETA_WATER = 5;

/** Vertical position labels (NL). */
export const VERTICAL_POSITION_LABELS: Record<string, string> = {
  wall: "Wand",
  floor: "Vloer",
  ceiling: "Plafond",
};

/** Frost protection type labels (NL). ISSO 51 Tabel 2.14 (erratum). */
export const FROST_PROTECTION_LABELS: Record<string, string> = {
  unknown: "Onbekend (θ_t = 10°C)",
  central_reduced_speed: "Centraal — toerenverlaging (θ_t = 10°C)",
  central_enthalpy: "Centraal — enthalpiewisselaar (θ_t = 12°C)",
  central_preheating: "Centraal — voorverwarming (θ_t = 16°C)",
  decentral_reduced_speed: "Decentraal — toerenverlaging (θ_t = 10°C)",
  decentral_enthalpy: "Decentraal — enthalpiewisselaar (θ_t = 12°C)",
  decentral_preheating: "Decentraal — voorverwarming (θ_t = 14°C)",
  electric_preheating: "Elektrisch voorverwarmen (θ_t = 5°C)",
};

/** Supply temperatures per frost protection type (°C). ISSO 51 Tabel 2.14 (erratum). */
export const FROST_PROTECTION_SUPPLY_TEMP: Record<string, number> = {
  unknown: 10,
  central_reduced_speed: 10,
  central_enthalpy: 12,
  central_preheating: 16,
  decentral_reduced_speed: 10,
  decentral_enthalpy: 12,
  decentral_preheating: 14,
  electric_preheating: 5,
};

/** Heating system labels (NL). */
export const HEATING_SYSTEM_LABELS: Record<string, string> = {
  local_gas_heater: "Gaskachel",
  ir_panel_wall: "IR paneel (wand)",
  ir_panel_ceiling: "IR paneel (plafond)",
  radiator_ht: "Radiator HT (>50°C)",
  radiator_lt: "Radiator LT (≤50°C)",
  ceiling_heating: "Plafondverwarming",
  wall_heating: "Wandverwarming",
  plinth_heating: "Plintverwarming",
  floor_heating_with_radiator_ht: "Vloerverw. + radiator HT",
  floor_heating_with_radiator_lt: "Vloerverw. + radiator LT",
  floor_heating_main_high: "Vloerverw. (≥27°C)",
  floor_heating_main_low: "Vloerverw. (<27°C)",
  floor_and_wall_heating: "Vloer- + wandverwarming",
  fan_convector: "Fanconvector",
};
