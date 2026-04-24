//! Distributie-systeem met lineair rendement η_dist.
//!
//! NTA 8800 §9.4 geeft een gedetailleerde methode voor distributie-verliezen
//! die afhankelijk is van leiding-lengte, -diameter, isolatie-dikte en
//! -conductiviteit, plus de temperatuur van de ruimte waar de leiding loopt.
//! V1 vereenvoudigt dit tot één constant rendement.
//!
//! Defaults:
//!
//! | Situatie | η_dist |
//! |---|---|
//! | Alle leidingen binnen thermische begrenzing, goed geïsoleerd | 0,95 |
//! | Leidingen deels onverwarmd (kruipruimte, zolder), matig geïsoleerd | 0,90 |
//! | Ongeïsoleerde leidingen door onverwarmde ruimte | 0,80 |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{HeatingCalcResult, HeatingError};

/// Distributie-systeem met scalair η_dist.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DistributionSystem {
    /// Distributie-rendement η_dist, 0 < η ≤ 1. V1 default 0,95.
    pub efficiency: f64,
}

impl DistributionSystem {
    /// V1 default — goed geïsoleerd distributienet, alle leidingen binnen
    /// thermische begrenzing: η_dist = 0,95.
    #[must_use]
    pub const fn default_insulated() -> Self {
        Self { efficiency: 0.95 }
    }

    /// Matige isolatie, leidingen deels in onverwarmde ruimten: η_dist = 0,90.
    #[must_use]
    pub const fn moderate() -> Self {
        Self { efficiency: 0.90 }
    }

    /// Ongeïsoleerd distributienet door onverwarmde ruimte: η_dist = 0,80.
    #[must_use]
    pub const fn uninsulated() -> Self {
        Self { efficiency: 0.80 }
    }

    /// Maak een `DistributionSystem` met een expliciete η_dist.
    ///
    /// # Errors
    ///
    /// [`HeatingError::InvalidEfficiency`] als `efficiency ∉ (0, 1]` of
    /// niet-eindig is.
    pub fn custom(efficiency: f64) -> HeatingCalcResult<Self> {
        if efficiency.is_finite() && efficiency > 0.0 && efficiency <= 1.0 {
            Ok(Self { efficiency })
        } else {
            Err(HeatingError::InvalidEfficiency {
                name: "η_dist",
                value: efficiency,
                upper: 1.0,
            })
        }
    }

    /// Valideer dat het opgeslagen η_dist in (0, 1] valt.
    ///
    /// # Errors
    ///
    /// [`HeatingError::InvalidEfficiency`] als het niet zo is.
    pub fn validated(&self) -> HeatingCalcResult<f64> {
        if self.efficiency.is_finite() && self.efficiency > 0.0 && self.efficiency <= 1.0 {
            Ok(self.efficiency)
        } else {
            Err(HeatingError::InvalidEfficiency {
                name: "η_dist",
                value: self.efficiency,
                upper: 1.0,
            })
        }
    }
}

impl Default for DistributionSystem {
    fn default() -> Self {
        Self::default_insulated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_0_95() {
        assert!((DistributionSystem::default().efficiency - 0.95).abs() < 1e-12);
    }

    #[test]
    fn moderate_is_0_90() {
        assert!((DistributionSystem::moderate().efficiency - 0.90).abs() < 1e-12);
    }

    #[test]
    fn uninsulated_is_0_80() {
        assert!((DistributionSystem::uninsulated().efficiency - 0.80).abs() < 1e-12);
    }

    #[test]
    fn custom_accepts_valid() {
        let d = DistributionSystem::custom(0.87).unwrap();
        assert!((d.efficiency - 0.87).abs() < 1e-12);
    }

    #[test]
    fn custom_rejects_zero() {
        assert!(DistributionSystem::custom(0.0).is_err());
    }

    #[test]
    fn custom_rejects_above_one() {
        assert!(DistributionSystem::custom(1.5).is_err());
    }

    #[test]
    fn custom_rejects_nan() {
        assert!(DistributionSystem::custom(f64::NAN).is_err());
    }

    #[test]
    fn serde_round_trip() {
        let d = DistributionSystem::moderate();
        let json = serde_json::to_string(&d).unwrap();
        let back: DistributionSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
