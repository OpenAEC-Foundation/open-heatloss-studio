//! Transmission heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::{ConstructionElement, DesignConditions, Room};

/// Calculate transmission heat losses for a room.
/// ISSO 53 formules 4.2, 4.3, 4.13, 4.17.
pub fn calculate_transmission(
    _room: &Room,
    _all_rooms: &[Room],
    _climate: &DesignConditions,
) -> Result<TransmissionResult> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate H_T,ie to exterior air.
/// ISSO 53 formule 4.3.
pub fn calculate_h_t_exterior(_elements: &[ConstructionElement]) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate H_T,iae to unheated spaces.
/// ISSO 53 formule 4.13.
pub fn calculate_h_t_unheated(_elements: &[ConstructionElement]) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate H_T,iaBE to adjacent buildings.
/// ISSO 53 formule 4.17.
pub fn calculate_h_t_adjacent_buildings(_elements: &[ConstructionElement]) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Results from transmission calculation.
pub struct TransmissionResult {
    pub phi_t: f64,
    pub h_t_exterior: f64,
    pub h_t_adjacent_rooms: f64,
    pub h_t_unheated: f64,
    pub h_t_adjacent_buildings: f64,
    pub h_t_ground: f64,
}