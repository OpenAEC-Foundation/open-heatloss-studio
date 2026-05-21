//! Construction element model for ISSO 53 heat loss calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{BoundaryType, MaterialType, VerticalPosition};

/// A single construction element forming part of a room boundary.
/// ISSO 53 §4 — each element contributes to the room's heat loss.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstructionElement {
    /// Unique identifier for this element.
    pub id: String,

    /// Human-readable description (e.g., "buitenwand noord", "raam kantoor").
    pub description: String,

    /// Area of the element in m².
    pub area: f64,

    /// U-value (thermal transmittance) in W/(m²·K).
    pub u_value: f64,

    /// Type of boundary this element faces.
    pub boundary_type: BoundaryType,

    /// Material type: masonry or non-masonry.
    /// Affects thermal bridge correction ΔU_TB.
    pub material_type: MaterialType,

    /// Temperature correction factor f_k (dimensionless).
    /// Set to `None` to have it auto-calculated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_factor: Option<f64>,

    /// ID of the adjacent room (for `BoundaryType::AdjacentRoom`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacent_room_id: Option<String>,

    /// Legacy: hardcoded design temperature of the adjacent space in °C.
    /// Only used as fallback when `adjacent_room_id` cannot be resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjacent_temperature: Option<f64>,

    /// Vertical position: floor, ceiling, or wall.
    #[serde(default = "default_vertical_position")]
    pub vertical_position: VerticalPosition,

    /// Whether to use the forfaitaire thermal bridge correction.
    /// Only applies to exterior boundary elements.
    #[serde(default = "default_true")]
    pub use_forfaitaire_thermal_bridge: bool,

    /// Custom ΔU_TB value in W/(m²·K) if not using the forfaitaire method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_delta_u_tb: Option<f64>,

    /// Ground parameters, only for BoundaryType::Ground elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_params: Option<GroundParameters>,

    /// Whether this element has embedded heating behind it.
    #[serde(default)]
    pub has_embedded_heating: bool,
}

/// Parameters for ground heat loss calculation.
/// ISSO 53 formule 4.21: H_T,ig = 1.45 × Σ(A_k × U_equiv,k × f_gw × f_ig,k)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundParameters {
    /// Equivalent U-value U_equiv in W/(m²·K).
    pub u_equivalent: f64,

    /// Ground water correction factor f_gw (dimensionless).
    /// 1.0 for groundwater ≥1m below floor, 1.15 otherwise.
    #[serde(default = "default_gw")]
    pub ground_water_factor: f64,

    /// Temperature correction factor f_ig (dimensionless).
    #[serde(default = "default_fig")]
    pub f_ig: f64,
}

fn default_vertical_position() -> VerticalPosition {
    VerticalPosition::Wall
}

fn default_true() -> bool {
    true
}

fn default_gw() -> f64 {
    1.0
}

fn default_fig() -> f64 {
    1.0
}