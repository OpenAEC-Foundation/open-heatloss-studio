//! Ventilator-energie `W_fan` — forfaitaire methode (NTA 8800 §11.4.3.3).
//!
//! Basisformule (11.142):
//! ```text
//! P_eff = f_SFP · f_systype · q_V;ODA;req · c     (W)
//! ```
//! met `c = 1,0` (tijdgemiddelde-correctie), `q_V` in m³/h, `f_SFP` in
//! W/(m³/h), `f_systype ∈ {0, 1, 2}` voor A / B+C / D+E.
//!
//! Over een maand: `W_fan;mi = P_eff · t_mi · 3600 / 10⁶` in MJ elektrisch.
//!
//! V1-beperking: geen `f_regfan` (luchtvolumestroomregeling-correctie),
//! geen `f_BAL-DEC`/`f_overig` splitsing voor zone-fractie. Deze factoren
//! zijn V2-scope.

use nta8800_model::units::Energy;

/// Bereken ventilator-energie voor één maand in MJ elektrisch.
///
/// # Parameters
///
/// - `f_sfp`: specifiek ventilator-vermogen in W/(m³/h) (NTA 8800 eenheid)
/// - `f_systype`: 0 voor systeem A, 1 voor B/C, 2 voor D/E (zie
///   [`crate::VentilationSystem::f_systype`])
/// - `q_m3_per_h`: mechanische ventilatiestroom in m³/h
/// - `t_mi_hours`: duur van de maand in h
///
/// # Eenheidsanalyse
///
/// `P [W] = W/(m³/h) × (m³/h) = W`. Dan
/// `W_fan [J] = P × t_s = P × (t_h × 3600)`, en `/ 10⁶ → MJ`.
#[must_use]
pub fn fan_energy_mj(f_sfp: f64, f_systype: f64, q_m3_per_h: f64, t_mi_hours: f64) -> Energy {
    let p_eff_w = f_sfp * f_systype * q_m3_per_h;
    let energy_j = p_eff_w * t_mi_hours * 3600.0;
    energy_j / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn system_a_has_zero_fan_energy() {
        // f_systype = 0 → altijd 0 ongeacht debiet
        let w = fan_energy_mj(0.5, 0.0, 200.0, 744.0);
        assert_relative_eq!(w, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn zero_airflow_zero_energy() {
        let w = fan_energy_mj(0.5, 2.0, 0.0, 744.0);
        assert_relative_eq!(w, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn modern_d_system_typical_january() {
        // f_SFP = 0,45/3,6 = 0,125 W/(m³/h) (tabel 11.23, DC, y>2006)
        // f_systype = 2 (D)
        // q = 150 m³/h, t = 744 h
        // P = 0,125 × 2 × 150 = 37,5 W
        // W = 37,5 × 744 × 3600 / 10⁶ = 100,44 MJ
        let w = fan_energy_mj(0.45 / 3.6, 2.0, 150.0, 744.0);
        assert_relative_eq!(w, 100.44, epsilon = 0.1);
    }

    #[test]
    fn january_equals_july_at_constant_fan_speed() {
        // Ventilatoren draaien continu op zelfde toerental → hoeveelheid
        // energie hangt af van t_mi (744 vs 744 voor januari/juli, gelijk).
        let w_jan = fan_energy_mj(0.125, 2.0, 150.0, 744.0);
        let w_jul = fan_energy_mj(0.125, 2.0, 150.0, 744.0);
        assert_relative_eq!(w_jan, w_jul, epsilon = 1e-9);
    }

    #[test]
    fn d_system_double_of_b_at_same_sfp_and_flow() {
        // f_systype_D / f_systype_B = 2/1 = 2× energie
        let w_b = fan_energy_mj(0.125, 1.0, 100.0, 744.0);
        let w_d = fan_energy_mj(0.125, 2.0, 100.0, 744.0);
        assert_relative_eq!(w_d, 2.0 * w_b, epsilon = 1e-6);
    }
}
