//! Per-room heat loss orchestration for ISSO 53 (§4.1).
//!
//! Combines transmission, ventilation, infiltration, heating-up and gain
//! per room into the total Φ_HL,i (formule 4.1).

use crate::error::Result;
use crate::model::{Building, DesignConditions, Room, VentilationConfig};
use crate::result::RoomResult;

/// Calculate the complete heat loss result for a single room.
/// ISSO 53 formule 4.1, 4.2.
pub fn calculate_room(
    _room: &Room,
    _all_rooms: &[Room],
    _building: &Building,
    _climate: &DesignConditions,
    _ventilation: &VentilationConfig,
) -> Result<RoomResult> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}
