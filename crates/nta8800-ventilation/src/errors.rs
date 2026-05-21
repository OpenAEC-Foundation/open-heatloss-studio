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

    /// De iteratieve `p_z;ref`-drukoplosroutine (NTA 8800 §11.2.1.6) heeft
    /// binnen de harde iteratie-cap geen oplossing met de vereiste
    /// nauwkeurigheid (formule (11.14)) gevonden.
    ///
    /// In de praktijk wijst dit op een numeriek pathologisch invoergeval
    /// (bv. een opening-set zonder enige conductantie, of een massabalans
    /// die geen tekenwissel kent). De `iterations` geeft het bereikte
    /// iteratie-aantal; `residual` de laatst-berekende `|Σq_m|` in kg/h.
    #[error(
        "p_z;ref-drukoplosroutine (§11.2.1.6) niet geconvergeerd na {iterations} \
         iteraties — laatste massabalans-residu |Σq_m| = {residual} kg/h"
    )]
    PressureSolverDidNotConverge {
        /// Aantal uitgevoerde iteraties voordat de cap werd bereikt.
        iterations: u32,
        /// Laatst-berekende massabalans-residu `|Σq_m|`, in kg/h.
        residual: f64,
    },
}
