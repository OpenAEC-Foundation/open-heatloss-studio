//! Domain model for ISSO 53 heat loss calculations.
//!
//! This module contains all data types representing the input and
//! configuration for a warmteverliesberekening (heat loss calculation)
//! according to ISSO publication 53.

pub mod building;
pub mod climate;
pub mod construction;
pub mod enums;
pub mod project;
pub mod room;
pub mod ventilation;

// Re-export key types for convenience
pub use building::Building;
pub use climate::DesignConditions;
pub use construction::ConstructionElement;
pub use enums::*;
pub use project::{Project, ProjectInfo};
pub use room::{Bezetting, Room};
pub use ventilation::VentilationConfig;