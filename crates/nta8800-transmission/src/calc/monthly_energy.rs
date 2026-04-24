//! Maand-energie conversie: `Q = H · ΔT · t · 0.001 · 3.6` van W/K naar MJ.
//!
//! Dit is de gedeelde primitive die [`super::calculate_transmission`] gebruikt
//! voor alle boundary-types. Apart geëxposed om in tests en in
//! downstream-crates (zoals `nta8800-demand`) hergebruikt te kunnen worden.
//!
//! Formule (7.14) rekent in kWh: `Q[kWh] = H · ΔT · 0.001 · t_mi`. Deze crate
//! drukt energie uit in MJ (zie [`nta8800_model::units::Energy`]), dus
//! vermenigvuldigen we met 3.6.

use nta8800_model::units::{Energy, Temperature};

use super::KWH_TO_MJ;

/// Bereken `Q = H · ΔT · 0.001 · t_mi · 3.6` — maandelijkse transmissiewarmte
/// in MJ.
///
/// # Argumenten
/// - `h` — warmteoverdrachtcoëfficiënt in W/K.
/// - `delta_t` — temperatuurverschil `θ_i − θ_e` in K (of equivalent °C).
/// - `t_mi` — lengte van de maand in uren (`MONTH_HOURS[idx]`).
///
/// # Voorbeeld
/// ```
/// # use nta8800_transmission::calc::monthly_energy::monthly_energy_mj;
/// let h = 10.0;        // W/K
/// let delta_t = 15.0;  // K (bv. 20 °C binnen − 5 °C buiten)
/// let t_mi = 744.0;    // uur (januari)
/// let q_mj = monthly_energy_mj(h, delta_t, t_mi);
/// // 10 · 15 · 0.001 · 744 = 111.6 kWh = 401.76 MJ
/// assert!((q_mj - 401.76).abs() < 1e-6);
/// ```
#[must_use]
pub fn monthly_energy_mj(h: f64, delta_t: Temperature, t_mi: f64) -> Energy {
    h * delta_t * 0.001 * t_mi * KWH_TO_MJ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_h_yields_zero_energy() {
        assert!((monthly_energy_mj(0.0, 20.0, 744.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_delta_t_yields_zero_energy() {
        assert!((monthly_energy_mj(100.0, 0.0, 744.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn standard_january_example() {
        // h = 10 W/K, ΔT = 15 K, t = 744 h → 10·15·744 = 111 600 Wh = 111.6 kWh = 401.76 MJ
        let mj = monthly_energy_mj(10.0, 15.0, 744.0);
        assert!((mj - 401.76).abs() < 1e-6, "got {mj}");
    }

    #[test]
    fn negative_delta_t_gives_negative_energy() {
        // Bij θ_e > θ_i (bv. zomer) mag het resultaat negatief zijn —
        // NTA 8800 hoofdstuk 7 OPMERKING 3 documenteert dit expliciet.
        let mj = monthly_energy_mj(10.0, -5.0, 744.0);
        assert!(mj < 0.0);
    }

    #[test]
    fn linear_in_h() {
        let a = monthly_energy_mj(1.0, 10.0, 744.0);
        let b = monthly_energy_mj(2.0, 10.0, 744.0);
        assert!((b - 2.0 * a).abs() < 1e-9);
    }

    #[test]
    fn linear_in_delta_t() {
        let a = monthly_energy_mj(5.0, 1.0, 744.0);
        let b = monthly_energy_mj(5.0, 3.0, 744.0);
        assert!((b - 3.0 * a).abs() < 1e-9);
    }
}
