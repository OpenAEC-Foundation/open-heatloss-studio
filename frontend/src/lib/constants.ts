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

/**
 * Single source of truth voor design-temperatures per functie (ISSO 51).
 * Importeer deze constante; creëer géén lokale kopieën. Wijzigingen hier
 * propageren automatisch naar chart-componenten (`components/charts/deltaT.ts`)
 * en PDF-rapport SVG's (`lib/reportCharts.ts`).
 */
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

/**
 * Aggregatiemethode-labels (NL). Default `vabi_compat` matcht Vabi-software
 * output (markt-conventie). `norm_strict` volgt ISSO 51:2023 §3.5.1 letterlijk
 * en levert ~17% hoger aansluitvermogen.
 */
export const AGGREGATION_METHOD_LABELS: Record<string, string> = {
  vabi_compat: "Vabi-conform (markt-default)",
  norm_strict: "Norm-strict (ISSO 51:2023 §3.5.1 letterlijk)",
};

/** Default aggregatiemethode — gelijk aan Rust core `serde(default)`. */
export const DEFAULT_AGGREGATION_METHOD = "vabi_compat" as const;

/**
 * Infiltratiemethode-labels (NL). Bepaalt de infiltratie-rekenketen (Φ_i).
 * Default `per_exterior_area` matcht de Rust core `serde(default)` (legacy 2017).
 * `vabi_compat`/`measured_qv10` voor Vabi-matching.
 */
export const INFILTRATION_METHOD_LABELS: Record<string, string> = {
  per_exterior_area: "Per geveloppervlak (legacy 2017)",
  per_floor_area: "Per vloeroppervlak (legacy)",
  vabi_compat: "Vabi-conform (Tabel 2.8-keten)",
  nta8800_strict: "NTA 8800 strikt",
  measured_qv10: "Gemeten qv10 (blower-door)",
};

/** Default infiltratiemethode — gelijk aan Rust core `serde(default)`. */
export const DEFAULT_INFILTRATION_METHOD = "per_exterior_area" as const;

/**
 * Regeltype van de verwarmingsinstallatie — ISSO 51:2023 §4.3. Stuurt de
 * opwarmtoeslag-tak Φ_hu (per zone → P×A_g, zelflerend → 0, kamerthermostaat
 * → bestaande-bouw fallback).
 */
export const HEATING_CONTROL_TYPE_LABELS: Record<string, string> = {
  per_zone: "Per zone (verblijfsgebied)",
  self_learning: "Zelflerend",
  room_thermostat: "Kamerthermostaat (bestaande bouw)",
};

/** Default regeltype — gelijk aan Rust core `serde(default)` = `per_zone`. */
export const DEFAULT_HEATING_CONTROL_TYPE = "per_zone" as const;

/** Heating system labels (NL) voor ISSO 51 (woningen). */
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

/**
 * Heating system labels (NL) voor ISSO 53 (utiliteit). camelCase keys
 * matchen de Rust `crates/isso53-core/src/model/enums.rs::HeatingSystem`
 * enum. `radiatorenConvHtEnLuchtverwarming` is gecombineerd in de norm —
 * één Δθ₂-correctie geldt voor zowel HT-radiatoren als luchtverwarming.
 */
export const HEATING_SYSTEM_LABELS_ISSO53: Record<string, string> = {
  lokaleVerwarming: "Lokale verwarming (gaskachel/heater)",
  radiatorenConvHtEnLuchtverwarming: "Radiator HT / Luchtverwarming (>50°C)",
  radiatorenConvLt: "Radiator LT (≤50°C)",
  plafondverwarming: "Plafondverwarming",
  wandverwarming: "Wandverwarming",
  plintverwarming: "Plintverwarming",
  vloerverwarmingPlusHtRadi: "Vloerverw. + radiator HT",
  vloerverwarmingPlusLtRadi: "Vloerverw. + radiator LT",
  vloerverwarming: "Vloerverwarming",
  vloerverwarmingPlusWandverwarming: "Vloer- + wandverwarming",
  betonkernactivering: "Betonkernactivering",
  ventilatorgedrevenConvRadi: "Ventilatorgedreven convector",
};

/**
 * Geef de juiste verwarmingssysteem-labels terug op basis van de actieve
 * norm. UI-componenten in zowel ISSO 51- als ISSO 53-context gebruiken
 * deze helper om de dropdown-opties consistent te filteren.
 */
export function getHeatingSystemLabels(
  norm: "isso51" | "isso53",
): Record<string, string> {
  return norm === "isso53" ? HEATING_SYSTEM_LABELS_ISSO53 : HEATING_SYSTEM_LABELS;
}
