//! Input-types voor de demand-crate.
//!
//! Deze module groepeert de invoer-structs die de maand-balans orkestreert:
//! interne warmtelast (`InternalGains`), setpoint-profielen
//! (`HeatingSetpoint` / `CoolingSetpoint`) en thermische-massa classificatie
//! (`ThermalMassInput`).
//!
//! Rekenresultaten leven in [`crate::result`].

pub mod internal_load;
pub mod setpoints;
pub mod thermal_mass;

pub use internal_load::InternalGains;
pub use setpoints::{CoolingSetpoint, HeatingSetpoint};
pub use thermal_mass::ThermalMassInput;
