/**
 * Handgeschreven TypeScript-spiegel van de Rust BENG-typen.
 *
 * Bron (serde-casing is normatief — spiegel exact):
 *  - Resultaat: `crates/openaec-project-shared/src/beng/mod.rs`
 *    (`BengResult`, `IndicatorReport`, `TojuliBengSummary`, `TojuliMethod`,
 *    `ServiceBreakdownKwhM2`).
 *  - Invoer: `crates/openaec-project-shared/src/energy.rs`
 *    (`EnergyInput` + sub-DTO's).
 *
 * Casing-conventies uit de Rust `#[serde(rename_all = ...)]`-attributen:
 *  - Struct-velden: snake_case (serde-default) — bv. `max_tojuli_k`,
 *    `service_breakdown_kwh_m2`, `wtw_efficiency`.
 *  - Generator/methode-enums: snake_case — `heat_pump_air`, `per_orientation`.
 *  - Ventilatiesysteem + BACS-klasse: UPPERCASE — `A`..`E`, `A`..`D`.
 *
 * Optionele velden (`Option<T>` met `skip_serializing_if`) → `field?: T | null`;
 * ze mogen bij het verzenden ontbreken. NIET via `npm run generate-types`
 * regenereren (kapotte pipeline) — dit bestand blijft handmatig.
 */

// ---------------------------------------------------------------------------
// Resultaat-typen (beng/mod.rs)
// ---------------------------------------------------------------------------

/** Eén BENG-indicator met (indien beschikbaar) grenswaarde en pass/fail. */
export interface IndicatorReport {
  /** Berekende waarde (BENG 1/2 in kWh/(m²·jr), BENG 3 in %). */
  value: number;
  /** Grenswaarde uit het Bbl (art. 4.149), of `null` als niet-geverifieerd. */
  limit: number | null;
  /** Voldoet de indicator? `null` als er geen grenswaarde is. */
  pass: boolean | null;
}

/** Methode waarmee de TOjuli-indicator is bepaald (serde snake_case). */
export type TojuliMethod = "actively_cooled" | "per_orientation";

/** TOjuli-oververhittingssamenvatting (§5.7 / Bbl 4.149b). */
export interface TojuliBengSummary {
  /** Maatgevende TOjuli [K]. */
  max_tojuli_k: number;
  /** Grenswaarde [K] (Bbl art. 4.149b lid 1). */
  limit_k: number;
  /** Is de rekenzone actief gekoeld? */
  actively_cooled: boolean;
  /** Voldoet de zone? `null` blijft gereserveerd voor niet-toetsbare gevallen. */
  pass: boolean | null;
  /** Gebruikte bepalingsmethode. */
  method: TojuliMethod;
}

/**
 * Primair energiegebruik per dienst in kWh/(m²·jr) — negatief voor PV
 * (netto opwekking).
 */
export interface ServiceBreakdownKwhM2 {
  heating: number;
  cooling: number;
  dhw: number;
  ventilation_aux: number;
  /** Verlichting (0 voor de woonfunctie). */
  lighting: number;
  /** PV-opwekking (negatief). */
  pv: number;
}

/** Volledig BENG-resultaat voor een ProjectV2. */
export interface BengResult {
  /** BENG 1 — energiebehoefte [kWh/(m²·jr)]. */
  beng1: IndicatorReport;
  /** BENG 2 — karakteristiek primair fossiel energiegebruik [kWh/(m²·jr)]. */
  beng2: IndicatorReport;
  /** BENG 3 — aandeel hernieuwbare energie [%]. */
  beng3: IndicatorReport;
  /** TOjuli-oververhitting. */
  tojuli: TojuliBengSummary;
  /** Energielabel (A++++ t/m G). */
  energy_label: string;
  /** Hernieuwbaar aandeel [0..=1]. */
  renewable_share: number;
  /** CO₂-uitstoot [kg/(m²·jr)]. */
  co2_kg_per_m2: number;
  /** Gebruiksoppervlak A_g [m²]. */
  a_g_m2: number;
  /** Verliesoppervlak A_ls [m²] (thermische schil). */
  a_ls_m2: number;
  /** Vormfactor A_ls/A_g. */
  als_ag_ratio: number;
  /** Primair energiegebruik per dienst [kWh/(m²·jr)]. */
  service_breakdown_kwh_m2: ServiceBreakdownKwhM2;
  /** Bekende vereenvoudigingen/stubs die op dit resultaat van toepassing zijn. */
  notes: string[];
}

// ---------------------------------------------------------------------------
// Invoer-DTO (energy.rs) — `ProjectV2::energy`
// ---------------------------------------------------------------------------

/** Opwekker-type verwarming (serde snake_case). */
export type HeatGeneratorType =
  | "hr_boiler"
  | "heat_pump_air"
  | "heat_pump_ground"
  | "electric_resistance"
  | "district_heating";

/** HR-ketelklasse (serde snake_case). */
export type HrBoilerClass = "hr100" | "hr104" | "hr107";

/** Afgiftesysteem verwarming (serde snake_case). */
export type HeatEmissionType =
  | "radiator_high_temp"
  | "radiator_low_temp"
  | "floor_heating"
  | "air_heating"
  | "radiant_panel";

/** Verwarmingssysteem-invoer (NTA 8800 H.9). */
export interface HeatingInput {
  generator: HeatGeneratorType;
  /** Seizoens-COP — alleen warmtepomp-varianten. */
  cop?: number | null;
  /** HR-ketelklasse — alleen `hr_boiler`. */
  hr_class?: HrBoilerClass | null;
  /** Grensvlak-factor — alleen `district_heating`. */
  district_factor?: number | null;
  emission?: HeatEmissionType | null;
  distribution_efficiency?: number | null;
  control_factor?: number | null;
  /** Dekkingsgraad (0..=1); Rust-default 1,0. */
  coverage_fraction?: number;
}

/** Opwekker-type warm tapwater (serde snake_case). */
export type DhwGeneratorType =
  | "hr_combi_boiler"
  | "electric_boiler"
  | "heat_pump"
  | "district_heating";

/** Douchewater-warmteterugwinning. */
export interface DwtwInput {
  /** Netto thermisch rendement η (0..=1). */
  efficiency: number;
  /** Aandeel douche in Q_W;nd (0..=1); Rust-default 0,4. */
  douche_aandeel?: number | null;
}

/** Warm-tapwater-invoer (NTA 8800 H.13). */
export interface DhwInput {
  generator: DhwGeneratorType;
  /** η_W;gen of SCOP_W (warmtepomp). */
  efficiency?: number | null;
  /**
   * Douchewater-WTW. `null` (clear-conventie van dit blok) én afwezig
   * (`undefined`, zoals Rust-serde bij `skip_serializing_if` levert) betekenen
   * beide "geen DWTW-unit" — lees altijd via `?? undefined`, zodat er geen
   * runtime-verschil tussen de twee bestaat.
   */
  dwtw?: DwtwInput | null;
  /** Zonneboiler aanwezig (V2-scope in de crate; nu louter invoer). */
  has_solar_boiler?: boolean;
  solar_boiler_fraction?: number | null;
}

/**
 * Ventilatiesysteem-type A–E (serde UPPERCASE). NTA 8800-conventie:
 * B = mechanische toevoer, C = mechanische afvoer.
 */
export type EnergyVentilationSystemType = "A" | "B" | "C" | "D" | "E";

/** Ventilatie-invoer (NTA 8800 H.11). */
export interface VentilationInput {
  system: EnergyVentilationSystemType;
  /** WTW-rendement η_hr (0..=1). Aanwezigheid activeert WTW bij systeem D. */
  wtw_efficiency?: number | null;
  /** f_SFP in W/(m³/h) (NTA 8800 tab 11.23). */
  sfp_w_per_m3h?: number | null;
  bypass_enabled?: boolean;
  mechanical_supply_m3_per_h?: number | null;
  mechanical_exhaust_m3_per_h?: number | null;
  infiltration_m3_per_h?: number | null;
}

/** Koudeopwekker-type (serde snake_case). */
export type CoolingGeneratorType = "compression" | "absorption" | "free_cooling";

/** Koel-invoer (NTA 8800 H.10). Aanwezigheid van dit blok = actieve koeling. */
export interface CoolingInput {
  generator: CoolingGeneratorType;
  /** SEER/SCOP_cooling voor compressiekoeling. */
  seer?: number | null;
  /** COP voor absorptiekoeling. */
  cop?: number | null;
  /** Benuttingsfractie (0..=1) voor vrije koeling. */
  free_cooling_fraction?: number | null;
}

/** Eén PV-veld/-string (NTA 8800 H.16). */
export interface PvInput {
  id?: string | null;
  name?: string | null;
  /** Piekvermogen in kWp (> 0). */
  peak_power_kwp: number;
  /** Azimut in graden (0 = noord, 90 = oost, 180 = zuid, 270 = west). */
  azimuth_degrees: number;
  /** Hellingshoek in graden (0 = horizontaal, 90 = verticaal). */
  tilt_degrees: number;
  system_efficiency?: number | null;
  inverter_efficiency?: number | null;
  shadow_factor?: number | null;
}

/** BACS-klasse (serde UPPERCASE; NEN-EN 15232). */
export type BacsClassInput = "A" | "B" | "C" | "D";

/** Gebouwautomatisering-invoer (NTA 8800 H.15). */
export interface AutomationInput {
  bacs_class: BacsClassInput;
}

/**
 * Additief installatie-/opwek-invoerblok op `ProjectV2` (`ProjectV2::energy`).
 * Alle deelsystemen optioneel; een ontbrekend systeem = norm-forfait of nul.
 */
export interface EnergyInput {
  heating?: HeatingInput | null;
  dhw?: DhwInput | null;
  ventilation?: VentilationInput | null;
  cooling?: CoolingInput | null;
  /** PV-velden/-strings. Lege lijst = geen PV. */
  pv?: PvInput[];
  /** Gebouwautomatisering (BACS). Afwezig = referentieklasse C. */
  automation?: AutomationInput | null;
}
