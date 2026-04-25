//! Ontvochtigings-energie berekeningen.


/// Bereken ontvochtigingsbehoefte conform NTA 8800 formule (12.2).
///
/// `Q_dhum = ṁ_a · Δx · r_w` waarbij:
/// - `ṁ_a` = massastroom droge lucht [kg/h]
/// - `Δx` = vochtigheidsverschil (x_ODA - x_IDA) [kg/kg]
/// - `r_w` = verdampingswarmte water = 2501 kJ/kg
///
/// # Arguments
///
/// * `mass_flow_kg_h` - Massastroom droge lucht in kg/h
/// * `delta_x_kg_per_kg` - Vochtigheidsverschil in kg/kg (positief = ontvochtiging)
/// * `duration_hours` - Duur van de periode in uren
///
/// # Returns
///
/// Ontvochtigingsbehoefte in MJ thermisch
///
/// # Referenties
///
/// - [`NTA_8800_2025_FORMULE12_2`]: ontvochtigingsbehoefte berekening
/// - [`NTA_8800_2025_CONST_R_W_KJ_PER_KG`]: verdampingswarmte water
#[must_use]
pub fn calculate_dehumidification_energy(
    mass_flow_kg_h: f64,
    delta_x_kg_per_kg: f64,
    duration_hours: f64,
) -> f64 {
    if delta_x_kg_per_kg <= 0.0 {
        return 0.0; // Geen ontvochtiging nodig
    }

    // Q_dhum = ṁ_a · Δx · r_w · t [kJ]
    let q_dhum_kj = mass_flow_kg_h * delta_x_kg_per_kg * super::WATER_LATENT_HEAT_KJ_PER_KG * duration_hours;

    // Converteer kJ naar MJ
    q_dhum_kj / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn dehumidification_energy_calculation() {
        // 150 kg/h luchtstroom, 0.003 kg/kg vochtigheidsafname, 744 uur (zomermaand)
        let mass_flow = 150.0; // kg/h
        let delta_x = 0.003; // kg/kg (3 g/kg)
        let duration = 744.0; // h

        let q_dhum = calculate_dehumidification_energy(mass_flow, delta_x, duration);

        // Verwachte waarde: 150 * 0.003 * 2501 * 744 / 1000 = 837.1 MJ
        assert_abs_diff_eq!(q_dhum, 837.1, epsilon = 1.0);
    }

    #[test]
    fn no_dehumidification_needed() {
        // Negatief of nul vochtigheidssurplus → geen ontvochtiging
        assert_abs_diff_eq!(calculate_dehumidification_energy(100.0, 0.0, 744.0), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(calculate_dehumidification_energy(100.0, -0.001, 744.0), 0.0, epsilon = 1e-9);
    }

    #[test]
    fn small_dehumidification_load() {
        // Kleine luchtstroom, klein vochtigheidssurplus
        let q_dhum = calculate_dehumidification_energy(80.0, 0.002, 200.0);

        // 80 * 0.002 * 2501 * 200 / 1000 = 80.0 MJ
        assert_abs_diff_eq!(q_dhum, 80.0, epsilon = 0.1);
    }

    #[test]
    fn large_dehumidification_load() {
        // Grote luchtstroom, groot vochtigheidssurplus (vochtige zomer)
        let q_dhum = calculate_dehumidification_energy(400.0, 0.008, 1000.0);

        // 400 * 0.008 * 2501 * 1000 / 1000 = 8003.2 MJ
        assert_abs_diff_eq!(q_dhum, 8003.2, epsilon = 1.0);
    }
}