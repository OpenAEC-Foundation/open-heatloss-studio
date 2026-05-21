//! Infiltration heat loss for ISSO 53 (§4.7.1).
//!
//! Two methods depending on whether `q_v10,kar` is known:
//! - Known: lookup in tabel 4.5
//! - Unknown: formule 4.31 with f_wind, f_type, f_inf, f_jaar

use crate::error::Result;
use crate::model::{Building, Room};

/// Calculate the specific infiltration heat loss H_i for a room.
/// ISSO 53 formule 4.27.
pub fn calculate_h_i(_room: &Room, _building: &Building) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate the infiltration heat loss Φ_i for a room.
/// ISSO 53 formule 4.25.
pub fn calculate_phi_i(_room: &Room, _building: &Building, _theta_e: f64) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}
