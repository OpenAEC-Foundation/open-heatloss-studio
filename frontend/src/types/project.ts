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

/**
 * Aggregatiemethode voor `Φ_basis_gebouw` op gebouwniveau.
 *
 * - `vabi_compat` (default): Φ_T,iae NIET opgenomen in Φ_basis_gebouw
 *   (Vabi-conventie, markt-compatible).
 * - `norm_strict`: Φ_T,iae WEL in Φ_basis_gebouw conform ISSO 51:2023 §3.5.1
 *   letterlijk. Geeft ~17% hogere connection_capacity.
 *
 * Default in Rust core = `vabi_compat`. Veld in `Building` is optioneel
 * (serde default in Rust).
 */
export type AggregationMethod = "vabi_compat" | "norm_strict";

/**
 * Regeltype van de verwarmingsinstallatie — ISSO 51:2023 §4.3.
 *
 * Bepaalt hoe de opwarmtoeslag `Φ_hu` wordt berekend:
 * - `per_zone` — §4.3.1 regeling per verblijfsgebied → `Φ_hu,i = P × A_g`
 *   (Formule 4.15). Default voor nieuwbouw.
 * - `self_learning` — §4.3.2 zelflerende regeling → `Φ_hu,i = 0` (p.70).
 * - `room_thermostat` — §4.3.3 kamerthermostaat (bestaande-bouw, buiten de
 *   huidige nieuwbouw-scope, 5 W/m² fallback).
 */
export type HeatingControlType = "per_zone" | "self_learning" | "room_thermostat";

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

/**
 * Verwarmingssysteem-keys voor ISSO 51 (woningen). snake_case, mirror van
 * de Rust `HeatingSystem` enum in `crates/isso51-core/src/model/enums.rs`.
 * 14 variants — gebruikt voor Δθ₁/Δθ₂/Δθᵥ correcties (ISSO 51 Tabel 2.12).
 */
export type HeatingSystemIsso51 =
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

/**
 * Verwarmingssysteem-keys voor ISSO 53 (utiliteit). camelCase, mirror van
 * de Rust `HeatingSystem` enum in `crates/isso53-core/src/model/enums.rs`.
 * 12 variants — gebruikt voor Δθ-correcties (ISSO 53 Tabel 2.3).
 *
 * NB: `radiatorenConvHtEnLuchtverwarming` is een gecombineerde categorie
 * die zowel HT-radiatoren als luchtverwarming dekt (norm-correct, zelfde
 * Δθ₂ = -1K). Er is geen losse `Luchtverwarming`-variant in ISSO 53.
 */
export type HeatingSystemIsso53 =
  | "lokaleVerwarming"
  | "radiatorenConvHtEnLuchtverwarming"
  | "radiatorenConvLt"
  | "plafondverwarming"
  | "wandverwarming"
  | "plintverwarming"
  | "vloerverwarmingPlusHtRadi"
  | "vloerverwarmingPlusLtRadi"
  | "vloerverwarming"
  | "vloerverwarmingPlusWandverwarming"
  | "betonkernactivering"
  | "ventilatorgedrevenConvRadi";

/**
 * Union van beide norm-specifieke verwarmingssysteem-types. Gebruikt waar
 * shared code (Building.default_heating_system, Room.heating_system) zowel
 * een ISSO 51- als een ISSO 53-project moet kunnen vertegenwoordigen.
 *
 * UI-componenten moeten de juiste subset filteren via
 * `getHeatingSystemLabels(norm)` uit `lib/constants.ts`. Norm-switch
 * mappings in `lib/normSwitch.ts` converteren tussen de twee sets.
 */
export type HeatingSystem = HeatingSystemIsso51 | HeatingSystemIsso53;

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

/**
 * Type randafstandhouder voor de Ψ_g-waarde van de beglazingsrand.
 * Mirror van het Rust `Spacer`-enum (`construction.rs`) en van
 * `nta8800_tables::glazing_edge::SpacerKind`. snake_case serialisatie.
 */
export type Spacer =
  | "aluminium"
  | "stainless"
  | "warm_edge_polymer"
  | "warm_edge_foam";

/**
 * Onderbouwing van de samengestelde raam-U-waarde U_w.
 *
 * Volgens NEN-EN-ISO 10077-1:
 * `U_w = (ΣA_g·U_g + ΣA_f·U_f + Σl_g·Ψ_g) / (ΣA_g + ΣA_f)`.
 *
 * Standaard-detailniveau: uniform kozijn — één U_g voor alle ruiten, één
 * U_f, uniforme profielbreedte. Afgeleide waarden (`a_g_m2`, `a_f_m2`,
 * `l_g_m`, `u_w`) worden gecachet maar zijn herberekenbaar uit de invoer.
 * Niet naar de Rust rekenkern gestuurd — uitsluitend persistente
 * onderbouwing op het kozijn-element.
 */
export interface UwBreakdown {
  /** Raambreedte buitenwerks in mm. */
  width_mm: number;
  /** Raamhoogte buitenwerks in mm. */
  height_mm: number;
  /** Uniforme profielbreedte (buitenkozijn + tussenprofielen) in mm. */
  frame_width_mm: number;
  /** Aantal ruit-kolommen (ruit-indeling), standaard 1. */
  pane_columns: number;
  /** Aantal ruit-rijen (ruit-indeling), standaard 1. */
  pane_rows: number;
  /** Glas-U-waarde U_g in W/(m²·K) — handmatige invoer van de glasleverancier. */
  u_g: number;
  /**
   * Herkomst van U_g — vrije-tekst label van de gekozen glasopbouw.
   * Afwezig bij handmatige invoer. Vrije tekst, geen catalogus-id.
   */
  u_g_source?: string;
  /** Profiel-U-waarde U_f in W/(m²·K) — handmatige invoer van de profielfabrikant. */
  u_f: number;
  /**
   * Herkomst van U_f — vrije-tekst label van het gekozen profielsysteem.
   * Afwezig bij handmatige invoer. Vrije tekst, geen catalogus-id.
   */
  u_f_source?: string;
  /**
   * Type randafstandhouder voor de Ψ_g-tabelwaarde.
   * `null`/afwezig = volledig handmatige Ψ_g-invoer.
   */
  spacer?: Spacer | null;
  /** Effectieve lineaire warmtedoorgangscoëfficiënt Ψ_g in W/(m·K). */
  psi_g: number;
  /** `true` wanneer `psi_g` een handmatige override op de spacer-tabelwaarde is. */
  psi_g_is_manual: boolean;
  /** Afgeleid: totale glasoppervlakte ΣA_g in m². Gecachet, herberekenbaar. */
  a_g_m2: number;
  /** Afgeleid: totale profieloppervlakte ΣA_f in m². Gecachet, herberekenbaar. */
  a_f_m2: number;
  /** Afgeleid: totale zichtbare glasrand-omtrek Σl_g in m. Gecachet, herberekenbaar. */
  l_g_m: number;
  /** Resultaat: samengestelde raam-U-waarde U_w in W/(m²·K). */
  u_w: number;
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
  /**
   * Onderbouwing van de samengestelde raam-U-waarde (U_w). Optioneel —
   * alleen aanwezig op kozijn-/vullings-elementen waarvoor de U_w-calculator
   * is gebruikt. De rekenkern negeert dit veld; alleen `u_value` is de
   * rekeningang.
   */
  uw_breakdown?: UwBreakdown;
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
  /** Bron-kamer ID waar ventilatielucht vandaan komt (overstroom).
   *  `null`/undefined = gevelrooster/buitenlucht (default).
   *  String met room ID = overstroom uit die kamer. UI resolveert dit naar
   *  `supply_air_temperature` op basis van bron-kamer's θ_i. */
  air_source_room_id?: string | null;
  clamp_positive?: boolean;
}

export interface Building {
  building_type: BuildingType;
  qv10: number;
  total_floor_area: number;
  security_class: SecurityClass;
  /**
   * Of alle verwarmde vertrekken (ook verdiepingen) vloerverwarming hebben.
   * Zo ja → `Φ_hu = 0` (ISSO 51:2023 p.70: vloerverwarming reageert traag,
   * nachtverlaging is dan niet zinvol). Default = `false`.
   */
  all_floor_heating?: boolean;
  /**
   * Of de woning ná 2015 is gebouwd (nieuwbouw). Stuurt de afkoeling-
   * bepaling: nieuwbouw → 2 K (resp. 1 K bij Ū≤0,5). Default = `true`
   * (nieuwbouw-scope; bestaande bouw met Afb. 2.7-grafiek is nog niet
   * geïmplementeerd — zie TODO in `calc/heating_up.rs`).
   */
  built_after_2015?: boolean;
  /**
   * Effectieve warmtecapaciteit `c_eff` van het gebouw [Wh/K] — bepaalt de
   * gebouwzwaarte voor Tabel 2.10 (`c_eff ≤ 70` → ZL+L+M, anders Z).
   *
   * Optioneel: `null` → conservatieve aanname "zwaar" (`ThermalMass::Heavy`,
   * hoogste toeslag). Forfaitair te bepalen via ISSO 51:2023 Tabel 2.1 of
   * Formule 2.46 (`c_eff = C_eff / A_g`).
   */
  c_eff?: number | null;
  /**
   * Regeltype van de verwarmingsinstallatie (ISSO 51:2023 §4.3).
   *
   * Stuurt de opwarmtoeslag-tak: `per_zone` → `Φ_hu = P × A_g`,
   * `self_learning` → `Φ_hu = 0`, `room_thermostat` → bestaande-bouw (buiten
   * scope, 5 W/m² fallback). Default = `per_zone` (nieuwbouw, regeling per
   * verblijfsgebied — de meest voorkomende nieuwbouw-keuze).
   */
  heating_control_type?: HeatingControlType;
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
  /**
   * Aggregatiemethode voor Φ_basis_gebouw op gebouwniveau. Zie
   * `AggregationMethod`. Optioneel — Rust core gebruikt `serde(default)` =
   * `vabi_compat` wanneer afwezig. UI volgt later (apart spoor).
   */
  aggregation_method?: AggregationMethod;
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
  /** Optionele footer-afbeelding die op elke content-pagina onderaan
   * wordt gerenderd (boven het paginanummer-text-footer). Hergebruikt
   * het CoverImage-shape (base64 + media_type). */
  footer_image?: CoverImage | null;
  /** Optionele header-afbeelding bovenaan elke content-pagina (boven de
   * accent-lijn). Bedoeld voor bedrijfslogo / bureaubeeldmerk. */
  header_image?: CoverImage | null;
  /** Per-project rapport opmaak overrides — wanneer afwezig, defaults. */
  report_style?: ReportStyle | null;
}

/** Opmaak-tokens voor het PDF rapport (marges, accent-kleur).
 * Alle velden optioneel — Rust-side vult defaults in wanneer afwezig.
 *
 * Lettertype is V3 (vereist extra TTF files in resources/fonts/ —
 * momenteel alleen LiberationSans geregistreerd). */
export interface ReportStyle {
  /** Bovenmarge in mm. Default 20. */
  margin_top_mm?: number | null;
  /** Ondermarge in mm. Default 28 (laat plek voor footer + paginanummer). */
  margin_bottom_mm?: number | null;
  /** Horizontale marges (links + rechts) in mm. Default 15. */
  margin_horizontal_mm?: number | null;
  /** Accent-kleur als hex (zonder #). Default "0F766E" (OHS teal). */
  accent_color_hex?: string | null;
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
