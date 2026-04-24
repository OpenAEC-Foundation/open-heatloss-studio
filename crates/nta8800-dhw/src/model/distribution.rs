//! Distributierendement warm tapwater `η_W;dis`.
//!
//! NTA 8800:2025+C1:2026 §13.4 drukt de distributieverliezen uit in een
//! rendement per warmtapwatersysteem. V1 reduceert dit tot één forfaitaire
//! factor per systeem. Conversieverlies van afleversets (formule 13.24a) en
//! lineaire circulatieleiding-verliezen (leidinglengte × lineair verlies,
//! formule 13.25) zijn V2 scope.
//!
//! ## V1 defaults
//!
//! | Variant | η_W;dis | Context |
//! |---|---|---|
//! | [`DhwDistribution::default_individueel`] | 1,00 | Individueel toestel (combi-ketel) zonder circulatieleiding; alle distributie-verliezen zitten in η_W;em |
//! | [`DhwDistribution::default_circulatie`] | 0,85 | Woongebouw met circulatieleiding, goed geïsoleerd |
//! | [`DhwDistribution::custom`] | user | Kwaliteitsverklaring of meet-data |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{DhwCalcResult, DhwError};

/// Distributierendement-systeem voor warm tapwater (V1: forfaitair of custom).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DhwDistribution {
    /// η_W;dis in (0, 1].
    pub efficiency: f64,
}

impl DhwDistribution {
    /// Individueel tapwatertoestel (combi-ketel zonder circulatieleiding).
    ///
    /// η_W;dis = 1,0 — alle uittapleiding-verliezen zitten al in η_W;em.
    #[must_use]
    pub const fn default_individueel() -> Self {
        Self { efficiency: 1.0 }
    }

    /// Woongebouw of utiliteit met circulatieleiding (goed geïsoleerd).
    ///
    /// V1 engineering-default η_W;dis = 0,85. De werkelijke waarde hangt
    /// sterk af van leidinglengte, isolatie-klasse en circulatie-regeling.
    /// Voor projecten met specifieke leidinglengte-data: gebruik
    /// [`DhwDistribution::custom`].
    #[must_use]
    pub const fn default_circulatie() -> Self {
        Self { efficiency: 0.85 }
    }

    /// User-supplied distributierendement.
    #[must_use]
    pub const fn custom(efficiency: f64) -> Self {
        Self { efficiency }
    }

    /// Valideert dat η_W;dis in (0, 1] valt.
    ///
    /// # Errors
    ///
    /// [`DhwError::InvalidEfficiency`] als de waarde ongeldig is.
    pub fn validated(&self) -> DhwCalcResult<f64> {
        if self.efficiency.is_finite() && self.efficiency > 0.0 && self.efficiency <= 1.0 {
            Ok(self.efficiency)
        } else {
            Err(DhwError::InvalidEfficiency {
                name: "η_W;dis",
                value: self.efficiency,
                upper: 1.0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_individueel_is_one() {
        assert!((DhwDistribution::default_individueel().efficiency - 1.0).abs() < 1e-12);
    }

    #[test]
    fn default_circulatie_is_85() {
        assert!((DhwDistribution::default_circulatie().efficiency - 0.85).abs() < 1e-12);
    }

    #[test]
    fn validated_rejects_zero() {
        let err = DhwDistribution::custom(0.0).validated().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_negative() {
        let err = DhwDistribution::custom(-0.1).validated().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_above_one() {
        let err = DhwDistribution::custom(1.05).validated().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_accepts_valid() {
        let eta = DhwDistribution::custom(0.90).validated().unwrap();
        assert!((eta - 0.90).abs() < 1e-12);
    }

    #[test]
    fn serde_round_trip() {
        let d = DhwDistribution::custom(0.88);
        let json = serde_json::to_string(&d).unwrap();
        let back: DhwDistribution = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
