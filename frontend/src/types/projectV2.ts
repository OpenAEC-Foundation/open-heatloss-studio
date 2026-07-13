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
import type { EnergyInput } from "./beng";
import type { BengGeometry } from "./bengGeometry";

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
  /**
   * Specifieke luchtdoorlatendheid q_v10;spec in dm³/(s·m²) — de norm-exacte
   * infiltratie-invoer voor de BENG-keten (`compute_beng` leest dit veld). Rust
   * spiegel is `Option<f64>` met `#[serde(default, skip_serializing_if)]`; door
   * de `.uniec3`-import gevuld, anders afwezig (backend valt terug op forfait).
   */
  q_v10_spec_dm3_s_m2?: number | null;
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
 * (transitional) een volledige ISSO 53 project-JSON. Velden worden in latere
 * fasen uitgesplitst.
 *
 * Rust spiegel: `openaec_project_shared::calcs::Iso53Inputs` met
 * `#[serde(flatten)] pub legacy: serde_json::Value`. Door de `flatten`
 * verschijnen de projectvelden (`info`, `building`, `climate`, …) INLINE
 * direct onder `calcs.isso53` — er is GEEN `legacy`-wrapper-key op de wire.
 * Daarom is dit type een vlakke record van die inline velden, niet
 * `{ legacy: ... }` (die wrapper veroorzaakte een `missing field 'info'`
 * deserialisatie-fout aan de Rust-kant).
 */
export type Iso53Inputs = Record<string, unknown>;

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
 * Bouwfase voor de minimale ventilatie-eisen volgens het Bouwbesluit
 * (ISSO 53 tabel 4.10). Spiegel van Rust
 * `isso53_core::model::enums::VentilatieBouwfase` (serde camelCase):
 * - `nieuwbouw` — strengere eisen (dm³/s per persoon);
 * - `bestaand` — bestaande bouw, soepelere eisen.
 * Voorheen hardcoded op `nieuwbouw` (~+89% Φ_V voor bestaande bouw).
 */
export type Isso53VentilatieBouwfase = "nieuwbouw" | "bestaand";

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
 * Luchtdichtheidsklasse q_v10;kar (ISSO 53 tabel 4.5).
 *
 * Spiegelt exact de serde-representatie van de Rust-enum
 * `Qv10Class` in `crates/isso53-core/src/tables/infiltration.rs`
 * (geen `rename_all`, dus de PascalCase variant-namen zijn de
 * serde-strings). q_v10;kar in dm³/(s·m² gebruiksoppervlak):
 * - `LessThan020`   — q_v10;kar < 0,20
 * - `From020To040`  — 0,20 ≤ q_v10;kar < 0,40
 * - `From040To060`  — 0,40 ≤ q_v10;kar < 0,60
 * - `From060To080`  — 0,60 ≤ q_v10;kar < 0,80
 * - `From080To100`  — 0,80 ≤ q_v10;kar ≤ 1,00
 * - `GreaterThan100` — q_v10;kar > 1,0
 */
export type Qv10Class =
  | "LessThan020"
  | "From020To040"
  | "From040To060"
  | "From060To080"
  | "From080To100"
  | "GreaterThan100";

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
  /**
   * Bouwfase die de minimale ventilatie-eisen bepaalt (ISSO 53 tabel 4.10):
   * `nieuwbouw` (strenger) vs `bestaand` (soepeler dm³/s·pp). Mapt naar de
   * Rust `VentilationConfig.bouwfase`. Default = `nieuwbouw` (backward-compat).
   */
  bouwfase: Isso53VentilatieBouwfase;
  constructionYear: number | null;
  /** Jaargemiddelde buitentemperatuur θ_me (°C). ISSO 53-default = 9,0. */
  thetaMe: number;
  /** Infiltratie-luchtdoorlatendheidsklasse q_v10;kar (ISSO 53 tabel 4.5). */
  qv10KarClass: Qv10Class;
  /**
   * Toeslag voor bedrijfsbeperking / opwarmtoeslag (ISSO 53 §4.8).
   *
   * Mapt 1:1 op de Rust `HeatingUpConfig`
   * (`crates/isso53-core/src/model/heating_up.rs`, serde camelCase).
   * De kern doet sinds Fase A (commit `e8dd82b`) een **automatische**
   * tabel-lookup (4.13 vrije afkoeling / 4.14 beperkte afkoeling) over
   * regime × opwarmtijd × verlaging, met `pWPerM2Override` als handmatige
   * override (leeg = automatisch). `warmupMinutes`/`pWPerM2` zijn vervallen.
   */
  heatingUp: Isso53HeatingUpState;
}

/**
 * Afkoel-regime tijdens de bedrijfsbeperking. Spiegelt de Rust-enum
 * `CoolingRegime` (`#[serde(rename_all="camelCase", tag="type")]`):
 * intern tagged op `type` met varianten `free` / `limited`.
 */
export type Isso53CoolingRegimeType = "free" | "limited";

/**
 * Aantal luchtwisselingen tijdens de afkoelperiode. Spiegelt de Rust-enum
 * `AirChangeRate` (`#[serde(rename_all="camelCase")]`): `low` (0,1) / `high`
 * (0,5).
 */
export type Isso53AirChangeRate = "low" | "high";

/**
 * Frontend-state voor de §4.8 toeslag-configuratie. Wordt door
 * `isso53ProjectMapper` geserialiseerd naar de Rust `HeatingUpConfig`-JSON.
 *
 * `regimeType` + de bijbehorende velden (uren bij vrije, graden bij beperkte
 * afkoeling) worden bij serialisatie samengevouwen tot de tagged
 * `CoolingRegime`-enum. `pWPerM2Override` is optioneel: `null`/leeg → de kern
 * doet de automatische tabel-lookup; een getal overschrijft die lookup.
 *
 * `legacyPWPerM2`/`legacyWarmupMinutes` houden vervallen velden uit oude
 * opgeslagen projecten vast voor migratie (zie `normalizeIsso53HeatingUp`);
 * ze worden niet meer in de UI getoond en niet meer geserialiseerd.
 */
export interface Isso53HeatingUpState {
  /** Toeslag voor bedrijfsbeperking actief — zonder dit is de toeslag 0. */
  setbackActive: boolean;
  /** Afkoel-regime: vrije (tabel 4.13) of beperkte (tabel 4.14) afkoeling. */
  regimeType: Isso53CoolingRegimeType;
  /** Vrije afkoeling: aantal úren verlaging doordeweeks (typisch 14). */
  setbackHoursWeekday: number;
  /** Vrije afkoeling: aantal úren verlaging in het weekend (typisch 62). */
  setbackHoursWeekend: number;
  /** Beperkte afkoeling: aantal gráden verlaging doordeweeks {1..5}. */
  degreesWeekday: number;
  /** Beperkte afkoeling: aantal gráden verlaging in het weekend {1..5}. */
  degreesWeekend: number;
  /** Aantal luchtwisselingen tijdens de afkoelperiode (0,1 of 0,5). */
  airChanges: Isso53AirChangeRate;
  /** Maximaal toegestane opwarmtijd doordeweeks [h]. */
  warmupHoursWeekday: number;
  /** Maximaal toegestane opwarmtijd na het weekend [h]. */
  warmupHoursWeekend: number;
  /** Mechanische toevoer uit tijdens opwarmen (§4.8.3, a=1 bij true). */
  mechanicalSupplyOff: boolean;
  /**
   * Handmatige override voor de specifieke toeslag φ_hu,i [W/m²].
   * `null` (leeg) → automatische §4.8-tabel-lookup; een getal overschrijft.
   */
  pWPerM2Override: number | null;
  /**
   * Gelijktijdigheidsfactor opwarmtoeslag (ISSO 53 §4.1/§5.1). Bereik 0..1.
   * `1,0` (default) = 100% gelijktijdigheid — alle opwarmtoeslagen treden
   * tegelijk op (backward-compatible). `< 1,0` rekent slechts dat deel van
   * Σ Φ_hu aan Φ_source toe (af te stemmen met de opdrachtgever). Grijpt aan
   * op Σ Φ_hu in de bron-berekening, niet op per-vertrek Φ_hu of het
   * rapport-totaal. Mapt naar Rust `HeatingUpConfig.simultaneity_factor`.
   */
  simultaneityFactor: number;
}

export const DEFAULT_ISSO53_BUILDING: Isso53BuildingState = {
  buildingShape: "meerlaags",
  buildingPosition: "meerlaagsTussen",
  windPressureType: "meerlaagsStandaard",
  thermalMass: "gemiddeld",
  ventilationSystem: "systemD",
  bouwfase: "nieuwbouw",
  constructionYear: null,
  thetaMe: 9.0,
  qv10KarClass: "From040To060",
  heatingUp: {
    setbackActive: false,
    regimeType: "free",
    setbackHoursWeekday: 14,
    setbackHoursWeekend: 62,
    degreesWeekday: 3,
    degreesWeekend: 3,
    airChanges: "low",
    warmupHoursWeekday: 2,
    warmupHoursWeekend: 4,
    mechanicalSupplyOff: false,
    pWPerM2Override: null,
    simultaneityFactor: 1.0,
  },
};

/**
 * Migreer een (mogelijk legacy) `heatingUp`-blob naar de actuele
 * `Isso53HeatingUpState`. Robuust tegen:
 * - vervallen velden `pWPerM2` / `warmupMinutes` (Fase A verwijderde deze);
 *   een aanwezige legacy `pWPerM2 > 0` wordt naar `pWPerM2Override` getild
 *   zodat de oude handmatige waarde behouden blijft.
 * - ontbrekende nieuwe velden → val terug op de defaults.
 *
 * Wordt door de store-rehydration aangeroepen op gepersisteerde projecten.
 */
export function normalizeIsso53HeatingUp(raw: unknown): Isso53HeatingUpState {
  const d = DEFAULT_ISSO53_BUILDING.heatingUp;
  if (raw == null || typeof raw !== "object") {
    return { ...d };
  }
  const o = raw as Record<string, unknown>;
  const num = (v: unknown, fallback: number): number =>
    typeof v === "number" && Number.isFinite(v) ? v : fallback;

  // Legacy `pWPerM2` (vervallen veld): til een positieve waarde over naar de
  // override zodat de oude handmatige toeslag niet stilzwijgend verdwijnt.
  let pWPerM2Override: number | null = null;
  if (typeof o.pWPerM2Override === "number" && Number.isFinite(o.pWPerM2Override)) {
    pWPerM2Override = o.pWPerM2Override;
  } else if (typeof o.pWPerM2 === "number" && Number.isFinite(o.pWPerM2) && o.pWPerM2 > 0) {
    pWPerM2Override = o.pWPerM2;
  }

  const regimeType: Isso53CoolingRegimeType =
    o.regimeType === "limited" ? "limited" : "free";
  const airChanges: Isso53AirChangeRate = o.airChanges === "high" ? "high" : "low";

  return {
    setbackActive: typeof o.setbackActive === "boolean" ? o.setbackActive : d.setbackActive,
    regimeType,
    setbackHoursWeekday: num(o.setbackHoursWeekday, d.setbackHoursWeekday),
    setbackHoursWeekend: num(o.setbackHoursWeekend, d.setbackHoursWeekend),
    degreesWeekday: num(o.degreesWeekday, d.degreesWeekday),
    degreesWeekend: num(o.degreesWeekend, d.degreesWeekend),
    airChanges,
    warmupHoursWeekday: num(o.warmupHoursWeekday, d.warmupHoursWeekday),
    warmupHoursWeekend: num(o.warmupHoursWeekend, d.warmupHoursWeekend),
    mechanicalSupplyOff:
      typeof o.mechanicalSupplyOff === "boolean"
        ? o.mechanicalSupplyOff
        : d.mechanicalSupplyOff,
    pWPerM2Override,
    simultaneityFactor: num(o.simultaneityFactor, d.simultaneityFactor),
  };
}

/**
 * Per-ruimte ISSO 53 sidecar-state (gebruiksFunctie + ruimteType).
 * Key = `room.id` uit V1 `Project.rooms[]`. Wordt alleen gerenderd
 * en gepersisteerd wanneer `norm === "isso53"`.
 */
export interface Isso53RoomState {
  gebruiksFunctie: Isso53GebruiksFunctie;
  ruimteType: Isso53RuimteType;
  /**
   * Override aantal personen in dit vertrek.
   * `undefined`/`null` = auto-bepaling via ISSO 53 tabel 4.11
   * (personen per m² afhankelijk van ruimtetype).
   */
  personen?: number | null;
  /**
   * Reductiefactor z voor infiltratie (ISSO 53 tabel 4.4):
   * 1.0 = 1 buitengevel of 2 niet-tegenover elkaar,
   * 0.7 = overig,
   * 0.5 = 2 buitengevels tegenover elkaar.
   */
  infiltrationReductionZ: number;
  /**
   * Temperatuurreductiefactor f_k voor het verlies naar deze ruimte wanneer
   * zij als ONVERWARMDE doelruimte fungeert (een verwarmde ruimte heeft een
   * constructie met `boundary_type === "unheated_space"` en
   * `adjacent_room_id` = deze room-id). Toegepast op álle wanden die hieraan
   * grenzen.
   *
   * `undefined` = norm-default 0,5 (isso51-consistent, `h_t_unheated_element`
   * unwrap_or(0.5)). Voor een ruimte die passief meeverwarmt (meterkast,
   * berging) kan dit lager gezet worden (bv. 0,17). Alleen zinvol voor rooms
   * die daadwerkelijk een onverwarmd-doel zijn (zie `isso53Unheated.ts`).
   */
  unheatedFactor?: number;
  /**
   * Markeer dit vertrek expliciet als ONVERWARMD. Sommige service-ruimtes
   * (techniek, afval) zijn in het model als 10 °C `adjacent_room` opgenomen
   * i.p.v. als onverwarmd doel; daardoor verliezen verwarmde buren via dunne
   * wanden onnodig warmte naar 10 °C. Met deze vlag worden álle wanden van
   * buren náár dit vertrek behandeld als grensvlak naar een onverwarmde ruimte
   * (f_k-reductie via {@link unheatedFactor}), gelijk aan een
   * `unheated_space`-doel.
   *
   * Verwijdert dit vertrek NIET uit het gebouwtotaal (aparte follow-up i.v.m.
   * dubbeltelling) — het zet alleen de buren-wanden om.
   */
  isUnheated?: boolean;
}

export const DEFAULT_ISSO53_ROOM: Isso53RoomState = {
  gebruiksFunctie: "kantoor",
  ruimteType: "verblijfsruimte",
  infiltrationReductionZ: 1.0,
};

// ---------------------------------------------------------------------------
// ProjectV2 root
// ---------------------------------------------------------------------------

export interface ProjectV2 {
  schema_version: typeof SCHEMA_VERSION_V2;
  shared: SharedProject;
  geometry: SharedGeometry;
  calcs: Calcs;
  /**
   * Additief installatie-/opwek-invoerblok voor de NTA 8800 / BENG-keten.
   * Spiegel van de Rust `ProjectV2::energy` (`Option<EnergyInput>` met
   * `#[serde(default, skip_serializing_if)]`): afwezig → geen BENG-invoer
   * (`compute_beng` geeft dan 422). Wordt door de BENG-tab gevuld.
   */
  energy?: EnergyInput | null;
  /**
   * Additief gevel-georiënteerd geometrie-invoerblok (F6). Spiegel van de Rust
   * `ProjectV2::beng_geometry` (`Option<BengGeometry>` met `#[serde(default,
   * skip_serializing_if)]`): afwezig → geen gevel-geometrie, de backend valt
   * terug op de room-geometrie. De `geometry_bridge` prefereert dit blok wanneer
   * het aanwezig is. Wordt door de BENG-tab (geometrie-subtab) gevuld.
   */
  beng_geometry?: BengGeometry | null;
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
   * Specifieke luchtdoorlatendheid q_v10;spec in dm³/(s·m²). V2-only; niet in
   * V1 `Building` (dat kent alleen de ISSO 51 `qv10` in dm³/(s·m²)-schaal met
   * andere semantiek). Door de `.uniec3`-import gevuld en bij `buildV2Payload`
   * overgenomen in `shared.q_v10_spec_dm3_s_m2` zodat de BENG-recompute
   * dezelfde infiltratie gebruikt als de certified export.
   */
  q_v10_spec_dm3_s_m2?: number | null;
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
  q_v10_spec_dm3_s_m2: null,
  infiltration_m3_per_h: null,
  mechanical_supply_m3_per_h: null,
  mechanical_exhaust_m3_per_h: null,
};
