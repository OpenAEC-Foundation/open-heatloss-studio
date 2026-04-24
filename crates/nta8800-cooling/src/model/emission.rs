//! Koudeafgiftesysteem — rendement η_em voor koude.
//!
//! Radiatoren worden zelden als koudeafgifte gebruikt; typische varianten
//! zijn vloerkoeling, plafondkoeling, wandconvectoren en split-unit
//! verdampers. V1 modelleert alle varianten met één enkel η_em-getal;
//! type-specifieke temperatuurcorrecties en stralings-/convectie-balans
//! volgen in V2.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Koudeafgifte-rendement η_em voor koude.
///
/// Typische waarden (forfaitaire defaults vanuit NTA 8800 afgifte-tabellen):
/// - Vloerkoeling / betonkernactivering: 0,88–0,92
/// - Luchtkoeling (split-unit): 0,92–0,95
/// - Wandconvectoren: 0,85–0,90
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingEmission {
    /// η_em;C — afgifte-rendement voor koude, dimensieloos in (0, 1].
    pub efficiency: f64,
    /// f_reg — dimensieloze regelfactor voor koelcircuit (temperatuur- en
    /// tijdsregeling). Typisch 0,95–1,00 voor goede PID-regeling met
    /// dagschema. Vervangt de klassieke H.10 regel-factor η_reg.
    pub regulation_factor: f64,
}

impl CoolingEmission {
    /// Forfaitair default η_em = 0,92 en f_reg = 0,97.
    #[must_use]
    pub const fn default_values() -> Self {
        Self {
            efficiency: 0.92,
            regulation_factor: 0.97,
        }
    }
}

impl Default for CoolingEmission {
    fn default() -> Self {
        Self::default_values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_sensible() {
        let e = CoolingEmission::default();
        assert!((e.efficiency - 0.92).abs() < 1e-9);
        assert!((e.regulation_factor - 0.97).abs() < 1e-9);
    }

    #[test]
    fn emission_serde_round_trip() {
        let e = CoolingEmission {
            efficiency: 0.90,
            regulation_factor: 0.95,
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: CoolingEmission = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }
}
