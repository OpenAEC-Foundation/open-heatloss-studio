//! Ventilation configuration for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::VentilationSystemType;

/// Ventilation system configuration.
/// ISSO 53 supports systems A/B/C/D/E (E is new for ISSO 53).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VentilationConfig {
    /// Ventilatiesysteemtype (A/B/C/D/E).
    pub system_type: VentilationSystemType,

    /// Whether the system has heat recovery (WTW).
    #[serde(default)]
    pub has_heat_recovery: bool,

    /// Heat recovery efficiency (0.0-1.0).
    /// Only relevant if `has_heat_recovery` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_recovery_efficiency: Option<f64>,

    /// Frost protection setting.
    /// Reduces WTW efficiency when outdoor temperature is very low.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_protection: Option<f64>,

    /// Supply air temperature θ_t in °C.
    /// For heated supply air systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_temperature: Option<f64>,

    /// Whether the system has preheating.
    #[serde(default)]
    pub has_preheating: bool,

    /// Preheating temperature in °C.
    /// Only relevant if `has_preheating` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheating_temperature: Option<f64>,
}