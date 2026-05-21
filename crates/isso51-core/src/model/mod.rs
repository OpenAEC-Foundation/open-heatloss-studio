//! Domain model for ISSO 51 heat loss calculations.
//!
//! This module contains all data types representing the input and
//! configuration for a warmteverliesberekening (heat loss calculation)
//! according to ISSO publication 51.

pub mod building;
pub mod climate;
pub mod construction;
pub mod enums;
pub mod room;
pub mod ventilation;

// Re-export key types for convenience
pub use building::{Building, Project, ProjectInfo};
pub use climate::DesignConditions;
pub use construction::{
    ConstructionElement, ConstructionLayer, ConstructionType, GroundParameters, Spacer, UwBreakdown,
};
pub use enums::*;
pub use room::Room;
pub use ventilation::VentilationConfig;
