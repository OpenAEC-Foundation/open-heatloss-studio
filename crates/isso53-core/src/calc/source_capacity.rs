//! Source (boiler/heat-pump) connection capacity for ISSO 53 (hoofdstuk 5).
//!
//! Two variants:
//! - Individual installation: formule 5.1
//! - Collective installation: formule 5.9 (excludes Φ_T,iaBE)
//!
//! Both apply the infiltration reduction fraction z from tabel 5.1
//! to prevent over-dimensioning (wind never hits all facades simultaneously).

use crate::error::Result;
use crate::result::RoomResult;

/// Calculate the individual connection capacity Φ_source.
/// ISSO 53 formule 5.1.
pub fn calculate_individual(_rooms: &[RoomResult], _z: f64) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}

/// Calculate the collective contribution Φ_source.
/// ISSO 53 formule 5.9.
pub fn calculate_collective(_rooms: &[RoomResult], _z: f64) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}
