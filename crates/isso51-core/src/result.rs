//! Result types for ISSO 51 heat loss calculations.
//!
//! These types represent the output of the calculation engine.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Complete calculation result for an entire project/dwelling.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectResult {
    /// Results per room.
    pub rooms: Vec<RoomResult>,

    /// Building-level summary.
    pub summary: BuildingSummary,
}

/// Calculation result for a single room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoomResult {
    /// Room ID (matches input Room.id).
    pub room_id: String,

    /// Room name.
    pub room_name: String,

    /// Design indoor temperature θ_i in °C.
    pub theta_i: f64,

    /// Transmission heat loss breakdown.
    pub transmission: TransmissionResult,

    /// Infiltration heat loss.
    pub infiltration: InfiltrationResult,

    /// Ventilation heat loss.
    pub ventilation: VentilationResult,

    /// Heating-up allowance (opwarmtoeslag).
    pub heating_up: HeatingUpResult,

    /// System losses (floor/wall/ceiling heating).
    pub system_losses: SystemLossResult,

    /// Total heat loss for this room in W.
    /// Φ_HL,i = Φ_basis + Φ_extra
    pub total_heat_loss: f64,

    /// Basis heat loss (always occurring) in W.
    /// Φ_basis = Φ_T,exterior + Φ_T,unheated + Φ_T,ground + Φ_infiltration + Φ_system
    pub basis_heat_loss: f64,

    /// Extra heat loss (quadratic sum of non-simultaneous) in W.
    /// Φ_extra = √(Φ_vent² + Φ_T,adj² + Φ_hu²)
    pub extra_heat_loss: f64,
}

/// Breakdown of transmission heat losses for a room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransmissionResult {
    /// Specific heat loss to exterior H_T,ie in W/K.
    pub h_t_exterior: f64,

    /// Specific heat loss to adjacent rooms H_T,ia in W/K.
    pub h_t_adjacent_rooms: f64,

    /// Specific heat loss via unheated spaces H_T,io in W/K.
    pub h_t_unheated: f64,

    /// Specific heat loss to neighboring buildings H_T,ib in W/K.
    pub h_t_adjacent_buildings: f64,

    /// Specific heat loss to ground H_T,ig in W/K.
    pub h_t_ground: f64,

    /// Total transmission heat loss Φ_T in W.
    pub phi_t: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Infiltration heat loss result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfiltrationResult {
    /// Specific heat loss by infiltration H_i in W/K.
    pub h_i: f64,

    /// Infiltration fraction z_i.
    pub z_i: f64,

    /// Infiltration heat loss Φ_i in W.
    pub phi_i: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Ventilation heat loss result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VentilationResult {
    /// Specific heat loss by ventilation H_v in W/K.
    pub h_v: f64,

    /// Temperature correction factor f_v.
    pub f_v: f64,

    /// Ventilation volume flow q_v in dm³/s (effective, used in calculation).
    pub q_v: f64,

    /// BBL minimum ventilation rate in dm³/s.
    /// Always calculated from room function and floor area (BBL Afdeling 3.6).
    pub q_v_minimum: f64,

    /// Ventilation heat loss Φ_v in W.
    pub phi_v: f64,

    /// In-scope ventilation loss Φ_vent (after subtracting infiltration) in W.
    pub phi_vent: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Heating-up allowance result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeatingUpResult {
    /// Heating-up allowance Φ_hu in W.
    pub phi_hu: f64,

    /// Heating-up factor f_RH in W/m².
    pub f_rh: f64,

    /// Accumulating surface area in m².
    pub accumulating_area: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// System loss result (embedded heating).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemLossResult {
    /// Floor heating loss to ground/crawlspace Φ_verlies1 in W.
    pub phi_floor_loss: f64,

    /// Wall heating loss to exterior/adjacent Φ_verlies2 in W.
    pub phi_wall_loss: f64,

    /// Ceiling heating loss to exterior/adjacent Φ_verlies3 in W.
    pub phi_ceiling_loss: f64,

    /// Total system losses in W.
    pub phi_system_total: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Building-level summary of heat losses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BuildingSummary {
    /// Total transmission loss through building envelope in W.
    pub total_envelope_loss: f64,

    /// Total transmission loss to neighbors in W.
    pub total_neighbor_loss: f64,

    /// Total ventilation/infiltration loss in W.
    pub total_ventilation_loss: f64,

    /// Total heating-up allowance in W.
    pub total_heating_up: f64,

    /// Total system losses in W.
    pub total_system_losses: f64,

    /// Connection capacity (aansluitvermogen) of the dwelling in W.
    pub connection_capacity: f64,

    /// Contribution to collective installation in W (if applicable).
    pub collective_contribution: f64,
}
