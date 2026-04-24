//! Error-types voor de dhw-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie. Deze
//! enum voegt dhw-specifieke fouten toe (negatieve SCOP_W, efficiency buiten
//! interval [0, 1], non-positive floor area, etc.).

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor warmtapwater-berekeningen.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum DhwError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// Een rendement viel buiten het toegestane interval (0, upper] of was niet-eindig.
    ///
    /// Voor η_W;em, η_W;dis en η_W;gen-achtige factoren moet gelden 0 < η ≤ 1.
    /// Warmtepomp SCOP_W mag > 1 zijn.
    #[error(
        "rendement `{name}` = {value} valt buiten toegestaan interval (0, {upper}] of is niet-eindig"
    )]
    InvalidEfficiency {
        /// Naam van het rendement (bv. "η_W;em", "η_W;dis", "η_W;gen").
        name: &'static str,
        /// De opgegeven waarde.
        value: f64,
        /// Bovengrens van het interval (inclusief).
        upper: f64,
    },

    /// SCOP_W voor een tapwater-warmtepomp is niet positief of niet-eindig.
    #[error("SCOP_W = {scop} voor tapwater-warmtepomp is niet > 0 of niet-eindig")]
    InvalidScop {
        /// De opgegeven SCOP_W-waarde.
        scop: f64,
    },

    /// Stadsverwarming-factor is niet positief of niet-eindig of > 1.
    #[error("stadsverwarming-factor = {factor} is niet > 0 of niet-eindig of > 1")]
    InvalidDistrictHeatingFactor {
        /// De opgegeven factor.
        factor: f64,
    },

    /// Gebruiksoppervlakte A_g is niet positief of niet-eindig.
    #[error("gebruiksoppervlakte A_g = {area} m² is niet > 0 of niet-eindig")]
    InvalidFloorArea {
        /// De opgegeven A_g waarde.
        area: f64,
    },
}

/// Result-alias voor dhw-berekeningen.
pub type DhwCalcResult<T> = Result<T, DhwError>;
