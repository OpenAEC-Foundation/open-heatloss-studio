//! Maand-balans — combineer Q_ht + Q_gn + η tot Q_H,nd en Q_C,nd.
//!
//! NTA 8800 §7.4 formule (7.4):  `Q_H;nd = Q_H;ht − η_H;gn · Q_H;gn`
//! NTA 8800 §7.5 formule (7.10): `Q_C;nd = Q_C;gn − η_C;ls · Q_C;ht`
//!
//! In V1 gebruiken we dezelfde `Q_ht` en `Q_gn` voor de koude-balans (de H/C-
//! setpoints leiden tot dezelfde transmissie/ventilatie-profielen wanneer die
//! al door de upstream-crates zijn geleverd). Beide resultaten worden geclampt
//! op 0 — negatieve behoeften bestaan niet fysisch.

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;

/// Clamp een Energy-waarde op 0 ondergrens.
#[inline]
#[must_use]
fn clamp_nonneg(x: Energy) -> Energy {
    if x < 0.0 {
        0.0
    } else {
        x
    }
}

/// Bereken γ = Q_gn / Q_ht.
///
/// Retourneert `+∞` voor `Q_ht ≤ 0` (geen warmtevraag; in dat geval is η
/// niet gedefinieerd in [`super::utilization::utilization_heating`], die
/// retourneert dan 1,0 en daarna clampt `heating_demand` op 0).
#[must_use]
pub fn gamma(q_gn: Energy, q_ht: Energy) -> f64 {
    if q_ht <= 0.0 || !q_ht.is_finite() {
        return f64::INFINITY;
    }
    q_gn / q_ht
}

/// Maandelijkse netto warmtebehoefte — formule (7.4), geclampt ≥ 0.
#[must_use]
pub fn heating_demand(q_ht: Energy, q_gn: Energy, eta_h_gn: f64) -> Energy {
    clamp_nonneg(q_ht - eta_h_gn * q_gn)
}

/// Maandelijkse netto koudebehoefte — formule (7.10), geclampt ≥ 0.
#[must_use]
pub fn cooling_demand(q_ht: Energy, q_gn: Energy, eta_c_ls: f64) -> Energy {
    clamp_nonneg(q_gn - eta_c_ls * q_ht)
}

/// Som van een maandprofiel over 12 maanden.
#[must_use]
pub fn annual_sum(profile: &MonthlyProfile<Energy>) -> Energy {
    Month::all().iter().map(|&m| profile[m]).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heating_koude_maand_positief() {
        // Veel verlies, weinig winst → Q_nd ≈ Q_ht
        let q = heating_demand(1000.0, 100.0, 0.9);
        assert!((q - 910.0).abs() < 1e-9);
    }

    #[test]
    fn heating_warme_maand_geclampt_op_nul() {
        // Winst groter dan verlies × η → zou negatief worden → clamp 0
        let q = heating_demand(100.0, 500.0, 0.9);
        assert!(q.abs() < 1e-9);
    }

    #[test]
    fn cooling_warme_maand_positief() {
        // Veel winst, weinig verlies benutbaar → Q_C;nd > 0
        let q = cooling_demand(500.0, 2000.0, 0.6);
        assert!((q - (2000.0 - 0.6 * 500.0)).abs() < 1e-9);
    }

    #[test]
    fn cooling_koude_maand_geclampt_op_nul() {
        // Meer verlies dan winst → Q_C;nd < 0 → clamp 0
        let q = cooling_demand(2000.0, 500.0, 0.9);
        assert!(q.abs() < 1e-9);
    }

    #[test]
    fn gamma_q_ht_nul_geeft_infinity() {
        let g = gamma(100.0, 0.0);
        assert!(g.is_infinite());
    }

    #[test]
    fn gamma_q_ht_negatief_geeft_infinity() {
        let g = gamma(100.0, -50.0);
        assert!(g.is_infinite());
    }

    #[test]
    fn gamma_standaard_verhouding() {
        let g = gamma(500.0, 1000.0);
        assert!((g - 0.5).abs() < 1e-9);
    }

    #[test]
    fn annual_sum_constant_profiel() {
        let p = MonthlyProfile::from_constant(100.0_f64);
        assert!((annual_sum(&p) - 1200.0).abs() < 1e-9);
    }
}
