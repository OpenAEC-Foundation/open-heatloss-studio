//! Foutsoorten voor `nta8800-automation`.

use thiserror::Error;

/// Fouten die kunnen optreden bij gebouwautomatisering-berekeningen.
#[derive(Debug, Error, PartialEq)]
pub enum AutomationError {
    /// Ongeldige BAC-klasse voor een specifieke dienst.
    #[error("BAC-klasse {class:?} is ongeldig voor dienst {service}")]
    InvalidBacClass {
        /// De BAC-klasse die niet geldig is.
        class: String,
        /// De energiedienst waarvoor de klasse niet geldig is.
        service: String,
    },

    /// Gebruiksfunctie niet ondersteund voor automatisering.
    #[error("Gebruiksfunctie {function:?} heeft geen gedefinieerde BAC-factoren")]
    UnsupportedUsageFunction {
        /// De gebruiksfunctie waarvoor geen factoren zijn gedefinieerd.
        function: String,
    },

    /// Correctiefactor buiten fysisch realistische range.
    #[error("Correctiefactor f_BAC = {factor} voor {service} ligt buiten bereik [0.5, 2.0]")]
    UnrealisticCorrectionFactor {
        /// De berekende factor.
        factor: f64,
        /// De energiedienst.
        service: String,
    },

    /// Ontbrekende configuratie voor een vereiste energiedienst.
    #[error("Ontbrekende BAC-configuratie voor energiedienst {service}")]
    MissingServiceConfiguration {
        /// De energiedienst waarvoor geen configuratie is opgegeven.
        service: String,
    },
}