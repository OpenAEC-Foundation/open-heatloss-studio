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
};

/** Boundary type color keys for Tailwind classes. */
export const BOUNDARY_COLORS: Record<string, string> = {
  exterior: "blue",
  unheated_space: "purple",
  adjacent_room: "green",
  adjacent_building: "amber",
  ground: "stone",
};

/** Vertical position labels (NL). */
export const VERTICAL_POSITION_LABELS: Record<string, string> = {
  wall: "Wand",
  floor: "Vloer",
  ceiling: "Plafond",
};

/** Heating system labels (NL). */
export const HEATING_SYSTEM_LABELS: Record<string, string> = {
  local_gas_heater: "Gaskachel",
  ir_panel_wall: "IR paneel (wand)",
  ir_panel_ceiling: "IR paneel (plafond)",
  radiator_ht: "Radiator HT (>50\u00B0C)",
  radiator_lt: "Radiator LT (\u226450\u00B0C)",
  ceiling_heating: "Plafondverwarming",
  wall_heating: "Wandverwarming",
  plinth_heating: "Plintverwarming",
  floor_heating_with_radiator_ht: "Vloerverw. + radiator HT",
  floor_heating_with_radiator_lt: "Vloerverw. + radiator LT",
  floor_heating_main_high: "Vloerverw. (\u226527\u00B0C)",
  floor_heating_main_low: "Vloerverw. (<27\u00B0C)",
  floor_and_wall_heating: "Vloer- + wandverwarming",
  fan_convector: "Fanconvector",
};
