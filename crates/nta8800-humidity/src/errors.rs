//! Foutsoorten voor `nta8800-humidity`.

use thiserror::Error;

/// Fouten die kunnen optreden bij humidity-berekeningen.
#[derive(Debug, Error, PartialEq)]
pub enum HumidityError {
    /// Ongeldig humidification systeem rendement (buiten `[0,1]`).
    #[error("Stoomgenerator rendement η = {0} ligt buiten geldig bereik [0,1]")]
    InvalidSteamEfficiency(f64),

    /// Ongeldig dehumidification systeem COP (≤ 0).
    #[error("Dehumidification COP = {0} moet groter zijn dan 0")]
    InvalidDehumidificationCop(f64),

    /// Ongeldige vochtigheidssetpoint waarden (min > max).
    #[error("Vochtigheid min ({min} g/kg) kan niet hoger zijn dan max ({max} g/kg)")]
    InvalidHumidityRange {
        /// Minimum vochtigheidswaarde in g/kg
        min: f64,
        /// Maximum vochtigheidswaarde in g/kg
        max: f64
    },

    /// Negatieve vochtigheidswaarde — fysisch onmogelijk.
    #[error("Vochtigheidswaarde {value} g/kg is negatief")]
    NegativeHumidity {
        /// Negatieve vochtigheidswaarde in g/kg
        value: f64
    },

    /// Onrealistische vochtigheidswaarde (> 30 g/kg).
    #[error("Vochtigheidswaarde {value} g/kg overschrijdt realistisch maximum 30 g/kg")]
    UnrealisticHumidity {
        /// Onrealistische vochtigheidswaarde in g/kg
        value: f64
    },

    /// Ongeldige luchttemperatuur voor verzadigingsdampdruk berekening.
    #[error("Temperatuur {temp} °C ligt buiten geldig bereik [-40, 60] voor dampdruk berekening")]
    InvalidTemperatureRange {
        /// Temperatuur in °C die buiten geldig bereik valt
        temp: f64
    },

    /// Negatief luchtvolume.
    #[error("Zone volume {volume} m³ moet positief zijn")]
    InvalidZoneVolume {
        /// Zone volume in m³ dat negatief is
        volume: f64
    },
}