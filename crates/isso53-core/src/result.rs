//! Result types for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Complete calculation results for an entire project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResult {
    /// Results for each room.
    pub rooms: Vec<RoomResult>,

    /// Building-level summary.
    pub summary: BuildingSummary,
}

/// Heat loss calculation results for a single room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoomResult {
    /// Room identifier.
    pub room_id: String,

    /// Room name.
    pub room_name: String,

    /// Design indoor temperature θ_i in °C.
    pub theta_i: f64,

    /// Transmission heat loss Φ_T in W.
    pub phi_t: f64,

    /// Ventilation heat loss Φ_v in W.
    pub phi_v: f64,

    /// Infiltration heat loss Φ_i in W.
    pub phi_i: f64,

    /// Heating-up supplement Φ_hu in W.
    pub phi_hu: f64,

    /// System losses Φ_system in W.
    pub phi_system: f64,

    /// Internal heat gains Φ_gain in W (negative = heat source).
    pub phi_gain: f64,

    /// Total heat loss Φ_HL,i in W.
    pub total_heat_loss: f64,

    /// Transmission coefficient H_T,ie to exterior in W/K.
    pub h_t_exterior: f64,

    /// Transmission coefficient H_T,ia to adjacent rooms in W/K.
    pub h_t_adjacent_rooms: f64,

    /// Transmission coefficient H_T,iae to unheated spaces in W/K.
    pub h_t_unheated: f64,

    /// Transmission coefficient H_T,iaBE to adjacent buildings in W/K.
    pub h_t_adjacent_buildings: f64,

    /// Transmission coefficient H_T,ig to ground in W/K.
    pub h_t_ground: f64,

    /// Ventilation coefficient H_v in W/K.
    pub h_v: f64,

    /// Infiltration coefficient H_i in W/K.
    pub h_i: f64,
}

/// Building-level summary results.
/// ISSO 53 uses simple addition, not quadratic summation like ISSO 51.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingSummary {
    /// Total transmission heat loss Φ_T,build in W.
    pub total_transmission_loss: f64,

    /// Total ventilation heat loss Φ_V,build in W.
    pub total_ventilation_loss: f64,

    /// Total infiltration heat loss Φ_I,build in W.
    pub total_infiltration_loss: f64,

    /// Total heating-up supplement Φ_hu,build in W.
    pub total_heating_up: f64,

    /// Total system losses Φ_system,build in W.
    pub total_system_losses: f64,

    /// Total internal gains Φ_gain,build in W.
    pub total_internal_gains: f64,

    /// Total building heat loss Φ_HL,build in W.
    pub total_building_heat_loss: f64,

    /// Connection capacity individual method Φ_source in W (formule 5.1).
    pub connection_capacity_individual: f64,

    /// Connection capacity collective method Φ_source in W (formule 5.9).
    pub connection_capacity_collective: f64,

    /// Infiltration reduction factor z applied at building level (tabel 5.1).
    pub infiltration_reduction_factor_z: f64,
}