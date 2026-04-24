//! Verlichtings-berekening — orkestrator.
//!
//! Implementeert de V1 lumped vorm van NTA 8800 §14.2.2 formule (14.7):
//!
//! ```text
//! W_L;use;mi = P_n × F_u × F_d × F_c × A_f × t_mi × 3600 / 10^6   [MJ]
//! ```
//!
//! met `t_mi` de kalenderuren per maand (§17.2). Zie
//! [`monthly_use::monthly_w_l_use`] voor de atomaire formule.

pub mod monthly_use;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::zoning::Rekenzone;

use crate::errors::{LightingCalcResult, LightingError};
use crate::model::{EnergyCarrier, LightingSystem};
use crate::result::LightingResult;

/// Bereken het maandelijks en jaarlijks eindenergiegebruik voor verlichting
/// `W_L;use` voor één [`Rekenzone`] + [`LightingSystem`].
///
/// # Argumenten
///
/// - `zone` — [`Rekenzone`] met vloeroppervlakte `A_f` in m².
/// - `system` — [`LightingSystem`] met `P_n, F_u, F_d, F_c`.
///
/// # Returns
///
/// [`LightingResult`] met maandprofiel + jaartotaal in MJ en
/// [`EnergyCarrier::Electricity`] als energiedrager.
///
/// # Errors
///
/// - [`LightingError::InvalidInstalledPower`] als `P_n < 0` of niet-eindig.
/// - [`LightingError::InvalidFactor`] voor elke factor buiten `[0, 1]`.
/// - [`LightingError::InvalidFloorArea`] als `A_f < 0` of niet-eindig.
pub fn calculate_lighting(
    zone: &Rekenzone,
    system: &LightingSystem,
) -> LightingCalcResult<LightingResult> {
    // ---- Validatie ----
    system.validate()?;
    let area = zone.floor_area;
    if !area.is_finite() || area < 0.0 {
        return Err(LightingError::InvalidFloorArea { value: area });
    }

    // ---- Maandlus ----
    let mut out = [0.0_f64; 12];
    for month in Month::all() {
        out[month.index()] = monthly_use::monthly_w_l_use(system, area, month);
    }
    let monthly_w_l_use = MonthlyProfile::new(out);
    let annual_w_l_use: Energy = out.iter().sum();

    Ok(LightingResult {
        energy_carrier: EnergyCarrier::Electricity,
        monthly_w_l_use,
        annual_w_l_use,
        floor_area_m2: area,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nta8800_model::zoning::UsageFunction;

    fn zone(area: f64) -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Test".into(),
            gebouw_id: "g1".into(),
            floor_area: area,
            volume: area * 2.7,
            efr_ids: vec![],
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        }
    }

    #[test]
    fn kantoor_100m2_realistic() {
        // P_n = 10, F_u = 0.3, F_d = 0.7, F_c = 1.0, A = 100
        // Jaartotaal = 10 × 0.3 × 0.7 × 1.0 × 100 × 8760 × 3600 / 10^6
        //            = 2.1 × 100 × 8760 × 0.0036
        //            = 6 622.56 MJ
        let system = LightingSystem::new(10.0, 0.3, 0.7, 1.0).unwrap();
        let z = zone(100.0);
        let r = calculate_lighting(&z, &system).unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Electricity);
        assert_relative_eq!(r.annual_w_l_use, 6_622.56, epsilon = 1e-6);
        assert_relative_eq!(r.floor_area_m2, 100.0, epsilon = 1e-12);
    }

    #[test]
    fn f_c_zero_gives_zero_use() {
        // F_c = 0 simuleert volledig afgeschakelde verlichting.
        let system = LightingSystem::new(10.0, 0.3, 0.7, 0.0).unwrap();
        let r = calculate_lighting(&zone(100.0), &system).unwrap();
        assert_relative_eq!(r.annual_w_l_use, 0.0, epsilon = 1e-12);
        for m in Month::all() {
            assert_relative_eq!(r.monthly_w_l_use[m], 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn zero_power_gives_zero_use() {
        let system = LightingSystem::new(0.0, 0.3, 0.7, 1.0).unwrap();
        let r = calculate_lighting(&zone(200.0), &system).unwrap();
        assert_relative_eq!(r.annual_w_l_use, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_area_gives_zero_use() {
        let system = LightingSystem::new(10.0, 0.3, 0.7, 1.0).unwrap();
        let r = calculate_lighting(&zone(0.0), &system).unwrap();
        assert_relative_eq!(r.annual_w_l_use, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn annual_equals_sum_of_monthly() {
        let system = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let r = calculate_lighting(&zone(250.0), &system).unwrap();
        let sum: f64 = Month::all().iter().map(|m| r.monthly_w_l_use[*m]).sum();
        assert_relative_eq!(r.annual_w_l_use, sum, epsilon = 1e-9);
    }

    #[test]
    fn kantoor_forfaitair_100m2_jaartotaal() {
        // Forfaitair kantoor: P_n = 16, F_u = 2500/8760, F_d = 1, F_c = 1.
        // Jaartotaal = 16 × (2500/8760) × 100 × 8760 × 0.0036
        //            = 16 × 2500 × 100 × 0.0036 = 14_400 MJ ≙ 4000 kWh
        //            ≙ 40 kWh/m², klopt met tabel 14.3 + 14.1 norm-forfaitair.
        let system = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let r = calculate_lighting(&zone(100.0), &system).unwrap();
        assert_relative_eq!(r.annual_w_l_use, 14_400.0, epsilon = 1e-3);
        // Per m²: 14 400 MJ / 100 m² / 3.6 MJ/kWh = 40 kWh/m².
        let kwh_per_m2 = r.annual_w_l_use / 100.0 / 3.6;
        assert_relative_eq!(kwh_per_m2, 40.0, epsilon = 1e-3);
    }

    #[test]
    fn winkel_hogere_jaar_dan_kantoor() {
        // Winkel P_n = 30 + brandduren (2700, 400); kantoor 16 + (2200, 300).
        // Winkel-verbruik per m² moet significant hoger zijn.
        let winkel_sys = LightingSystem::forfaitair(UsageFunction::Winkelfunctie);
        let kantoor_sys = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let winkel = calculate_lighting(&zone(100.0), &winkel_sys).unwrap();
        let kantoor = calculate_lighting(&zone(100.0), &kantoor_sys).unwrap();
        assert!(winkel.annual_w_l_use > 2.0 * kantoor.annual_w_l_use);
    }

    #[test]
    fn maand_verdeling_op_uren_basis() {
        // Voor een constant W_L;use per uur moet januari (744 h) ≈ 1.033 ×
        // juli (744 h) → eigenlijk gelijk — beide 744 h.
        // Februari (672 h) moet kleiner zijn dan januari (744 h) met
        // verhouding 672/744.
        let system = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let r = calculate_lighting(&zone(100.0), &system).unwrap();
        let jan = r.monthly_w_l_use[Month::Januari];
        let feb = r.monthly_w_l_use[Month::Februari];
        let jul = r.monthly_w_l_use[Month::Juli];
        assert_relative_eq!(jan, jul, epsilon = 1e-9);
        assert_relative_eq!(feb / jan, 672.0 / 744.0, epsilon = 1e-6);
    }

    #[test]
    fn invalid_system_propagates() {
        // Constructie via new() weigert ongeldige input, maar een struct
        // met rauwe literal kan alsnog NaN bevatten — calc moet dat ook
        // afvangen via validate().
        let system = LightingSystem {
            installed_power_w_per_m2: f64::NAN,
            utilization_factor: 0.3,
            daylight_factor: 0.7,
            control_factor: 1.0,
        };
        let e = calculate_lighting(&zone(100.0), &system).unwrap_err();
        assert!(matches!(e, LightingError::InvalidInstalledPower { .. }));
    }

    #[test]
    fn invalid_area_propagates() {
        let system = LightingSystem::forfaitair(UsageFunction::Kantoorfunctie);
        let mut z = zone(100.0);
        z.floor_area = -10.0;
        let e = calculate_lighting(&z, &system).unwrap_err();
        assert!(matches!(e, LightingError::InvalidFloorArea { .. }));
    }

    #[test]
    fn serde_round_trip_lighting_result() {
        let system = LightingSystem::forfaitair(UsageFunction::Logiesfunctie);
        let r = calculate_lighting(&zone(150.0), &system).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: LightingResult = serde_json::from_str(&json).unwrap();
        for m in Month::all() {
            let a = r.monthly_w_l_use[m];
            let b = back.monthly_w_l_use[m];
            assert!((a - b).abs() <= 1e-9 * a.abs().max(1.0), "W_L;use {m:?}");
        }
        assert_relative_eq!(r.annual_w_l_use, back.annual_w_l_use, epsilon = 1e-9);
        assert_eq!(r.energy_carrier, back.energy_carrier);
    }
}
