//! Heating-up supplement for ISSO 53 (§4.8).
//!
//! ISSO 53 uses a specific supplement P [W/m²] based on warm-up time and
//! building thermal mass, differing from the ISSO 51 main-room percentage method.

use crate::error::Result;
use crate::model::Room;

/// Calculate the heating-up supplement Φ_hu for a room.
/// ISSO 53 §4.8.
pub fn calculate_heating_up(_room: &Room) -> Result<f64> {
    // TODO: implement in batch 2 — needs P-table from PDF p51-53
    unimplemented!("batch 2")
}
