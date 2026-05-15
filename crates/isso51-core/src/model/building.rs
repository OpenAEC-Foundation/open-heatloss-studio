//! Building/project model for ISSO 51 heat loss calculations.
//!
//! The project is the top-level container holding all information needed
//! for a complete heat loss calculation of a dwelling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::climate::DesignConditions;
use super::enums::{
    AggregationMethod, BuildingType, ConstructionVariant, DwellingClass, InfiltrationMethod,
    SecurityClass,
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
}

fn default_warmup_time() -> f64 {
    2.0
}

fn default_floors() -> u32 {
    1
}
