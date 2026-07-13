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

/// Maandelijkse netto koudebehoefte mét de NTA 8800 §7.2.2-poort — formules
/// (7.6)/(7.7).
///
/// Formule (7.7) is `Q_C;nd = Q_C;gn − η_C;ht · Q_C;ht` (idem [`cooling_demand`]),
/// maar §7.2.2 zet die op **0** zodra de verliezen de winst met meer dan een
/// factor 2 overheersen (formule 7.6):
///
/// ```text
/// indien (1/γ_C) > 2,0:  Q_C;nd = 0        met γ_C = Q_C;gn / Q_C;ht
/// ```
///
/// Dus `1/γ_C = Q_C;ht / Q_C;gn > 2` → maand telt niet als koelmaand. Zonder deze
/// poort blijven schouderseizoen-maanden (waar `η_C;ht` de verliezen niet volledig
/// verrekent) een residuele koudebehoefte houden die de norm expliciet afkapt.
///
/// `q_c_ht` is de warmteoverdracht voor **koeling** (§7.2.3, formule 7.15/7.16:
/// getransmitteerd/geventileerd tegen de koel-setpoint θ_int;set;C), niet de
/// verwarmings-`Q_H;ht`.
#[must_use]
pub fn cooling_demand_gated(q_c_ht: Energy, q_c_gn: Energy, eta_c_ht: f64) -> Energy {
    // Geen (positieve) winst → geen koudebehoefte (γ_C ≤ 0).
    if q_c_gn <= 0.0 {
        return 0.0;
    }
    // Formule (7.6): (1/γ_C) > 2,0 met γ_C = Q_C;gn / Q_C;ht ⇒ Q_C;ht / Q_C;gn > 2.
    if q_c_ht / q_c_gn > 2.0 {
        return 0.0;
    }
    clamp_nonneg(q_c_gn - eta_c_ht * q_c_ht)
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
    fn gated_poort_kapt_verliesgedomineerde_maand_af() {
        // 1/γ_C = Q_C;ht/Q_C;gn = 3000/1000 = 3 > 2 → formule (7.6) → 0,
        // óók al zou (7.7) een positieve rest geven bij lage η.
        let q = cooling_demand_gated(3000.0, 1000.0, 0.3);
        assert!(q.abs() < 1e-12, "poort moet 0 geven, kreeg {q}");
    }

    #[test]
    fn gated_koelmaand_gelijk_aan_ongepoort() {
        // 1/γ_C = 500/2000 = 0,25 ≤ 2 → poort inactief → identiek aan (7.10).
        let g = cooling_demand_gated(500.0, 2000.0, 0.6);
        let u = cooling_demand(500.0, 2000.0, 0.6);
        assert!((g - u).abs() < 1e-12);
        assert!((g - (2000.0 - 0.6 * 500.0)).abs() < 1e-9);
    }

    #[test]
    fn gated_grens_precies_twee_telt_nog_mee() {
        // 1/γ_C = 2000/1000 = 2,0 → niet > 2,0 → poort inactief.
        let q = cooling_demand_gated(2000.0, 1000.0, 0.4);
        assert!(q > 0.0, "grens 2,0 mag niet afkappen, kreeg {q}");
    }

    #[test]
    fn gated_geen_winst_geeft_nul() {
        assert!(cooling_demand_gated(1000.0, 0.0, 0.5).abs() < 1e-12);
        assert!(cooling_demand_gated(1000.0, -10.0, 0.5).abs() < 1e-12);
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
