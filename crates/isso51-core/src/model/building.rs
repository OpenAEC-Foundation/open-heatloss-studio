//! Building/project model for ISSO 51 heat loss calculations.
//!
//! The project is the top-level container holding all information needed
//! for a complete heat loss calculation of a dwelling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::climate::DesignConditions;
use super::enums::{
    AggregationMethod, BuildingType, ConstructionVariant, DwellingClass, HeatingControlType,
    InfiltrationMethod, SecurityClass,
};
use super::room::Room;
use super::ventilation::VentilationConfig;

/// Top-level project containing all input data for the calculation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    /// Project metadata.
    pub info: ProjectInfo,

    /// Building characteristics.
    pub building: Building,

    /// Climate/design conditions.
    pub climate: DesignConditions,

    /// Ventilation system configuration.
    pub ventilation: VentilationConfig,

    /// All rooms in the dwelling.
    pub rooms: Vec<Room>,
}

/// Project metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectInfo {
    /// Project name.
    pub name: String,

    /// Project number/reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_number: Option<String>,

    /// Address of the building.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Client name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// Calculation date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Engineer performing the calculation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engineer: Option<String>,

    /// Additional notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Building characteristics that affect the calculation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Building {
    /// Type of building.
    pub building_type: BuildingType,

    /// Air tightness: q_v,10 value in dm³/s.
    /// Measured air volume flow at 10 Pa pressure difference.
    pub qv10: f64,

    /// Total usable floor area (gebruiksoppervlak) A_g in m².
    pub total_floor_area: f64,

    /// Security class for heat loss to neighbors.
    pub security_class: SecurityClass,

    /// Whether night setback / operational reduction is used.
    #[serde(default)]
    pub has_night_setback: bool,

    /// Desired warm-up time in hours (typically 1 or 2).
    #[serde(default = "default_warmup_time")]
    pub warmup_time: f64,

    /// Building height in m (buitenafmetingen).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_height: Option<f64>,

    /// Number of floors above ground.
    #[serde(default = "default_floors")]
    pub num_floors: u32,

    /// Infiltration calculation method.
    /// PerExteriorArea (default): q_i = qi_spec × ΣA_exterior (ISSO 51:2023)
    /// PerFloorArea: q_i = qi_spec × A_floor (ISSO 51:2024)
    /// VabiCompat / Nta8800Strict: zie `InfiltrationMethod` doc.
    #[serde(default)]
    pub infiltration_method: InfiltrationMethod,

    /// Woningclassificatie volgens ISSO 51:2023 Tabel 2.8 (qi,spec keying).
    ///
    /// Optioneel — alleen relevant bij `infiltration_method = VabiCompat`.
    /// `None` → bestaande legacy-rekenketen blijft van toepassing.
    /// Bestaande fixture-JSONs zonder dit veld blijven werken via `serde(default)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dwelling_class: Option<DwellingClass>,

    /// Uitvoeringsvariant volgens NTA 8800 Tabel 11.14 (tussen / kop / vrijstaand).
    ///
    /// Optioneel — alleen relevant bij `infiltration_method = Nta8800Strict`.
    /// `None` → val terug op `f_type = 1.0` (tussenwoning-equivalent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction_variant: Option<ConstructionVariant>,

    /// Bouwjaar van het gebouw — driver voor NTA 8800 Tabel 11.13 `f_y`
    /// bouwjaarcorrectie.
    ///
    /// Optioneel — `None` → `f_y = 1.0` (onbekend bouwjaar, neutrale factor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction_year: Option<u16>,

    /// Aggregatiemethode voor `Φ_basis_gebouw` op gebouwniveau.
    ///
    /// Default = `VabiCompat` (markt-conventie, sluit `Φ_T,iae` uit). Voor
    /// strikte ISSO 51:2023 §3.5.1 audits → `NormStrict`. Zie
    /// `AggregationMethod` doc voor verschillen (~17% op connection_capacity).
    #[serde(default)]
    pub aggregation_method: AggregationMethod,

    // -------------------------------------------------------------------
    // Opwarmtoeslag (Φ_hu) — ISSO 51:2023 §2.5.8 / §4.3 (Ronde 5, A1+A2).
    // Alle velden hebben serde-defaults zodat bestaande project-JSONs en de
    // Vabi-fixtures (die `has_night_setback=false` hebben → Φ_hu=0) ongewijzigd
    // blijven werken.
    // -------------------------------------------------------------------
    /// Regeltype van de verwarmingsinstallatie (ISSO 51:2023 §4.3).
    ///
    /// Stuurt de opwarmtoeslag-tak: `PerZone` → `Φ_hu = P × A_g`,
    /// `SelfLearning` → `Φ_hu = 0`, `RoomThermostat` → bestaande-bouw
    /// (buiten scope, 5 W/m² fallback). Default = `PerZone` (nieuwbouw,
    /// regeling per verblijfsgebied — de meest voorkomende nieuwbouw-keuze).
    #[serde(default)]
    pub heating_control_type: HeatingControlType,

    /// Effectieve warmtecapaciteit `c_eff` van het gebouw [Wh/(m³·K)] — bepaalt de
    /// gebouwzwaarte voor Tabel 2.10 (`c_eff ≤ 70` → ZL+L+M, anders Z).
    ///
    /// Optioneel: `None` → conservatieve aanname "zwaar" (`ThermalMass::Heavy`,
    /// hoogste toeslag). Forfaitair te bepalen via ISSO 51:2023 Tabel 2.1 of
    /// Formule 2.46 (`c_eff = C_eff / V`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_eff: Option<f64>,

    /// Of de woning ná 2015 is gebouwd (nieuwbouw). Stuurt de afkoeling-
    /// bepaling: nieuwbouw → 2 K (resp. 1 K bij Ū≤0,5). Default = `true`
    /// (nieuwbouw-scope; bestaande bouw met Afb. 2.7-grafiek is nog niet
    /// geïmplementeerd — zie TODO in `calc/heating_up.rs`).
    #[serde(default = "default_true")]
    pub built_after_2015: bool,

    /// Of alle verwarmde vertrekken (ook verdiepingen) vloerverwarming hebben.
    /// Zo ja → `Φ_hu = 0` (ISSO 51:2023 p.70: vloerverwarming reageert traag,
    /// nachtverlaging is dan niet zinvol). Default = `false`.
    #[serde(default)]
    pub all_floor_heating: bool,
}

fn default_warmup_time() -> f64 {
    2.0
}

fn default_floors() -> u32 {
    1
}

fn default_true() -> bool {
    true
}
