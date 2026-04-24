//! Foutsoorten voor `nta8800-ventilation`.

use thiserror::Error;

/// Fouten die kunnen optreden bij ventilatie-berekeningen.
#[derive(Debug, Error, PartialEq)]
pub enum VentilationError {
    /// Ongeldige WTW-efficiëntie (buiten `[0,1]`).
    #[error("WTW-efficiëntie η_hr = {0} ligt buiten geldig bereik [0,1]")]
    InvalidWtwEfficiency(f64),

    /// Ongeldig specifiek ventilator-vermogen SFP (< 0).
    #[error("Specifiek ventilator-vermogen f_SFP = {0} W/(m³/h) is negatief")]
    InvalidFanSfp(f64),

    /// Negatieve luchtstroom — fysisch onmogelijk voor een volumestroom.
    #[error("Luchtstroom {name} = {value} m³/h is negatief")]
    NegativeAirFlow {
        /// Naam van het debiet-type (bv. `"mechanical_supply"`).
        name: &'static str,
        /// De aangeleverde waarde.
        value: f64,
    },

    /// WTW opgegeven voor een systeem dat geen mechanische balansventilatie heeft.
    ///
    /// WTW vereist zowel mechanische toevoer áls afvoer; alleen systeem D
    /// kwalificeert (NTA 8800 §11.1 / bijlage S).
    #[error("WTW-specificatie opgegeven voor ventilatiesysteem zonder balansventilatie")]
    WtwWithoutBalancedSystem,
}
