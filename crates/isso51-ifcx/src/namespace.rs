//! ISSO 51 custom namespace definitions for IFCX.
//!
//! These attributes extend an IFCX document with warmteverliesberekening
//! input data and calculation results. Custom namespaces are explicitly
//! supported by the IFC5 spec.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Namespace keys
// ---------------------------------------------------------------------------

/// ISSO 51 namespace constants for IFCX attribute keys.
pub mod ns {
    /// Building-level input: building type, qv10, floor area, etc.
    pub const BUILDING: &str = "isso51::building";

    /// Room-level input: function, floor area, height, ventilation config.
    pub const ROOM: &str = "isso51::room";

    /// Construction properties on wall/slab/roof elements.
    pub const CONSTRUCTION: &str = "isso51::construction";

    /// Material layers on construction elements.
    pub const LAYERS: &str = "isso51::construction::layers";

    /// Design conditions (climate) on project.
    pub const CONDITIONS: &str = "isso51::conditions";

    /// Ventilation config on project/building.
    pub const VENTILATION: &str = "isso51::ventilation";

    /// Transmission result per space.
    pub const CALC_TRANSMISSION: &str = "isso51::calc::transmission";

    /// Ventilation result per space.
    pub const CALC_VENTILATION: &str = "isso51::calc::ventilation";

    /// Reheat result per space.
    pub const CALC_REHEAT: &str = "isso51::calc::reheat";

    /// Total result per space.
    pub const CALC_RESULT: &str = "isso51::calc::result";

    /// Building-level report/summary.
    pub const CALC_REPORT: &str = "isso51::report";
}

// ---------------------------------------------------------------------------
// Input attribute types (written by UI/IFC parser, read by calculator)
// ---------------------------------------------------------------------------

/// Building-level input properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Building {
    pub building_type: String,
    pub qv10: f64,
    pub total_floor_area: f64,
    pub security_class: String,
    #[serde(default)]
    pub has_night_setback: bool,
    #[serde(default)]
    pub warmup_time: f64,
    pub num_floors: Option<u32>,
    pub infiltration_method: Option<String>,
}

/// Room-level input properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Room {
    pub function: String,
    pub floor_area: f64,
    pub height: f64,
    pub custom_temperature: Option<f64>,
    pub ventilation_rate: Option<f64>,
    #[serde(default)]
    pub has_mechanical_exhaust: bool,
    #[serde(default)]
    pub has_mechanical_supply: bool,
    #[serde(default = "default_one")]
    pub fraction_outside_air: f64,
    pub heating_system: Option<String>,
}

fn default_one() -> f64 {
    1.0
}

/// Construction element properties on a wall/slab/roof entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Construction {
    pub description: String,
    pub area: f64,
    pub u_value: f64,
    pub boundary_type: String,
    pub material_type: String,
    pub vertical_position: Option<String>,
    pub temperature_factor: Option<f64>,
    pub adjacent_temperature: Option<f64>,
    pub adjacent_room_path: Option<String>,
    #[serde(default)]
    pub use_forfaitaire_thermal_bridge: bool,
    pub custom_delta_u_tb: Option<f64>,
    #[serde(default)]
    pub has_embedded_heating: bool,
}

/// Material layer in a construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51MaterialLayer {
    pub name: String,
    /// Thickness in mm.
    pub thickness: f64,
    /// Thermal conductivity in W/(m·K).
    pub lambda: f64,
    /// Thermal resistance of this layer in m²K/W.
    #[serde(rename = "R")]
    pub r_value: f64,
}

/// Design conditions (climate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Conditions {
    /// Outdoor design temperature θ_e in °C.
    pub theta_e: f64,
    /// Ground temperature θ_b for residential in °C.
    pub theta_b_residential: Option<f64>,
    /// Wind class.
    pub wind_class: Option<String>,
    /// Location description.
    pub location: Option<String>,
}

/// Ventilation system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Ventilation {
    pub system_type: String,
    #[serde(default)]
    pub has_heat_recovery: bool,
    pub heat_recovery_efficiency: Option<f64>,
}

// ---------------------------------------------------------------------------
// Output attribute types (written by calculator, read by UI/reports)
// ---------------------------------------------------------------------------

/// Per-space transmission result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51CalcTransmission {
    /// H_T in W/K.
    #[serde(rename = "H_T")]
    pub h_t: f64,
    /// Φ_T in W.
    pub phi_t: f64,
}

/// Per-space ventilation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51CalcVentilation {
    /// H_V in W/K.
    #[serde(rename = "H_V")]
    pub h_v: f64,
    /// Φ_V in W.
    pub phi_v: f64,
    /// Specific infiltration qi_spec in dm³/(s·m²).
    pub qi_spec: Option<f64>,
}

/// Per-space reheat result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51CalcReheat {
    /// Φ_RH in W.
    pub phi_rh: f64,
    /// f_RH factor in W/m².
    pub f_rh: f64,
}

/// Per-space total result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51CalcResult {
    /// Total design heat loss Φ_HL in W.
    pub phi_hl: f64,
    /// Transmission loss Φ_T in W.
    pub phi_t: f64,
    /// Ventilation loss Φ_V in W.
    pub phi_v: f64,
    /// Reheat Φ_RH in W.
    pub phi_rh: f64,
    /// Design temperature θ_i in °C.
    pub theta_int: f64,
    /// Basis heat loss in W.
    pub phi_basis: f64,
    /// Extra heat loss in W.
    pub phi_extra: f64,
}

/// Building-level report summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isso51Report {
    /// Connection capacity in W.
    pub connection_capacity: f64,
    /// Total transmission envelope loss in W.
    pub total_envelope_loss: f64,
    /// Total ventilation loss in W.
    pub total_ventilation_loss: f64,
    /// Total heating-up in W.
    pub total_heating_up: f64,
    /// Total floor area in m².
    pub total_area: f64,
    /// Specific loss in W/m².
    pub specific_loss: f64,
}
