//! Douchewater-warmteterugwinning (DWTW) — bijlage U vereenvoudigd.
//!
//! NTA 8800:2025+C1:2026 §13.5 + bijlage U detail:
//! - Formule 13.51: `Q_W;rcd;d = C_W;nd;sh × Q_W;nd;d × η_W;sh;rcd × f_prac;sh × C_W;sh;rcd;T × C_W;sh;rcd;conf`
//!
//! ## V1 vereenvoudiging
//!
//! V1 reduceert dit tot één netto thermisch rendement η dat direct op het
//! douche-aandeel van Q_W;nd wordt toegepast:
//!
//! ```text
//! Q_rcd = η × C_sh × Q_W;nd         (V1)
//! ```
//!
//! met:
//! - η — user-supplied netto rendement (typisch 0,25-0,50 voor consumenten-units)
//!   dat f_prac, C_T en C_conf al incorporeert.
//! - C_sh — vast 0,4 (schatting: 40 % van de warmtapwaterbehoefte komt uit de
//!   douche voor een typische woning). De norm geeft in §13.5.3 gebruiks­
//!   functie-specifieke waarden; V1 gebruikt dit single-point default om de
//!   API simpel te houden.
//!
//! Projecten met meerdere douches waarvan niet alle zijn uitgerust met DWTW
//! moeten η vooraf middelen volgens bijlage U (formules 13.54-13.57).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{DhwCalcResult, DhwError};

/// Aandeel van douche in de nettowarmtebehoefte warm tapwater (C_W;nd;sh).
///
/// NTA 8800 §13.5.3 geeft waarden per gebruiksfunctie. V1 gebruikt een single
/// engineering-default 0,4 voor woningbouw; utiliteit zonder douche-verbruik
/// (kantoor, winkel) krijgt typisch 0.
pub const DEFAULT_DOUCHE_AANDEEL: f64 = 0.4;

/// Douchewater-warmteterugwinning unit (V1 vereenvoudigd).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DouchewtwRecovery {
    /// Netto thermisch rendement van de DWTW-unit (f_prac × C_T × C_conf
    /// × η_W;sh;rcd). Typisch 0,25-0,50 voor consumenten-units.
    pub efficiency: f64,
    /// Aandeel douche in Q_W;nd (C_W;nd;sh). Default [`DEFAULT_DOUCHE_AANDEEL`].
    pub douche_aandeel: f64,
}

impl DouchewtwRecovery {
    /// Constructor met douche-aandeel = [`DEFAULT_DOUCHE_AANDEEL`].
    #[must_use]
    pub const fn new(efficiency: f64) -> Self {
        Self {
            efficiency,
            douche_aandeel: DEFAULT_DOUCHE_AANDEEL,
        }
    }

    /// Constructor met expliciet douche-aandeel (bv. utiliteit zonder
    /// douche-verbruik: 0,0; sportfunctie: 0,8).
    #[must_use]
    pub const fn with_aandeel(efficiency: f64, douche_aandeel: f64) -> Self {
        Self {
            efficiency,
            douche_aandeel,
        }
    }

    /// Valideert dat beide factoren in [0, 1] vallen en eindig zijn.
    ///
    /// Een DWTW-rendement van 0 = geen effect (geen eenheid geïnstalleerd),
    /// rendement 1 = volledig recuperatie (theoretisch maximum, niet reëel).
    ///
    /// # Errors
    ///
    /// [`DhwError::InvalidEfficiency`] bij ongeldige waarden.
    pub fn validated(&self) -> DhwCalcResult<(f64, f64)> {
        if !(self.efficiency.is_finite() && self.efficiency >= 0.0 && self.efficiency <= 1.0) {
            return Err(DhwError::InvalidEfficiency {
                name: "η_W;sh;rcd (DWTW)",
                value: self.efficiency,
                upper: 1.0,
            });
        }
        if !(self.douche_aandeel.is_finite()
            && self.douche_aandeel >= 0.0
            && self.douche_aandeel <= 1.0)
        {
            return Err(DhwError::InvalidEfficiency {
                name: "C_W;nd;sh (aandeel douche)",
                value: self.douche_aandeel,
                upper: 1.0,
            });
        }
        Ok((self.efficiency, self.douche_aandeel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_default_aandeel() {
        let r = DouchewtwRecovery::new(0.4);
        assert!((r.douche_aandeel - DEFAULT_DOUCHE_AANDEEL).abs() < 1e-12);
    }

    #[test]
    fn validated_accepts_zero_efficiency() {
        // η=0 moet valide zijn (= geen DWTW effect).
        let (e, c) = DouchewtwRecovery::new(0.0).validated().unwrap();
        assert!(e.abs() < 1e-12);
        assert!((c - 0.4).abs() < 1e-12);
    }

    #[test]
    fn validated_accepts_one_efficiency() {
        let (e, _) = DouchewtwRecovery::new(1.0).validated().unwrap();
        assert!((e - 1.0).abs() < 1e-12);
    }

    #[test]
    fn validated_rejects_above_one() {
        let err = DouchewtwRecovery::new(1.1).validated().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_negative() {
        let err = DouchewtwRecovery::new(-0.01).validated().unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_nan_aandeel() {
        let err = DouchewtwRecovery::with_aandeel(0.4, f64::NAN)
            .validated()
            .unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let r = DouchewtwRecovery::with_aandeel(0.45, 0.5);
        let json = serde_json::to_string(&r).unwrap();
        let back: DouchewtwRecovery = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
