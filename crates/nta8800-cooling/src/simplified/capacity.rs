//! Formules (AA.11) en (AA.13) — benodigde koelcapaciteit.
//!
//! `B_C;req;TO = (q_C − 35) / 1000 · A_g;vr`  [kW]
//!
//! Met een vaste aftrek van 35 W/m² die overeenkomt met net-voldaan aan
//! TO_juli < 1,2 of GTO < 450 h (zie opmerking 1 bij formule (AA.11)).
//! De uitkomst kan niet kleiner dan 0 kW zijn.

use crate::model::FIXED_OUTDOOR_DEDUCTION_W_PER_M2;

/// Formule (AA.11) / (AA.13) — benodigde koelcapaciteit in kW.
///
/// `B_C;req;TO = max(0, (q_C − 35) / 1000 · A)`  [kW]
///
/// Dezelfde formule wordt gebruikt voor zowel de rekenzone (AA.11) als de
/// individuele verblijfsruimte (AA.13); alleen de inputs verschillen.
///
/// # Parameters
/// - `q_c_w_per_m2` — maatgevende koelbehoefte q_C uit AA.8 (zone) of AA.9
///   (verblijfsruimte) in W/m².
/// - `area_m2` — de bijbehorende oppervlakte A_g;vr in m².
///
/// Returns de benodigde capaciteit in kW, altijd ≥ 0.
#[must_use]
pub fn required_cooling_capacity_kw(q_c_w_per_m2: f64, area_m2: f64) -> f64 {
    let raw = (q_c_w_per_m2 - FIXED_OUTDOOR_DEDUCTION_W_PER_M2) / 1000.0 * area_m2;
    if raw < 0.0 {
        0.0
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn aa11_lage_behoefte_clamped_op_nul() {
        // q_C = 20 W/m² < 35 → (20-35)/1000 × 120 = -1,8 → 0 kW
        let b = required_cooling_capacity_kw(20.0, 120.0);
        assert_abs_diff_eq!(b, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn aa11_hoge_behoefte_plausibele_capaciteit() {
        // q_C = 50 W/m² (forse koelbehoefte), 120 m² woning
        // → (50-35)/1000 × 120 = 1,8 kW
        let b = required_cooling_capacity_kw(50.0, 120.0);
        assert_abs_diff_eq!(b, 1.8, epsilon = 1e-9);
    }

    #[test]
    fn aa11_typische_woning_3_5_kw() {
        // q_C = 60 W/m², 150 m² woning → (60-35)/1000 × 150 = 3,75 kW
        let b = required_cooling_capacity_kw(60.0, 150.0);
        assert_abs_diff_eq!(b, 3.75, epsilon = 1e-9);
        // Plausibel bereik 3-5 kW voor typische eengezinswoning
        assert!((3.0..=5.0).contains(&b));
    }

    #[test]
    fn aa11_grens_exact_op_35() {
        let b = required_cooling_capacity_kw(35.0, 120.0);
        assert_abs_diff_eq!(b, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn aa13_per_verblijfsruimte_dezelfde_formule() {
        // Woonkamer 30 m², q_C = 65 W/m² → (65-35)/1000 × 30 = 0,9 kW
        let b = required_cooling_capacity_kw(65.0, 30.0);
        assert_abs_diff_eq!(b, 0.9, epsilon = 1e-9);
    }
}
