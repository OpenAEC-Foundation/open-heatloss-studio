//! Heating-up configuration model for ISSO 53.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for heating-up supplement calculation (§4.8).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeatingUpConfig {
    /// Whether setback/heating-up supplement is active.
    pub setback_active: bool,

    /// Specific heating-up supplement in W/m².
    /// Value from ISSO 53 PDF p.51-53 P-table based on thermal mass and warm-up time.
    /// Default 0.0 = no supplement.
    pub p_w_per_m2: f64,

    /// Warm-up time in minutes (informational only, used for P-table lookup).
    /// Default 120 minutes.
    pub warmup_minutes: f64,
}

impl Default for HeatingUpConfig {
    fn default() -> Self {
        Self {
            setback_active: false,
            p_w_per_m2: 0.0,
            warmup_minutes: 120.0,
        }
    }
}