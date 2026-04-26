//! Domeintypen voor PV-systemen, locatie en bronregeneratie.
//!
//! De types in deze module beschrijven de **input** van de PV-berekening
//! (systeem-specificatie, geografische locatie, optionele bronregeneratie).
//! Rekenresultaten leven in [`crate::result`].

pub mod bronregeneratie;
pub mod pv_location;
pub mod pv_system;

pub use bronregeneratie::BronregeneratieConfig;
pub use pv_location::PvLocation;
pub use pv_system::PvSystem;
