//! Absolute vochtigheid berekeningen.

use crate::errors::HumidityError;
// Referenties gebruikt in doc-comments via NTA_8800_2025_* constanten

/// Bereken absolute vochtigheid uit temperatuur en relatieve vochtigheid.
///
/// Implementeert NTA 8800 formule (12.3) en (12.4) voor conversie van
/// relatieve naar absolute vochtigheid via verzadigingsdampdruk.
///
/// # Algoritme
///
/// 1. Bereken verzadigingsdampdruk `p_sat` via Magnus-Tetens formule (12.4)
/// 2. Bereken partiële dampdruk `p_v = φ · p_sat`
/// 3. Converteer naar absolute vochtigheid via formule (12.3)
///
/// # Arguments
///
/// * `temperature_c` - Luchttemperatuur in °C
/// * `relative_humidity_fraction` - Relatieve vochtigheid als fractie [0,1]
///
/// # Returns
///
/// Absolute vochtigheid in g/kg droge lucht
///
/// # Errors
///
/// - [`HumidityError::InvalidTemperatureRange`] bij temperatuur buiten [-40, 60] °C
/// - [`HumidityError::NegativeHumidity`] bij negatieve relatieve vochtigheid
/// - [`HumidityError::UnrealisticHumidity`] bij resultaat > 30 g/kg
///
/// # Referenties
///
/// - [`NTA_8800_2025_FORMULE12_3`]: absolute vochtigheid conversie
/// - [`NTA_8800_2025_FORMULE12_4`]: Magnus-Tetens verzadigingsdampdruk
pub fn calculate_absolute_humidity(
    temperature_c: f64,
    relative_humidity_fraction: f64,
) -> Result<f64, HumidityError> {
    // Input validatie
    if !(-40.0..=60.0).contains(&temperature_c) {
        return Err(HumidityError::InvalidTemperatureRange {
            temp: temperature_c,
        });
    }

    if relative_humidity_fraction < 0.0 {
        return Err(HumidityError::NegativeHumidity {
            value: relative_humidity_fraction * 1000.0, // Convert to g/kg voor error
        });
    }

    // Verzadigingsdampdruk via Magnus-Tetens (formule 12.4)
    // p_sat = 611.2 * exp((17.62 * T) / (T + 243.12)) in Pa
    let p_sat_pa = saturation_vapor_pressure(temperature_c);

    // Partiële dampdruk
    let p_v_pa = relative_humidity_fraction * p_sat_pa;

    // Absolute vochtigheid (formule 12.3)
    // x = 0.622 * p_v / (p_atm - p_v) in kg/kg
    let p_atm_pa = super::ATMOSPHERIC_PRESSURE_PA;
    let x_kg_per_kg = 0.622 * p_v_pa / (p_atm_pa - p_v_pa);

    // Converteer naar g/kg
    let absolute_humidity_g_per_kg = x_kg_per_kg * 1000.0;

    // Sanity check op resultaat
    if absolute_humidity_g_per_kg < 0.0 {
        return Err(HumidityError::NegativeHumidity { value: absolute_humidity_g_per_kg });
    }

    if absolute_humidity_g_per_kg > 30.0 {
        return Err(HumidityError::UnrealisticHumidity { value: absolute_humidity_g_per_kg });
    }

    Ok(absolute_humidity_g_per_kg)
}

/// Bereken verzadigingsdampdruk via Magnus-Tetens formule.
///
/// Implementeert NTA 8800 formule (12.4):
/// `p_sat = 611.2 · exp((17.62 · T) / (T + 243.12))` in Pa.
///
/// Geldig voor temperatuurrange -40°C tot +60°C.
fn saturation_vapor_pressure(temperature_c: f64) -> f64 {
    let t = temperature_c;
    611.2 * ((17.62 * t) / (t + 243.12)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn absolute_humidity_at_20c_50rh() {
        // 20°C, 50% RH → verwachte absolute vochtigheid ≈ 7.3 g/kg
        let x = calculate_absolute_humidity(20.0, 0.50).unwrap();
        assert_abs_diff_eq!(x, 7.3, epsilon = 0.5);
    }

    #[test]
    fn absolute_humidity_at_0c_80rh() {
        // 0°C, 80% RH → verwachte absolute vochtigheid ≈ 3.1 g/kg
        let x = calculate_absolute_humidity(0.0, 0.80).unwrap();
        assert_abs_diff_eq!(x, 3.1, epsilon = 0.5);
    }

    #[test]
    fn absolute_humidity_at_30c_70rh() {
        // 30°C, 70% RH → verwachte absolute vochtigheid ≈ 18.6 g/kg
        let x = calculate_absolute_humidity(30.0, 0.70).unwrap();
        assert_abs_diff_eq!(x, 18.6, epsilon = 1.0);
    }

    #[test]
    fn saturation_vapor_pressure_at_20c() {
        // 20°C → p_sat ≈ 2338 Pa (referentie uit psychrometrische tabellen)
        let p_sat = saturation_vapor_pressure(20.0);
        assert_abs_diff_eq!(p_sat, 2338.0, epsilon = 50.0);
    }

    #[test]
    fn saturation_vapor_pressure_at_0c() {
        // 0°C → p_sat ≈ 611 Pa (definitie van de formule)
        let p_sat = saturation_vapor_pressure(0.0);
        assert_abs_diff_eq!(p_sat, 611.0, epsilon = 5.0);
    }

    #[test]
    fn error_invalid_temperature_range() {
        assert_eq!(
            calculate_absolute_humidity(-50.0, 0.5),
            Err(HumidityError::InvalidTemperatureRange { temp: -50.0 })
        );
        assert_eq!(
            calculate_absolute_humidity(70.0, 0.5),
            Err(HumidityError::InvalidTemperatureRange { temp: 70.0 })
        );
    }

    #[test]
    fn error_negative_relative_humidity() {
        let result = calculate_absolute_humidity(20.0, -0.1);
        assert!(matches!(result, Err(HumidityError::NegativeHumidity { .. })));
    }

    #[test]
    fn boundary_values() {
        // 100% RH at 20°C should work
        let x = calculate_absolute_humidity(20.0, 1.0).unwrap();
        assert!(x > 10.0 && x < 20.0);

        // 0% RH should give 0 g/kg
        let x = calculate_absolute_humidity(20.0, 0.0).unwrap();
        assert_abs_diff_eq!(x, 0.0, epsilon = 0.001);
    }
}