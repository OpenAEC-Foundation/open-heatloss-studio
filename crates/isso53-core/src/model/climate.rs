//! Climate conditions for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Design climate conditions for heat loss calculation.
/// ISSO 53 uses different defaults than ISSO 51.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DesignConditions {
    /// Buitentemperatuur θ_e in °C.
    /// ISSO 53 default: -10°C.
    #[serde(default = "default_theta_e")]
    pub theta_e: f64,

    /// Gemiddelde buitentemperatuur θ_me in °C.
    /// ISSO 53 default: 9°C.
    #[serde(default = "default_theta_me")]
    pub theta_me: f64,

    /// Ontwerptemperatuur aangrenzend gebouw θ_b in °C.
    /// 15°C voor kantoren/winkels, 5°C vorstvrij, θ_e voor stallingen.
    #[serde(default = "default_theta_b")]
    pub theta_b_adjacent_building: f64,
}

impl Default for DesignConditions {
    fn default() -> Self {
        Self {
            theta_e: default_theta_e(),
            theta_me: default_theta_me(),
            theta_b_adjacent_building: default_theta_b(),
        }
    }
}

fn default_theta_e() -> f64 {
    -10.0
}

fn default_theta_me() -> f64 {
    9.0
}

fn default_theta_b() -> f64 {
    15.0
}