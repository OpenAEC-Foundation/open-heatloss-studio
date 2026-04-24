//! Error-types voor de cooling-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie en
//! voegt cooling-specifieke fouten toe.

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor koelings-berekeningen (H.10 + bijlage AA).
#[derive(Debug, Error, PartialEq, Clone)]
pub enum CoolingError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// SCOP / COP moet positief zijn (anders deelt men door 0 of negatief).
    #[error("COP / SCOP {value} is niet > 0 — ongeldige koel-efficiëntie")]
    NonPositiveCop {
        /// De ongeldige waarde.
        value: f64,
    },

    /// Factor voor vrije koeling moet in bereik 0..=1 liggen.
    #[error(
        "FreeCooling factor {value} valt buiten 0..=1 — factor is dimensieloos benuttingsfractie"
    )]
    InvalidFreeCoolingFactor {
        /// De opgegeven factor.
        value: f64,
    },

    /// Rendement (η_em, η_dist, f_reg) moet in bereik (0, 1] liggen.
    #[error(
        "rendement {name} = {value} valt buiten (0, 1] — fysisch onmogelijk voor \
         distributie/afgifte/regeling"
    )]
    InvalidEfficiency {
        /// Korte naam van het betreffende rendement (`"η_em"`, `"η_dist"`, `"f_reg"`).
        name: &'static str,
        /// De ongeldige waarde.
        value: f64,
    },

    /// Vloeroppervlakte is niet positief of niet-eindig.
    #[error("vloeroppervlakte A = {area_m2} m² is niet > 0 of niet-eindig")]
    InvalidFloorArea {
        /// De opgegeven waarde.
        area_m2: f64,
    },

    /// Aantal bewoners per woonfunctie moet positief zijn.
    #[error("aantal bewoners per woonfunctie P_p;woon = {persons} is niet > 0")]
    InvalidPersonCount {
        /// De opgegeven waarde.
        persons: f64,
    },
}

/// Result-alias voor koel-berekeningen.
pub type CoolingCalcResult<T> = Result<T, CoolingError>;
