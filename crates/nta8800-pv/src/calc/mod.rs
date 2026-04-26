//! PV-opbrengst berekening conform NTA 8800:2025+C1:2026 H.16.
//!
//! Hoofdfunctie [`calculate_pv_yield`] implementeert formule 16.101:
//! `Q_PV;mi = P_peak * I_sol;mi * η_total * t_maand / 1000` [MJ]
//!
//! ## V1 Scope & Vereenvoudigingen
//!
//! - **Tilt/azimuth correctie:** forfaitair via cosinus-benadering
//!   `f_tilt_az = cos(β - β_opt) * cos((γ - γ_opt)/2)` met β_opt=35°, γ_opt=180°
//!   (V2: volledige interpolatie van NTA 8800 tabel 16.1/16.2)
//! - **Zoninstraling:** gebruikt horizontale I_sol direct (V1 vereenvoudiging)
//! - **Inverter-efficiëntie:** constant η_inv (V2: dynamisch η(P_load/P_rated))
//! - **Temperatuur-correctie:** niet geïmplementeerd (V2: T_cel vs T_amb)

use nta8800_model::{
    climate::ClimateData,
    location::Orientation,
    time::{Month, MonthlyProfile},
};
use nta8800_tables::climate::de_bilt::DE_BILT_MONTH_LENGTHS_HOURS;

use crate::{
    errors::PvError,
    model::{PvLocation, PvSystem},
    result::PvResult,
};

/// Bereken maandelijkse en jaarlijkse PV-opbrengst conform NTA 8800 H.16.
///
/// Implementeert formule 16.101: `Q_PV;mi = P_peak * I_sol;mi * η_total * t_maand / 1000`
/// waarbij η_total de gecombineerde systeem- × inverter- × schaduw-efficiëntie is.
///
/// # Argumenten
///
/// * `systems` - Array van [`PvSystem`] configuraties (mag niet leeg zijn)
/// * `location` - [`PvLocation`] voor geografische correcties (momenteel V1 stub)
/// * `climate` - [`ClimateData`] met maandelijkse zoninstraling-profielen
///
/// # Returns
///
/// [`PvResult`] met maandprofiel + jaartotaal Q_PV;mi en breakdown van verliezen.
///
/// # Errors
///
/// * [`PvError::EmptySystemList`] - als `systems.is_empty()`
/// * [`PvError::NegativeSolarIrradiation`] - als irradiation < 0 in climate data
/// * [`PvError::MissingClimateData`] - als horizontale zoninstraling ontbreekt
///
/// # V1 Beperkingen
///
/// - Gebruikt horizontale zoninstraling rechtstreeks (geen tilt-correctie)
/// - Forfaitaire azimuth-correctie via cosinus-benadering voor β_opt=35°, γ_opt=180°
/// - Constante inverter-efficiëntie (geen load-afhankelijke curve)
///
/// # Example
///
/// ```
/// use nta8800_pv::{calculate_pv_yield, PvSystem, PvLocation};
/// use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
///
/// let systems = vec![PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96)?];
/// let location = PvLocation::new(52.1, 5.2)?;
/// let climate = de_bilt_climate_data();
/// let result = calculate_pv_yield(&systems, &location, &climate)?;
///
/// assert!(result.annual_yield_mj > 15000.0);  // ~5.5 kWp → ~19800 MJ/jaar verwacht
/// # Ok::<(), nta8800_pv::PvError>(())
/// ```
pub fn calculate_pv_yield(
    systems: &[PvSystem],
    _location: &PvLocation,
    climate: &ClimateData,
) -> Result<PvResult, PvError> {
    // Validatie: systemen mogen niet leeg zijn
    if systems.is_empty() {
        return Err(PvError::EmptySystemList);
    }

    // Haal horizontale zoninstraling op (V1: gebruik deze rechtstreeks)
    let horizontal_irradiation = climate
        .solar_irradiation
        .get(&Orientation::Horizontaal)
        .ok_or(PvError::MissingClimateData { month: 0 })?;

    // Validatie: zoninstraling mag niet negatief zijn
    for month in Month::all() {
        let irrad = horizontal_irradiation[month];
        if irrad < 0.0 {
            return Err(PvError::NegativeSolarIrradiation(irrad));
        }
    }

    // Arrays voor resultaten en verliezen
    let mut monthly_yield = [0.0_f64; 12];
    let mut inverter_losses = [0.0_f64; 12];
    let mut system_losses = [0.0_f64; 12];

    // Bereken opbrengst per maand
    for month in Month::all() {
        let month_idx = month.index();
        let irradiation_mj_per_m2 = horizontal_irradiation[month];
        let month_hours = DE_BILT_MONTH_LENGTHS_HOURS[month];

        // Som over alle systemen
        for system in systems {
            // Tilt/azimuth correctiefactor (V1 vereenvoudiging)
            let tilt_az_factor =
                calculate_tilt_azimuth_factor(system.tilt_degrees, system.azimuth_degrees);

            // Gecorrigeerde zoninstraling voor dit systeem
            let corrected_irradiation = irradiation_mj_per_m2 * tilt_az_factor;

            // Formule NTA 8800 H.16: Q [MJ/maand] = P_peak[kWp] × I_avg[W/m²] × t_maand[h] × 0.0036
            // Conversie van W/m² gemiddeld naar MJ/maand via maanduren
            let gross_yield_wh = system.peak_power_kwp * corrected_irradiation * month_hours;
            let gross_yield = gross_yield_wh * 0.0036; // Wh → MJ

            // Pas efficiënties toe stapsgewijs (voor verlies-tracking)
            let after_system_efficiency =
                gross_yield * system.system_efficiency * system.shadow_factor;
            let final_yield = after_system_efficiency * system.inverter_efficiency;

            // Bereken verliezen
            let system_loss = gross_yield - after_system_efficiency;
            let inverter_loss = after_system_efficiency - final_yield;

            // Accumuleer in maand-arrays
            monthly_yield[month_idx] += final_yield;
            system_losses[month_idx] += system_loss;
            inverter_losses[month_idx] += inverter_loss;
        }
    }

    // Bereken jaartotaal
    let annual_yield: f64 = monthly_yield.iter().sum();

    Ok(PvResult {
        monthly_yield_mj: MonthlyProfile::new(monthly_yield),
        annual_yield_mj: annual_yield,
        inverter_losses_mj: MonthlyProfile::new(inverter_losses),
        system_losses_mj: MonthlyProfile::new(system_losses),
    })
}

/// V1 forfaitaire tilt/azimuth correctiefactor.
///
/// Implementeert cosinus-benadering: `f = cos(β - β_opt) * cos((γ - γ_opt)/2)`
/// met β_opt = 35° en γ_opt = 180° (optimaal voor Nederland).
///
/// V2 zal volledige interpolatie uit NTA 8800 tabel 16.1/16.2 implementeren.
fn calculate_tilt_azimuth_factor(tilt_degrees: f64, azimuth_degrees: f64) -> f64 {
    let beta_opt = 35.0; // Optimale hellingshoek voor Nederland
    let gamma_opt = 180.0; // Zuid-oriëntatie optimaal

    let tilt_factor = (tilt_degrees - beta_opt).to_radians().cos();
    let azimuth_factor = ((azimuth_degrees - gamma_opt) / 2.0).to_radians().cos();

    (tilt_factor * azimuth_factor).max(0.0) // Negatieve factoren op 0 zetten
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PvLocation;
    use approx::assert_abs_diff_eq;
    use std::collections::BTreeMap;

    /// Helper voor test klimaat met bekende zoninstraling
    fn test_climate_data() -> ClimateData {
        let mut solar_map = BTreeMap::new();
        // 12 maanden met realistische horizontale zoninstraling (MJ/m²)
        solar_map.insert(
            Orientation::Horizontaal,
            MonthlyProfile::new([
                50.0, 80.0, 140.0, 200.0, 250.0, 270.0, // Jan-Jun
                260.0, 230.0, 170.0, 110.0, 60.0, 40.0, // Jul-Dec
            ]),
        );

        ClimateData {
            outdoor_temperature: MonthlyProfile::new([3.0; 12]),
            solar_irradiation: solar_map,
            cooling_reference_temperature: MonthlyProfile::new([None; 12]),
            wind_speed: MonthlyProfile::new([3.0; 12]),
            wtw_preheat_temperature: MonthlyProfile::new([0.0; 12]),
        }
    }

    #[test]
    fn happy_path_single_system() {
        // 5.5 kWp zuid/35° systeem, verwacht ~5500 kWh/jaar ≈ 19800 MJ
        let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result = calculate_pv_yield(&[system], &location, &climate).unwrap();

        // Sanity checks
        assert!(result.annual_yield_mj > 15000.0); // Minimum verwacht
        assert!(result.annual_yield_mj < 25000.0); // Maximum realistisch

        // Check dat zomer > winter
        use nta8800_model::time::Month;
        assert!(result.monthly_yield_mj[Month::Juni] > result.monthly_yield_mj[Month::December]);

        // Check dat annual = som van monthly
        let sum_monthly: f64 = result.monthly_yield_mj.as_array().iter().sum();
        assert_abs_diff_eq!(result.annual_yield_mj, sum_monthly, epsilon = 1e-6);
    }

    #[test]
    fn multiple_systems_sum_correctly() {
        let system1 = PvSystem::new(3.0, 35.0, 180.0, 0.85, 0.96).unwrap();
        let system2 = PvSystem::new(2.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result_combined =
            calculate_pv_yield(&[system1.clone(), system2.clone()], &location, &climate).unwrap();
        let result1 = calculate_pv_yield(&[system1], &location, &climate).unwrap();
        let result2 = calculate_pv_yield(&[system2], &location, &climate).unwrap();

        // Som moet kloppen
        let expected_annual = result1.annual_yield_mj + result2.annual_yield_mj;
        assert_abs_diff_eq!(
            result_combined.annual_yield_mj,
            expected_annual,
            epsilon = 1e-6
        );
    }

    #[test]
    fn empty_system_list_returns_error() {
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result = calculate_pv_yield(&[], &location, &climate);
        assert!(matches!(result, Err(PvError::EmptySystemList)));
    }

    #[test]
    fn negative_solar_irradiation_returns_error() {
        let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let location = PvLocation::new(52.1, 5.2).unwrap();

        // Maak klimaat met negatieve zoninstraling
        let mut solar_map = BTreeMap::new();
        solar_map.insert(
            Orientation::Horizontaal,
            MonthlyProfile::new([
                50.0, -10.0, 140.0, 200.0, 250.0, 270.0, // Februari negatief
                260.0, 230.0, 170.0, 110.0, 60.0, 40.0,
            ]),
        );
        let bad_climate = ClimateData {
            outdoor_temperature: MonthlyProfile::new([3.0; 12]),
            solar_irradiation: solar_map,
            cooling_reference_temperature: MonthlyProfile::new([None; 12]),
            wind_speed: MonthlyProfile::new([3.0; 12]),
            wtw_preheat_temperature: MonthlyProfile::new([0.0; 12]),
        };

        let result = calculate_pv_yield(&[system], &location, &bad_climate);
        assert!(matches!(result, Err(PvError::NegativeSolarIrradiation(_))));
    }

    #[test]
    fn tilt_effect_vertical_less_than_optimal() {
        let optimal = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap(); // 35° optimaal
        let vertical = PvSystem::new(5.5, 90.0, 180.0, 0.85, 0.96).unwrap(); // 90° verticaal
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result_opt = calculate_pv_yield(&[optimal], &location, &climate).unwrap();
        let result_vert = calculate_pv_yield(&[vertical], &location, &climate).unwrap();

        // Verticaal moet minder opbrengst geven dan optimaal
        assert!(result_vert.annual_yield_mj < result_opt.annual_yield_mj);
    }

    #[test]
    fn azimuth_effect_south_better_than_east() {
        let south = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap(); // Zuid
        let east = PvSystem::new(5.5, 35.0, 90.0, 0.85, 0.96).unwrap(); // Oost
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result_south = calculate_pv_yield(&[south], &location, &climate).unwrap();
        let result_east = calculate_pv_yield(&[east], &location, &climate).unwrap();

        // Zuid moet beter zijn dan oost
        assert!(result_south.annual_yield_mj > result_east.annual_yield_mj);
    }

    #[test]
    fn shadow_factor_reduces_yield() {
        let no_shadow = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let with_shadow = PvSystem::with_shadow(5.5, 35.0, 180.0, 0.85, 0.96, 0.5).unwrap(); // 50% schaduw
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result_no_shadow = calculate_pv_yield(&[no_shadow], &location, &climate).unwrap();
        let result_with_shadow = calculate_pv_yield(&[with_shadow], &location, &climate).unwrap();

        // Met schaduw moet minder opbrengst geven
        assert!(result_with_shadow.annual_yield_mj < result_no_shadow.annual_yield_mj);

        // Ongeveer de helft (niet exact door tilt/azimuth factor)
        let ratio = result_with_shadow.annual_yield_mj / result_no_shadow.annual_yield_mj;
        assert!(ratio > 0.4 && ratio < 0.6);
    }

    #[test]
    fn extreme_tilt_angles_valid() {
        let horizontal = PvSystem::new(5.5, 0.0, 180.0, 0.85, 0.96).unwrap(); // 0° horizontaal
        let vertical = PvSystem::new(5.5, 90.0, 180.0, 0.85, 0.96).unwrap(); // 90° verticaal
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        // Beide moeten werken
        assert!(calculate_pv_yield(&[horizontal], &location, &climate).is_ok());
        assert!(calculate_pv_yield(&[vertical], &location, &climate).is_ok());
    }

    #[test]
    fn annual_equals_sum_of_monthly() {
        let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let climate = test_climate_data();

        let result = calculate_pv_yield(&[system], &location, &climate).unwrap();

        let sum_monthly: f64 = result.monthly_yield_mj.as_array().iter().sum();
        assert_abs_diff_eq!(result.annual_yield_mj, sum_monthly, epsilon = 1e-9);
    }

    #[test]
    fn missing_horizontal_irradiation_returns_error() {
        let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        let location = PvLocation::new(52.1, 5.2).unwrap();

        // Klimaat zonder horizontale zoninstraling
        let mut solar_map = BTreeMap::new();
        solar_map.insert(Orientation::Zuid, MonthlyProfile::new([50.0; 12])); // Alleen zuid

        let incomplete_climate = ClimateData {
            outdoor_temperature: MonthlyProfile::new([3.0; 12]),
            solar_irradiation: solar_map,
            cooling_reference_temperature: MonthlyProfile::new([None; 12]),
            wind_speed: MonthlyProfile::new([3.0; 12]),
            wtw_preheat_temperature: MonthlyProfile::new([0.0; 12]),
        };

        let result = calculate_pv_yield(&[system], &location, &incomplete_climate);
        assert!(matches!(
            result,
            Err(PvError::MissingClimateData { month: _ })
        ));
    }
}
