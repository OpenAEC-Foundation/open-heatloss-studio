//! Regeling-systeem met forfaitaire factor f_reg.
//!
//! NTA 8800 §9.6 beschrijft de regeling van het verwarmingssysteem. Voor V1
//! vereenvoudigen we tot één scalair dat de mate van overschot-verwarming en
//! onjuiste setpoint-volging uitdrukt.
//!
//! Defaults:
//!
//! | Regeling | f_reg |
//! |---|---|
//! | Aan/uit thermostaat (puur setpoint) | 1,00 |
//! | Modulerend (proportioneel) | 0,98 |
//! | Weersafhankelijk + ruimtethermostaat | 0,95 |
//! | Geen regeling (handmatig) | 1,05 — hogere verliezen → efficiency ↓ |
//!
//! De keuze om 1,05 voor "geen regeling" toe te staan (> 1) wijkt af van de
//! standaard η-range (0, 1] — in V1 interpreteren we f_reg strikt ∈ (0, 1].
//! "Geen regeling" mapt daarom op f_reg = 1,0 en accepteert dat de
//! Q_H;nd dan al gecorrigeerd moet zijn via de setpoint.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{HeatingCalcResult, HeatingError};

/// Regel-rendement f_reg (dimensieloos, 0 < f ≤ 1).
///
/// Hogere f_reg = nauwkeurigere setpoint-volging = minder overschot-verwarming.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ControlFactor {
    /// f_reg waarde. V1 default 1,0 (aan/uit thermostaat).
    pub factor: f64,
}

impl ControlFactor {
    /// Aan/uit thermostaat — f_reg = 1,0.
    #[must_use]
    pub const fn on_off() -> Self {
        Self { factor: 1.0 }
    }

    /// Modulerend / proportioneel-regelend — f_reg = 0,98.
    #[must_use]
    pub const fn modulating() -> Self {
        Self { factor: 0.98 }
    }

    /// Weersafhankelijk met ruimtethermostaat — f_reg = 0,95.
    #[must_use]
    pub const fn weather_compensated() -> Self {
        Self { factor: 0.95 }
    }

    /// Maak een `ControlFactor` met expliciete f_reg.
    ///
    /// # Errors
    ///
    /// [`HeatingError::InvalidEfficiency`] als `factor ∉ (0, 1]` of
    /// niet-eindig is.
    pub fn custom(factor: f64) -> HeatingCalcResult<Self> {
        if factor.is_finite() && factor > 0.0 && factor <= 1.0 {
            Ok(Self { factor })
        } else {
            Err(HeatingError::InvalidEfficiency {
                name: "f_reg",
                value: factor,
                upper: 1.0,
            })
        }
    }

    /// Valideer dat de opgeslagen factor in (0, 1] valt.
    ///
    /// # Errors
    ///
    /// [`HeatingError::InvalidEfficiency`] als dat niet zo is.
    pub fn validated(&self) -> HeatingCalcResult<f64> {
        if self.factor.is_finite() && self.factor > 0.0 && self.factor <= 1.0 {
            Ok(self.factor)
        } else {
            Err(HeatingError::InvalidEfficiency {
                name: "f_reg",
                value: self.factor,
                upper: 1.0,
            })
        }
    }
}

impl Default for ControlFactor {
    fn default() -> Self {
        Self::on_off()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_off_is_one() {
        assert!((ControlFactor::on_off().factor - 1.0).abs() < 1e-12);
    }

    #[test]
    fn modulating_is_0_98() {
        assert!((ControlFactor::modulating().factor - 0.98).abs() < 1e-12);
    }

    #[test]
    fn weather_is_0_95() {
        assert!((ControlFactor::weather_compensated().factor - 0.95).abs() < 1e-12);
    }

    #[test]
    fn custom_rejects_zero() {
        assert!(ControlFactor::custom(0.0).is_err());
    }

    #[test]
    fn custom_accepts_one() {
        assert!(ControlFactor::custom(1.0).is_ok());
    }

    #[test]
    fn custom_rejects_above_one() {
        assert!(ControlFactor::custom(1.2).is_err());
    }

    #[test]
    fn serde_round_trip() {
        let c = ControlFactor::weather_compensated();
        let json = serde_json::to_string(&c).unwrap();
        let back: ControlFactor = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }
}
