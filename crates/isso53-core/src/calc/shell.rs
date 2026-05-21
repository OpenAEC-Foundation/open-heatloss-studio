//! Shell method for ISSO 53 (hoofdstuk 3 — voorontwerp).
//!
//! Building treated as one large room. Fast estimate for connection
//! capacity during preliminary design / feasibility study.

use crate::error::Result;
use crate::model::Project;

/// Calculate building heat loss via the shell method.
/// ISSO 53 formule 3.1.
pub fn calculate_shell(_project: &Project) -> Result<f64> {
    // TODO: implement in batch 2
    unimplemented!("batch 2")
}
