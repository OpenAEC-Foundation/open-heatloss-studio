//! Afgifterendement warm tapwater `η_W;em`.
//!
//! NTA 8800:2025+C1:2026 §13.3 drukt de afgifteverliezen (warmteverliezen aan
//! tappunt + warmteverliezen uittapleidingen) uit in een jaar-rendement per
//! warmtapwatersysteem. V1 reduceert dit tot één η per systeem.
//!
//! ## V1 defaults
//!
//! Bron tabellen:
//!
//! | Tabel | Categorie | Rendement |
//! |---|---|---|
//! | 13.2 | Woningbouw — keuken l_k ∈ [2,4⟩ m | η_W;em;k = 0,69 |
//! | 13.2 | Woningbouw — badruimte l_b ∈ [2,4⟩ m | η_W;em;b = 0,95 |
//! | 13.3 | Utiliteit — gemiddelde uittapleiding ≤ 3 m | η_W;em = 1,0 |
//! | 13.3 | Utiliteit — gemiddelde uittapleiding > 3 m | η_W;em = 0,8 |
//!
//! De woningbouw-default [`DhwEmission::default_woning`] combineert keuken
//! (C_W;nd;k = 0,2) en badruimte (C_W;nd;b = 0,8) volgens formule 13.23 met
//! typische waarden l_k = 3 m en l_b = 3 m ⇒ η ≈ 1 / (0,8/0,95 + 0,2/0,69) ≈ 0,87.
//! Voor situaties met langere keuken-leiding (l_k > 4 m) zal het rendement
//! snel zakken — gebruik `Custom { efficiency }`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{DhwCalcResult, DhwError};

/// Afgifterendement-systeem voor warm tapwater (V1: forfaitair of custom).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DhwEmission {
    /// Woningbouw default — typische rijwoning/appartement met korte
    /// uittapleidingen (l_k ≈ 3 m, l_b ≈ 3 m). η_W;em ≈ 0,87 volgens
    /// formule 13.23 met η_W;em;k = 0,69 en η_W;em;b = 0,95.
    WoningDefault,

    /// Utiliteit met korte uittapleidingen (gemiddeld ≤ 3 m). Tabel 13.3.
    UtiliteitKort,

    /// Utiliteit met lange uittapleidingen (gemiddeld > 3 m). Tabel 13.3.
    UtiliteitLang,

    /// User-supplied custom η_W;em (0 < η ≤ 1).
    Custom {
        /// η_W;em waarde in (0, 1].
        efficiency: f64,
    },
}

impl DhwEmission {
    /// Default η_W;em per variant.
    #[must_use]
    pub fn default_efficiency(&self) -> f64 {
        match self {
            // Formule 13.23: η = 1 / (C_b/η_b + C_k/η_k)
            // met C_b=0,8, η_b=0,95, C_k=0,2, η_k=0,69
            // = 1 / (0,8421 + 0,2899) = 1 / 1,1320 ≈ 0,8834
            DhwEmission::WoningDefault => {
                let c_b: f64 = 0.8;
                let c_k: f64 = 0.2;
                let eta_b: f64 = 0.95;
                let eta_k: f64 = 0.69;
                1.0 / (c_b / eta_b + c_k / eta_k)
            }
            DhwEmission::UtiliteitKort => 1.0,
            DhwEmission::UtiliteitLang => 0.8,
            DhwEmission::Custom { efficiency } => *efficiency,
        }
    }

    /// Constructor-helper voor de `Custom`-variant.
    #[must_use]
    pub const fn custom(efficiency: f64) -> Self {
        DhwEmission::Custom { efficiency }
    }

    /// Valideert dat η_W;em in (0, 1] valt en eindig is.
    ///
    /// # Errors
    ///
    /// [`DhwError::InvalidEfficiency`] als de waarde ongeldig is.
    pub fn validated_efficiency(&self) -> DhwCalcResult<f64> {
        let eta = self.default_efficiency();
        if eta.is_finite() && eta > 0.0 && eta <= 1.0 {
            Ok(eta)
        } else {
            Err(DhwError::InvalidEfficiency {
                name: "η_W;em",
                value: eta,
                upper: 1.0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn woning_default_near_87_percent() {
        let eta = DhwEmission::WoningDefault.default_efficiency();
        assert_relative_eq!(eta, 0.8834, max_relative = 1e-3);
    }

    #[test]
    fn utiliteit_kort_is_one() {
        assert!((DhwEmission::UtiliteitKort.default_efficiency() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn utiliteit_lang_is_point_eight() {
        assert!((DhwEmission::UtiliteitLang.default_efficiency() - 0.8).abs() < 1e-12);
    }

    #[test]
    fn custom_passes_through() {
        assert!((DhwEmission::custom(0.77).default_efficiency() - 0.77).abs() < 1e-12);
    }

    #[test]
    fn validated_rejects_zero() {
        let err = DhwEmission::custom(0.0).validated_efficiency().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_above_one() {
        let err = DhwEmission::custom(1.1).validated_efficiency().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_nan() {
        let err = DhwEmission::custom(f64::NAN)
            .validated_efficiency()
            .unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn serde_round_trip_unit() {
        let s = DhwEmission::WoningDefault;
        let json = serde_json::to_string(&s).unwrap();
        let back: DhwEmission = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_round_trip_custom() {
        let s = DhwEmission::custom(0.82);
        let json = serde_json::to_string(&s).unwrap();
        let back: DhwEmission = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
