//! Error-types voor de lighting-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie. Deze
//! enum voegt lighting-specifieke fouten toe (negatief vermogen, factor
//! buiten [0, 1], niet-eindig).

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor verlichting-berekeningen.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum LightingError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// Een lichttechnische factor viel buiten het toegestane interval `[0, 1]`
    /// of was niet-eindig.
    ///
    /// Geldt voor `F_u` (bezettingsfactor), `F_d` (daglichtcorrectie) en
    /// `F_c` (regelfactor). Waarde 0 is toegestaan (volledig gedimd /
    /// afgeschakeld); negatieve waarden zijn fysisch onmogelijk.
    #[error("factor `{name}` = {value} valt buiten toegestaan interval [0, 1] of is niet-eindig")]
    InvalidFactor {
        /// Naam van de factor (bv. `"F_u"`, `"F_d"`, `"F_c"`).
        name: &'static str,
        /// De opgegeven waarde.
        value: f64,
    },

    /// Geïnstalleerd vermogen `P_n` is negatief of niet-eindig.
    ///
    /// `P_n = 0` is toegestaan (geen verlichting geïnstalleerd → W_L;use = 0).
    #[error("P_n = {value} W/m² is negatief of niet-eindig")]
    InvalidInstalledPower {
        /// De opgegeven waarde in W/m².
        value: f64,
    },

    /// Vloeroppervlakte `A_f` is negatief of niet-eindig.
    ///
    /// `A_f = 0` is toegestaan (lege rekenzone → W_L;use = 0).
    #[error("A_f = {value} m² is negatief of niet-eindig")]
    InvalidFloorArea {
        /// De opgegeven waarde in m².
        value: f64,
    },
}

/// Result-alias voor lighting-berekeningen.
pub type LightingCalcResult<T> = Result<T, LightingError>;
