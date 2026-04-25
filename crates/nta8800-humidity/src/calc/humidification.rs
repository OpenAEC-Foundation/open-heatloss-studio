//! Bevochtigings-energie berekeningen.

// Referenties gebruikt in doc-comments via NTA_8800_2025_* constanten

/// Bereken bevochtigingsbehoefte conform NTA 8800 formule (12.1).
///
/// `Q_hum = ṁ_a · Δx · r_w` waarbij:
/// - `ṁ_a` = massastroom droge lucht [kg/h]
/// - `Δx` = vochtigheidsverschil (x_IDA - x_ODA) [kg/kg]
/// - `r_w` = verdampingswarmte water = 2501 kJ/kg
///
/// # Arguments
///
/// * `mass_flow_kg_h` - Massastroom droge lucht in kg/h
/// * `delta_x_kg_per_kg` - Vochtigheidsverschil in kg/kg (positief = bevochtiging)
/// * `duration_hours` - Duur van de periode in uren
///
/// # Returns
///
/// Bevochtigingsbehoefte in MJ thermisch
///
/// # Referenties
///
/// - [`NTA_8800_2025_FORMULE12_1`]: bevochtigingsbehoefte berekening
/// - [`NTA_8800_2025_CONST_R_W_KJ_PER_KG`]: verdampingswarmte water
#[must_use]
pub fn calculate_humidification_energy(
    mass_flow_kg_h: f64,
    delta_x_kg_per_kg: f64,
    duration_hours: f64,
) -> f64 {
    if delta_x_kg_per_kg <= 0.0 {
        return 0.0; // Geen bevochtiging nodig
    }

    // Q_hum = ṁ_a · Δx · r_w · t [kJ]
    let q_hum_kj = mass_flow_kg_h * delta_x_kg_per_kg * super::WATER_LATENT_HEAT_KJ_PER_KG * duration_hours;

    // Converteer kJ naar MJ
    q_hum_kj / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn humidification_energy_calculation() {
        // 120 kg/h luchtstroom, 0.002 kg/kg vochtigheidstoename, 744 uur (januari)
        let mass_flow = 120.0; // kg/h
        let delta_x = 0.002; // kg/kg (2 g/kg)
        let duration = 744.0; // h

        let q_hum = calculate_humidification_energy(mass_flow, delta_x, duration);

        // Verwachte waarde: 120 * 0.002 * 2501 * 744 / 1000 = 447.4 MJ
        assert_abs_diff_eq!(q_hum, 447.4, epsilon = 1.0);
    }

    #[test]
    fn no_humidification_needed() {
        // Negatief of nul vochtigheidsdeficit → geen bevochtiging
        assert_abs_diff_eq!(calculate_humidification_energy(100.0, 0.0, 744.0), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(calculate_humidification_energy(100.0, -0.001, 744.0), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn small_humidification_load() {
        // Kleine luchtstroom, klein vochtigheidsdeficit
        let q_hum = calculate_humidification_energy(50.0, 0.001, 100.0);

        // 50 * 0.001 * 2501 * 100 / 1000 = 12.5 MJ
        assert_abs_diff_eq!(q_hum, 12.5, epsilon = 0.1);
    }

    #[test]
    fn large_humidification_load() {
        // Grote luchtstroom, groot vochtigheidsdeficit
        let q_hum = calculate_humidification_energy(500.0, 0.005, 1000.0);

        // 500 * 0.005 * 2501 * 1000 / 1000 = 6252.5 MJ
        assert_abs_diff_eq!(q_hum, 6252.5, epsilon = 1.0);
    }
}