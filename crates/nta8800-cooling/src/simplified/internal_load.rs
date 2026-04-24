//! Formules (AA.1), (AA.2), (AA.3a) en (AA.3b) — interne warmtelast.
//!
//! Implementeert per rekenzone de basiswaarde `N_int;zi`, de rekenwaarde per
//! vierkante meter `q_int;calc` en de verdeling naar woonkamer/keuken
//! (AA.3a) vs. overige verblijfsruimten (AA.3b).

use crate::errors::{CoolingCalcResult, CoolingError};

/// Formule (AA.1) — basiswaarde interne warmtelast in rekenzone.
///
/// `N_int;zi = 180 · N_woon;zi · P_p;woon;zi`  [W]
///
/// # Parameters
/// - `dwelling_count` — `N_woon;zi`, aantal woonfuncties in rekenzone (NTA
///   §6.6.6).
/// - `persons_per_dwelling` — `P_p;woon`, gemiddeld aantal bewoners per
///   woonfunctie (conform 7.5.2.1, forfaitair).
///
/// # Errors
/// Geeft [`CoolingError::InvalidPersonCount`] als `persons_per_dwelling ≤ 0`
/// of niet-eindig is.
pub fn interne_warmtelast_basis(
    dwelling_count: u32,
    persons_per_dwelling: f64,
) -> CoolingCalcResult<f64> {
    if !persons_per_dwelling.is_finite() || persons_per_dwelling <= 0.0 {
        return Err(CoolingError::InvalidPersonCount {
            persons: persons_per_dwelling,
        });
    }
    Ok(180.0 * f64::from(dwelling_count) * persons_per_dwelling)
}

/// Formule (AA.2) — rekenwaarde interne warmtelast per m².
///
/// `q_int;calc;zi = N_int;zi / (2 · A_vr;woon;zi + A_vr;overig;zi)`  [W/m²]
///
/// Woon/keuken/eetkamer tellen dubbel in de noemer (zie formule (AA.3a)
/// opmerking 5).
///
/// # Errors
/// Geeft [`CoolingError::InvalidFloorArea`] als de totale oppervlakte ≤ 0 is.
pub fn interne_warmtelast_rekenwaarde(
    n_int_w: f64,
    living_area_m2: f64,
    other_area_m2: f64,
) -> CoolingCalcResult<f64> {
    if !living_area_m2.is_finite() || living_area_m2 < 0.0 {
        return Err(CoolingError::InvalidFloorArea {
            area_m2: living_area_m2,
        });
    }
    if !other_area_m2.is_finite() || other_area_m2 < 0.0 {
        return Err(CoolingError::InvalidFloorArea {
            area_m2: other_area_m2,
        });
    }
    let denominator = 2.0 * living_area_m2 + other_area_m2;
    if denominator <= 0.0 {
        return Err(CoolingError::InvalidFloorArea {
            area_m2: denominator,
        });
    }
    Ok(n_int_w / denominator)
}

/// Formule (AA.3a) — interne warmtelast voor woonkamer/keuken/eetkamer.
///
/// `P_int;calc;woon;zi = 2 · q_int;calc;zi · A_vg;woon;zi`  [W]
#[must_use]
pub fn interne_warmtelast_woon(q_int_calc_w_per_m2: f64, living_area_m2: f64) -> f64 {
    2.0 * q_int_calc_w_per_m2 * living_area_m2
}

/// Formule (AA.3b) — interne warmtelast voor overige verblijfsruimten.
///
/// `P_int;calc;overig;zi = q_int;calc;zi · A_vg;overig;zi`  [W]
#[must_use]
pub fn interne_warmtelast_overig(q_int_calc_w_per_m2: f64, other_area_m2: f64) -> f64 {
    q_int_calc_w_per_m2 * other_area_m2
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn aa1_basis_1_woning_3_bewoners() {
        // AA.1: 180 × 1 × 3 = 540 W
        let n_int = interne_warmtelast_basis(1, 3.0).unwrap();
        assert_abs_diff_eq!(n_int, 540.0, epsilon = 1e-9);
    }

    #[test]
    fn aa1_basis_appartementencomplex() {
        // 24 woningen × 2,5 bewoners × 180 = 10 800 W
        let n_int = interne_warmtelast_basis(24, 2.5).unwrap();
        assert_abs_diff_eq!(n_int, 10_800.0, epsilon = 1e-9);
    }

    #[test]
    fn aa1_rejects_zero_persons() {
        let err = interne_warmtelast_basis(1, 0.0).unwrap_err();
        assert!(matches!(err, CoolingError::InvalidPersonCount { .. }));
    }

    #[test]
    fn aa2_rekenwaarde_voorbeeld_woning() {
        // Uit opdracht: 540 W / (2 × 80 + 40) = 540 / 200 = 2,7 W/m²
        let q = interne_warmtelast_rekenwaarde(540.0, 80.0, 40.0).unwrap();
        assert_abs_diff_eq!(q, 2.7, epsilon = 1e-9);
    }

    #[test]
    fn aa2_rejects_zero_area() {
        let err = interne_warmtelast_rekenwaarde(540.0, 0.0, 0.0).unwrap_err();
        assert!(matches!(err, CoolingError::InvalidFloorArea { .. }));
    }

    #[test]
    fn aa3a_woon_verdubbelt() {
        // q = 2,7 W/m², A_woon = 80 m² → 2 × 2,7 × 80 = 432 W
        let p_woon = interne_warmtelast_woon(2.7, 80.0);
        assert_abs_diff_eq!(p_woon, 432.0, epsilon = 1e-9);
    }

    #[test]
    fn aa3b_overig_enkelvoud() {
        // q = 2,7 W/m², A_overig = 40 m² → 1 × 2,7 × 40 = 108 W
        let p_overig = interne_warmtelast_overig(2.7, 40.0);
        assert_abs_diff_eq!(p_overig, 108.0, epsilon = 1e-9);
    }

    #[test]
    fn aa3_som_komt_overeen_met_aa1_maal_factor() {
        // P_woon + P_overig = 432 + 108 = 540 W (gelijk aan N_int;zi omdat
        // de rekenwaarde exact 2 × A_woon + A_overig als noemer gebruikt)
        let n_int = interne_warmtelast_basis(1, 3.0).unwrap();
        let q = interne_warmtelast_rekenwaarde(n_int, 80.0, 40.0).unwrap();
        let p_woon = interne_warmtelast_woon(q, 80.0);
        let p_overig = interne_warmtelast_overig(q, 40.0);
        assert_abs_diff_eq!(p_woon + p_overig, 540.0, epsilon = 1e-9);
    }
}
