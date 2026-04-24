//! Error-types voor de demand-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie. Deze
//! enum voegt demand-specifieke fouten toe (negatieve tijdconstante, niet
//! eindige benuttingsfactor-invoer, etc.).

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor maand-balans berekeningen.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum DemandError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// H_tr + H_ve is niet positief; tijdconstante zou oneindig of negatief zijn.
    #[error(
        "H_tr + H_ve = {total_conductance} W/K is niet > 0 — tijdconstante τ kan niet bepaald worden"
    )]
    NonPositiveConductance {
        /// Som van H_tr en H_ve in W/K (mag niet ≤ 0).
        total_conductance: f64,
    },

    /// Vloeroppervlakte is niet positief of niet-eindig.
    #[error("vloeroppervlakte A_g = {floor_area_m2} m² is niet > 0 of niet-eindig")]
    InvalidFloorArea {
        /// De opgegeven waarde.
        floor_area_m2: f64,
    },

    /// Interne warmtelast-flux is negatief of niet-eindig.
    #[error("interne warmtelast Φ_int = {flux_w_per_m2} W/m² is negatief of niet-eindig")]
    InvalidInternalHeatFlux {
        /// De opgegeven waarde in W/m².
        flux_w_per_m2: f64,
    },
}

/// Result-alias voor demand-berekeningen.
pub type DemandCalcResult<T> = Result<T, DemandError>;
