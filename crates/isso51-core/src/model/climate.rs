//! Climate and design condition parameters for ISSO 51.
//!
//! Defines the outdoor design conditions and related parameters.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Design conditions for the heat loss calculation.
/// ISSO 51 §2.7.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DesignConditions {
    /// Design outdoor temperature θ_e in °C.
    /// Standard for the Netherlands: -10°C.
    #[serde(default = "default_theta_e")]
    pub theta_e: f64,

    /// Temperature of adjacent buildings with residential function θ_b in °C.
    /// Erratum 2023: 17°C for residential, 14°C for non-residential.
    #[serde(default = "default_theta_b_residential")]
    pub theta_b_residential: f64,

    /// Temperature of adjacent buildings with non-residential function in °C.
    #[serde(default = "default_theta_b_non_residential")]
    pub theta_b_non_residential: f64,

    /// Wind exposure factor (for infiltration correction).
    /// Typically 1.0 for normal exposure.
    #[serde(default = "default_one")]
    pub wind_factor: f64,

    /// Design temperature of open water θ_water in °C.
    ///
    /// Used for `BoundaryType::Water` (woonboot use case). Default 5 °C.
    /// This is an engineering choice, not a norm value — overridable per
    /// project. Reports must include a footnote when any construction uses
    /// the Water boundary type.
    #[serde(default = "default_theta_water")]
    pub theta_water: f64,

    /// Design temperature of the ground θ_ground in °C.
    ///
    /// Used as a fallback reference for ground-contact constructions when
    /// no explicit ground parameters (`ground_params`) are available.
    /// Ground heat loss is normally computed via ISSO 51 §2.5.5
    /// (`u_equivalent`, `f_g2`, `G_w`); this field only drives the
    /// simplified ΔT display in the frontend and reports. Default 10 °C.
    #[serde(default = "default_theta_ground")]
    pub theta_ground: f64,
}

impl Default for DesignConditions {
    fn default() -> Self {
        Self {
            theta_e: -10.0,
            theta_b_residential: 17.0,
            theta_b_non_residential: 14.0,
            wind_factor: 1.0,
            theta_water: 5.0,
            theta_ground: 10.0,
        }
    }
}

fn default_theta_e() -> f64 {
    -10.0
}

fn default_theta_b_residential() -> f64 {
    17.0
}

fn default_theta_b_non_residential() -> f64 {
    14.0
}

fn default_one() -> f64 {
    1.0
}

fn default_theta_water() -> f64 {
    5.0
}

fn default_theta_ground() -> f64 {
    10.0
}
