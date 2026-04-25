//! Domeintypen voor humidity systemen, targets en configuratie.
//!
//! De types in deze module beschrijven de **input** van de humidity berekening
//! (systeemkeuze, setpoints, rendementen). Rekenresultaten
//! leven in [`crate::result`].

pub mod system;
pub mod target;

pub use system::{DehumidificationSystem, HumidificationSystem, HumiditySystemConfig};
pub use target::HumidityTarget;