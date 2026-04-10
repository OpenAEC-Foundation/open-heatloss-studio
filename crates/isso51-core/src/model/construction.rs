//! Construction element model for ISSO 51 heat loss calculations.
//!
//! A construction element represents a single boundary surface of a room
//! (wall, floor, ceiling, window, door) with its thermal properties.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{BoundaryType, MaterialType, VerticalPosition};

/// A single construction element forming part of a room boundary.
/// ISSO 51 §2.5 — each element contributes to the room's heat loss.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionElement {
    /// Unique identifier for this element.
    pub id: String,

    /// Human-readable description (e.g., "buitenwand noord", "raam woonkamer").
    pub description: String,

    /// Area of the element in m².
    pub area: f64,

    /// U-value (thermal transmittance) in W/(m²·K).
    pub u_value: f64,

    /// Type of boundary this element faces.
    pub boundary_type: BoundaryType,

    /// Material type: masonry or non-masonry.
    /// Affects thermal bridge correction in the forfaitaire method.
    pub material_type: MaterialType,

    /// Temperature correction factor f_k (dimensionless).
    /// For exterior elements: typically 1.0.
    /// For unheated spaces: from ISSO 51 Table 4.1.
    /// For adjacent rooms: calculated from temperature difference.
    /// For neighboring dwellings: calculated with Δθ corrections.
    /// Set to `None` to have it auto-calculated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_factor: Option<f64>,

    /// ID of the adjacent room (for `BoundaryType::AdjacentRoom`).
    ///
    /// The transmission calculation resolves the adjacent room's design
    /// temperature at runtime by looking the room up in the `Project` room
    /// list. This is the canonical source — `adjacent_temperature` below is
    /// a legacy fallback only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacent_room_id: Option<String>,

    /// Legacy: hardcoded design temperature of the adjacent space in °C.
    ///
    /// **Deprecated** — the calculation now derives the adjacent-room
    /// temperature from `adjacent_room_id` via a live lookup in
    /// `Project.rooms`. This field is retained for backward compatibility
    /// with older saved projects and is only consulted as a fallback when
    /// `adjacent_room_id` cannot be resolved. New code should leave this
    /// field at `None` and rely on the live room lookup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjacent_temperature: Option<f64>,

    /// Vertical position: floor, ceiling, or wall.
    /// Used for fb calculation to neighboring dwellings.
    #[serde(default = "default_vertical_position")]
    pub vertical_position: VerticalPosition,

    /// Whether to use the forfaitaire thermal bridge correction (ΔU_TB = 0.1).
    /// Only applies to exterior boundary elements (BoundaryType::Exterior).
    #[serde(default = "default_true")]
    pub use_forfaitaire_thermal_bridge: bool,

    /// Custom ΔU_TB value in W/(m²·K) if not using the forfaitaire method.
    /// Overrides the default 0.1 W/(m²·K) correction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_delta_u_tb: Option<f64>,

    /// Ground parameters, only for BoundaryType::Ground elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_params: Option<GroundParameters>,

    /// Whether this element has floor/wall/ceiling heating behind it.
    /// Relevant for system loss calculations (§2.9).
    #[serde(default)]
    pub has_embedded_heating: bool,

    /// Reference to a `CatalogEntry.id` produced by the thermal import.
    /// `None` for openings, manually-added elements and projects from
    /// before the catalog refactor (legacy projects keep working).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_ref: Option<String>,
}

/// Parameters for ground heat loss calculation.
/// ISSO 51 §2.5.5, formule (4.18): H_T,ig = 1.45 × G_w × Σ(A_k × f_g2 × U_e,k)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GroundParameters {
    /// Equivalent U-value for the ground element U_e,k in W/(m²·K).
    pub u_equivalent: f64,

    /// Ground water correction factor G_w (dimensionless).
    /// Typically 1.0 for normal conditions, higher for high water table.
    #[serde(default = "default_gw")]
    pub ground_water_factor: f64,

    /// Temperature correction factor f_g2 (dimensionless).
    /// Accounts for seasonal ground temperature variation.
    #[serde(default = "default_fg2")]
    pub fg2: f64,
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

fn default_fg2() -> f64 {
    1.0
}

/// A library entry for a reusable construction type.
/// Can be referenced by multiple ConstructionElements.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionType {
    /// Unique identifier.
    pub id: String,

    /// Human-readable name (e.g., "Spouwmuur Rc=4.5").
    pub name: String,

    /// U-value in W/(m²·K).
    pub u_value: f64,

    /// Material type.
    pub material_type: MaterialType,

    /// Layers making up this construction (optional detail).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<ConstructionLayer>,
}

/// A single layer in a construction assembly.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionLayer {
    /// Material name.
    pub material: String,

    /// Thickness in mm.
    pub thickness: f64,

    /// Thermal conductivity λ in W/(m·K).
    pub lambda: f64,
}

impl ConstructionLayer {
    /// Calculate the thermal resistance R of this layer in m²·K/W.
    pub fn thermal_resistance(&self) -> f64 {
        (self.thickness / 1000.0) / self.lambda
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_thermal_resistance() {
        let layer = ConstructionLayer {
            material: "insulation".to_string(),
            thickness: 100.0,
            lambda: 0.035,
        };
        let r = layer.thermal_resistance();
        assert!((r - 2.857).abs() < 0.01);
    }
}
