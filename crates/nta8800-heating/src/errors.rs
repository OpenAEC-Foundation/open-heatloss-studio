//! Error-types voor de heating-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie. Deze
//! enum voegt heating-specifieke fouten toe (negatieve SCOP, efficiency buiten
//! interval [0, 1], etc.).

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor verwarming-berekeningen.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum HeatingError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// Een rendement viel buiten het toegestane interval (0, 1] of was niet-eindig.
    ///
    /// Afgifte-, distributie- en opwekkingsrendementen moeten strikt > 0 zijn
    /// om deling door nul te vermijden, en ≤ 1 behalve bij warmtepomp (SCOP).
    #[error(
        "rendement `{name}` = {value} valt buiten toegestaan interval (0, {upper}] of is niet-eindig"
    )]
    InvalidEfficiency {
        /// Naam van het rendement (bv. "η_em", "η_dist", "f_reg").
        name: &'static str,
        /// De opgegeven waarde.
        value: f64,
        /// Bovengrens van het interval (inclusief).
        upper: f64,
    },

    /// SCOP / COP voor een warmtepomp is niet positief of niet-eindig.
    #[error("SCOP = {scop} voor warmtepomp is niet > 0 of niet-eindig")]
    InvalidScop {
        /// De opgegeven SCOP-waarde.
        scop: f64,
    },

    /// Stadsverwarming-factor is niet positief of niet-eindig.
    #[error("stadsverwarming-factor = {factor} is niet > 0 of niet-eindig")]
    InvalidDistrictHeatingFactor {
        /// De opgegeven factor.
        factor: f64,
    },
}

/// Result-alias voor heating-berekeningen.
pub type HeatingCalcResult<T> = Result<T, HeatingError>;
