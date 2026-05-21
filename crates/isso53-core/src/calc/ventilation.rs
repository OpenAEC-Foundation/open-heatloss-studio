//! Ventilation heat loss for ISSO 53 (§4.7.2).

use crate::error::Result;
use crate::model::{Room, VentilationConfig};

/// Calculate the specific ventilation heat loss H_v for a room.
/// ISSO 53 formule 4.37.
pub fn calculate_h_v(_room: &Room, _ventilation: &VentilationConfig) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate the ventilation heat loss Φ_vent for a room.
/// ISSO 53 formule 4.35.
pub fn calculate_phi_vent(
    _room: &Room,
    _ventilation: &VentilationConfig,
    _theta_e: f64,
) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}
