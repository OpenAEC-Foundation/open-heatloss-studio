//! Maandelijkse ventilatie-warmteverliezen `Q_V;mi`.
//!
//! Gebaseerd op NTA 8800 formule (11.106) als template:
//! `P = q · ρ_a · c_a · ΔT / 3600` (q in m³/h, P in W)
//!
//! Voor een hele maand met duur `t_mi` uren vermenigvuldigen we met
//! `3600 · t_mi / 10⁶` om J→MJ te krijgen. Resultaat:
//!
//! ```text
//! Q_V;mi [MJ] = q · ρ_a · c_a · (ϑ_i − ϑ_sup) · t_mi / 10⁶
//! ```
//!
//! met `q` in m³/h, `t_mi` in h, temperaturen in °C (verschil = K).

use nta8800_model::units::{Energy, Temperature};

use super::AIR_VOLUMETRIC_HEAT_J_PER_M3_K;

/// Bereken de ventilatie-warmteverlies voor één maand in MJ.
///
/// # Parameters
///
/// - `q_m3_per_h`: totale ventilatiestroom in m³/h
/// - `theta_indoor`: binnentemperatuur ϑ_i in °C
/// - `theta_supply`: temperatuur van toevoerlucht ϑ_sup in °C
///   (zonder WTW = ϑ_e, mét WTW = ϑ_e + η·(ϑ_i − ϑ_e))
/// - `t_mi_hours`: duur van de maand in h (uit tabel 17.1)
///
/// # Clamping
///
/// Als `theta_supply ≥ theta_indoor` (bv. in zomer) → geen warmteverlies
/// maar winst. Q_V wordt afgekapt op 0 voor de **verwarmingsbehoefte**; een
/// negatief getal zou "gratis verwarmen door ventilatie" impliceren wat
/// conceptueel onjuist is binnen H_nd-context. Voor cooling-behoefte wordt
/// dit elders behandeld (V2).
#[must_use]
pub fn heat_loss_mj(
    q_m3_per_h: f64,
    theta_indoor: Temperature,
    theta_supply: Temperature,
    t_mi_hours: f64,
) -> Energy {
    let delta_t = theta_indoor - theta_supply;
    if delta_t <= 0.0 {
        return 0.0;
    }
    // q [m³/h] × ρc [J/m³K] × ΔT [K] × t [h] = J·h/h → watts·h
    // → ×3600 om naar J·s te komen? Nee:
    // q_m³/s = q_m³/h / 3600. P [W] = q_m³/s × ρc × ΔT.
    // Energy = P × t [s] = q_m³/h/3600 × ρc × ΔT × t_h × 3600
    //        = q_m³/h × ρc × ΔT × t_h [J]
    // → eenheidsloos klopt: m³/h × J/(m³·K) × K × h = J (want h·1/h = 1 voor
    //   de 3600-factoren). We hebben dus alleen:  J = q × ρc × ΔT × t_h
    let energy_j = q_m3_per_h * AIR_VOLUMETRIC_HEAT_J_PER_M3_K * delta_t * t_mi_hours;
    energy_j / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn heat_loss_zero_if_supply_equals_indoor() {
        let q = heat_loss_mj(100.0, 20.0, 20.0, 744.0);
        assert_relative_eq!(q, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn heat_loss_zero_if_supply_warmer_than_indoor() {
        // zomer-situatie: lucht komt warmer binnen dan binnen is → geen H-verlies
        let q = heat_loss_mj(100.0, 18.0, 25.0, 744.0);
        assert_relative_eq!(q, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn heat_loss_january_reference_case() {
        // Januari De Bilt: ϑ_e ≈ 2,61 °C, ϑ_i = 20 °C, q = 150 m³/h, t = 744 h
        // Q = 150 × 1212,23 × (20 − 2,61) × 744 / 10⁶
        //   = 150 × 1212,23 × 17,39 × 744 / 10⁶
        //   ≈ 2352 MJ
        let q = heat_loss_mj(150.0, 20.0, 2.61, 744.0);
        assert_relative_eq!(q, 2352.1, epsilon = 1.0);
    }

    #[test]
    fn heat_loss_scales_linearly_with_airflow() {
        let q1 = heat_loss_mj(100.0, 20.0, 5.0, 744.0);
        let q2 = heat_loss_mj(200.0, 20.0, 5.0, 744.0);
        assert_relative_eq!(q2, 2.0 * q1, epsilon = 1e-6);
    }

    #[test]
    fn heat_loss_january_much_larger_than_july() {
        // Jan: ϑ_e = 2,61, Jul: ϑ_e = 17,94 (De Bilt).
        // Q_jan / Q_jul ≈ (20−2,61)/(20−17,94) ≈ 17,39/2,06 ≈ 8,44×
        // (bijna gelijke t_mi: 744 vs 744)
        let q_jan = heat_loss_mj(100.0, 20.0, 2.61, 744.0);
        let q_jul = heat_loss_mj(100.0, 20.0, 17.94, 744.0);
        assert!(
            q_jan > 5.0 * q_jul,
            "januari-verlies moet veel groter zijn dan juli: q_jan={q_jan}, q_jul={q_jul}"
        );
    }
}
