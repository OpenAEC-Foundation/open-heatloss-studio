//! Ground heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::ConstructionElement;

/// Calculate ground heat loss coefficient H_T,ig.
/// ISSO 53 formule 4.21, 4.24.
pub fn calculate_h_t_ground(_elements: &[ConstructionElement]) -> Result<f64> {
    // TODO: implement in batch 2 - includes complex U_equiv calculation from tabel 4.3
    unimplemented!("batch 2")
}

/// Calculate equivalent U-value for ground element.
/// ISSO 53 formule 4.24 with parameters from tabel 4.3.
pub fn calculate_u_equivalent(
    _area: f64,
    _perimeter: f64,
    _depth: f64,
    _u_construction: f64,
    _is_wall: bool,
) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}