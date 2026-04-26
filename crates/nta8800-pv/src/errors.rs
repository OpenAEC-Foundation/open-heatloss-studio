//! Foutsoorten voor `nta8800-pv`.

use thiserror::Error;

/// Fouten die kunnen optreden bij PV-berekeningen.
#[derive(Debug, Error, PartialEq)]
pub enum PvError {
    /// Ongeldig PV-piek-vermogen (≤ 0).
    #[error("PV-piek-vermogen {0} kWp is ongeldig (moet > 0)")]
    InvalidPeakPower(f64),

    /// Ongeldige hellingshoek (buiten [0°, 90°]).
    #[error("Hellingshoek β = {0}° ligt buiten geldig bereik [0°, 90°]")]
    InvalidTilt(f64),

    /// Ongeldige azimuth-hoek (buiten [-180°, +180°]).
    #[error("Azimuth γ = {0}° ligt buiten geldig bereik [-180°, +180°]")]
    InvalidAzimuth(f64),

    /// Ongeldige systeem-efficiëntie (buiten (0, 1]).
    #[error("Systeem-efficiëntie η_sys = {0} ligt buiten geldig bereik (0, 1]")]
    InvalidSystemEfficiency(f64),

    /// Ongeldige inverter-efficiëntie (buiten (0, 1]).
    #[error("Inverter-efficiëntie η_inv = {0} ligt buiten geldig bereik (0, 1]")]
    InvalidInverterEfficiency(f64),

    /// Ongeldige latitude (buiten [-90°, +90°]).
    #[error("Latitude {0}° ligt buiten geldig bereik [-90°, +90°]")]
    InvalidLatitude(f64),

    /// Ongeldige longitude (buiten [-180°, +180°]).
    #[error("Longitude {0}° ligt buiten geldig bereik [-180°, +180°]")]
    InvalidLongitude(f64),

    /// Negatieve zoninstraling — fysisch onmogelijk.
    #[error("Zoninstraling {0} W/m² is negatief")]
    NegativeSolarIrradiation(f64),

    /// Lege PV-systeem lijst — geen berekening mogelijk.
    #[error("Geen PV-systemen opgegeven voor berekening")]
    EmptySystemList,

    /// Klimaatdata ontbreekt vereiste maanden.
    #[error("Klimaatdata mist gegevens voor maand {month} (verwacht 1-12)")]
    MissingClimateData {
        /// De maand waarvoor data ontbreekt (1-12).
        month: u8,
    },

    /// Bronregeneratie-configuratie is onvolledig (V2-feature).
    #[error("Bronregeneratie-configuratie onvolledig: {details}")]
    IncompleteBronregeneratieConfig {
        /// Details van de ontbrekende configuratie.
        details: String,
    },
}
