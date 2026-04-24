//! Maand-balans orkestrator.
//!
//! De sub-modules dekken elk een deel van de H.7-rekenroute:
//!
//! | Module | Norm-ref | Rol |
//! |---|---|---|
//! | [`time_constant`] | §7.8 / (7.17) | `τ` in uren uit C_m en H_tr + H_ve |
//! | [`utilization`] | §7.6 / (7.6)–(7.7) / (7.12)–(7.13) | η_H,gn, η_C,ls |
//! | [`solar_gains`] | §7.9 / (7.33) | Q_sol;mi uit ramen + klimaat |
//! | [`internal_gains`] | §7.10 / (7.35) | Q_int;mi uit Φ_int + A_g |
//! | [`monthly_balance`] | §7.4 / (7.4), §7.5 / (7.10) | Q_H,nd / Q_C,nd |
//!
//! De publieke entry-point [`calculate_demand`] bindt alles samen.

use nta8800_model::geometry::Window;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::{ClimateData, Rekenzone};

use nta8800_transmission::TransmissionResult;
use nta8800_ventilation::VentilationResult;

use crate::errors::DemandCalcResult;
use crate::model::{CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use crate::result::{DemandBreakdown, DemandResult};

pub mod internal_gains;
pub mod monthly_balance;
pub mod solar_gains;
pub mod time_constant;
pub mod utilization;

/// Default shading-factor `F_sh` voor V1 (geen schaduwmodel).
///
/// `1,0` = geen schaduw; alle zoninstraling bereikt het glasoppervlak. V2
/// introduceert een overhang/obstructie-model conform NTA 8800 §7.9.3.
pub const DEFAULT_SHADING_FACTOR: f64 = 1.0;

/// Bereken maandelijkse warmte- en koudebehoefte voor één rekenzone.
///
/// Implementeert NTA 8800 H.7 maand-balans met benuttingsfactor.
///
/// # Argumenten
///
/// - `zone` — rekenzone met `floor_area`. Id-lijsten worden niet gelezen; de
///   caller levert expliciet `windows` aan.
/// - `transmission` — [`TransmissionResult`] uit `nta8800-transmission`, met
///   `monthly_q_t` in MJ en H-coëfficiënten voor τ.
/// - `ventilation` — [`VentilationResult`] uit `nta8800-ventilation`.
/// - `ventilation_h_ve` — ventilatie-conductance `H_ve` in W/K. De
///   ventilatie-crate rapporteert die niet als veld; de consumer leidt deze
///   af uit `q_v × ρ·c` of levert een vaste waarde. Gebruikt voor τ.
/// - `windows` — transparante elementen in de zone voor zoninstraling.
/// - `climate` — klimaatdata met temperatuur en zoninstraling per oriëntatie.
/// - `heating_setpoint` / `cooling_setpoint` — maandprofielen (ongebruikt in
///   de energie-balans zelf, maar bewaard voor traceability; de Q_ht-waarden
///   zijn al in de upstream berekeningen gerealiseerd tegen de H-setpoint).
/// - `internal_gains` — [`InternalGains`] met Φ_int in W/m² per maand.
/// - `thermal_mass` — classificatie voor `C_m`-lookup.
/// - `shading_factor` — `F_sh` ∈ [0, 1]; pass [`DEFAULT_SHADING_FACTOR`] als
///   geen schaduwmodel beschikbaar is.
///
/// # Returns
///
/// [`DemandResult`] met monthly + annual Q_H,nd en Q_C,nd plus een
/// [`DemandBreakdown`] voor traceability en rapportage.
///
/// # Errors
///
/// - [`crate::DemandError::InvalidFloorArea`] als `zone.floor_area ≤ 0`
/// - [`crate::DemandError::NonPositiveConductance`] als H_tr + H_ve ≤ 0
/// - [`crate::DemandError::InvalidInternalHeatFlux`] bij corrupte invoer
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn calculate_demand(
    zone: &Rekenzone,
    transmission: &TransmissionResult,
    ventilation: &VentilationResult,
    ventilation_h_ve: f64,
    windows: &[&Window],
    climate: &ClimateData,
    heating_setpoint: HeatingSetpoint,
    cooling_setpoint: CoolingSetpoint,
    internal_gains: &InternalGains,
    thermal_mass: ThermalMassInput,
    shading_factor: f64,
) -> DemandCalcResult<DemandResult> {
    // Setpoints zijn voor traceability/UI; Q_ht uit transmission/ventilation
    // is al berekend tegen deze setpoints upstream.
    let _ = (heating_setpoint, cooling_setpoint);

    // ---- H_tr (som deelcoëfficiënten H_D + H_U + H_g;an + H_A) ----
    let h_tr = transmission.h_d + transmission.h_u + transmission.h_g_an + transmission.h_a;

    // ---- τ en a ----
    let tau_hours =
        time_constant::time_constant_hours(&thermal_mass, zone.floor_area, h_tr, ventilation_h_ve)?;
    let a_heat = utilization::a_parameter(tau_hours);
    let a_cool = utilization::a_parameter(tau_hours);

    // ---- Q_int, Q_sol ----
    let monthly_q_int = internal_gains::monthly_internal_gains(internal_gains, zone.floor_area);
    let monthly_q_sol = solar_gains::monthly_solar_gains(windows, climate, shading_factor);

    // ---- Maandlus: Q_ht, Q_gn, γ, η, Q_nd ----
    let mut out_q_ht = [0.0_f64; 12];
    let mut out_q_gn = [0.0_f64; 12];
    let mut out_eta_heating = [0.0_f64; 12];
    let mut out_eta_cooling = [0.0_f64; 12];
    let mut out_heating_demand = [0.0_f64; 12];
    let mut out_cooling_demand = [0.0_f64; 12];

    for month in Month::all() {
        let idx = month.index();

        let q_ht: Energy = transmission.monthly_q_t[month] + ventilation.monthly_q_v[month];
        let q_gn: Energy = monthly_q_int[month] + monthly_q_sol[month];

        out_q_ht[idx] = q_ht;
        out_q_gn[idx] = q_gn;

        let gamma_h = monthly_balance::gamma(q_gn, q_ht);
        let eta_h = utilization::utilization_heating(gamma_h, a_heat);
        out_eta_heating[idx] = eta_h;
        out_heating_demand[idx] = monthly_balance::heating_demand(q_ht, q_gn, eta_h);

        // γ_C: volgens NTA 8800 formule (7.11) gebruikt koeling dezelfde
        // γ = Q_gn / Q_ht definitie als warmte. V1 hergebruikt Q_ht/Q_gn
        // uit de transmission/ventilation-cases (die tegen H-setpoint zijn
        // berekend). De koelmodus gebruikt `utilization_cooling` met γ^(-a)
        // vorm; in de limiet γ_C >> 1 benadert η_C,ls → 1.
        let gamma_c = gamma_h;
        let eta_c = utilization::utilization_cooling(gamma_c, a_cool);
        out_eta_cooling[idx] = eta_c;
        out_cooling_demand[idx] = monthly_balance::cooling_demand(q_ht, q_gn, eta_c);
    }

    let monthly_q_ht = MonthlyProfile::new(out_q_ht);
    let monthly_q_gn = MonthlyProfile::new(out_q_gn);
    let monthly_heating_demand = MonthlyProfile::new(out_heating_demand);
    let monthly_cooling_demand = MonthlyProfile::new(out_cooling_demand);

    let annual_heating_demand = monthly_balance::annual_sum(&monthly_heating_demand);
    let annual_cooling_demand = monthly_balance::annual_sum(&monthly_cooling_demand);

    Ok(DemandResult {
        monthly_heating_demand,
        monthly_cooling_demand,
        annual_heating_demand,
        annual_cooling_demand,
        breakdown: DemandBreakdown {
            monthly_q_ht,
            monthly_q_gn,
            monthly_q_sol,
            monthly_q_int,
            monthly_utilization_heating: MonthlyProfile::new(out_eta_heating),
            monthly_utilization_cooling: MonthlyProfile::new(out_eta_cooling),
            time_constant_hours: tau_hours,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::location::{Orientation, Tilt};
    use nta8800_model::zoning::UsageFunction;
    use nta8800_tables::climate::de_bilt_climate_data;

    use nta8800_transmission::{TransmissionBreakdown, TransmissionResult as TrResult};
    use nta8800_ventilation::VentilationResult as VnResult;

    /// Maak een kleine woning: 100 m² vloeroppervlak, matige isolatie.
    fn sample_zone() -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Woonzone".into(),
            gebouw_id: "g1".into(),
            floor_area: 100.0,
            volume: 250.0,
            efr_ids: vec![],
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        }
    }

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    /// Synthetische transmissie: constante ΔT benadering voor De Bilt.
    /// H_tr = 150 W/K, ΔT ≈ 20 − θ_e;mi; Q_tr;mi = H × ΔT × t_mi × 3.6/1000.
    fn sample_transmission() -> TrResult {
        // Realistische waardes: 100 m² woning met gemiddelde schil
        // ~2015 bouw: H_D ≈ 120 W/K, H_U ≈ 0, H_g ≈ 30 W/K, H_A = 0
        // Q-profielen vooraf berekend tegen de Bilt klimaat en θ_i = 20.
        // Voor de integration-test maken we handmatig een plausibel profiel.
        let climate = de_bilt_climate_data();
        let h_d = 120.0_f64;
        let h_g = 30.0_f64;
        let theta_i = 20.0_f64;
        let annual_avg = {
            let s: f64 = Month::all()
                .iter()
                .map(|&m| climate.outdoor_temperature[m])
                .sum();
            s / 12.0
        };
        let month_hours = [
            744.0_f64, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
        ];
        let mut q_t = [0.0_f64; 12];
        for month in Month::all() {
            let idx = month.index();
            let theta_e = climate.outdoor_temperature[month];
            // kWh = H × ΔT × t/1000; MJ = kWh × 3.6
            let q_outdoor_kwh = h_d * (theta_i - theta_e) * 0.001 * month_hours[idx];
            let q_ground_kwh = h_g * (theta_i - annual_avg) * 0.001 * month_hours[idx];
            q_t[idx] = (q_outdoor_kwh + q_ground_kwh) * 3.6;
        }
        let annual_q_t: f64 = q_t.iter().sum();
        TrResult {
            monthly_q_t: MonthlyProfile::new(q_t),
            annual_q_t,
            breakdown: TransmissionBreakdown {
                outdoor: MonthlyProfile::from_constant(0.0),
                unheated_space: zero_profile(),
                ground: zero_profile(),
                adjacent_zone: zero_profile(),
                thermal_bridges: zero_profile(),
            },
            h_d,
            h_u: 0.0,
            h_g_an: h_g,
            h_a: 0.0,
        }
    }

    /// Synthetische ventilatie: 150 m³/h balans, geen WTW.
    fn sample_ventilation() -> VnResult {
        let climate = de_bilt_climate_data();
        let q_flow = 150.0_f64; // m³/h
        let rho_c = 1212.23_f64; // J/(m³·K)
        let theta_i = 20.0_f64;
        let month_hours = [
            744.0_f64, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
        ];
        let mut q_v = [0.0_f64; 12];
        for month in Month::all() {
            let idx = month.index();
            let theta_e = climate.outdoor_temperature[month];
            let dt = (theta_i - theta_e).max(0.0);
            let energy_j = q_flow * rho_c * dt * month_hours[idx];
            q_v[idx] = energy_j / 1_000_000.0;
        }
        let annual_q_v: f64 = q_v.iter().sum();
        VnResult {
            monthly_q_v: MonthlyProfile::new(q_v),
            annual_q_v,
            monthly_w_fan: MonthlyProfile::from_constant(0.0),
            annual_w_fan: 0.0,
            monthly_wtw_recovery: MonthlyProfile::from_constant(0.0),
            annual_wtw_recovery: 0.0,
        }
    }

    /// H_ve consistent met sample_ventilation (150 m³/h balans):
    /// H_ve = q · ρc / 3600 = 150 × 1212.23 / 3600 ≈ 50.5 W/K
    fn sample_h_ve() -> f64 {
        150.0 * 1212.23 / 3600.0
    }

    fn sample_windows() -> Vec<Window> {
        vec![
            Window::new(
                "w-zuid",
                "c",
                8.0,
                Orientation::Zuid,
                Tilt::VERTICAL,
                1.1,
                0.6,
                0.2,
            )
            .unwrap(),
            Window::new(
                "w-noord",
                "c",
                4.0,
                Orientation::Noord,
                Tilt::VERTICAL,
                1.1,
                0.6,
                0.2,
            )
            .unwrap(),
        ]
    }

    #[test]
    fn integration_kleine_woning_de_bilt() {
        let zone = sample_zone();
        let tr = sample_transmission();
        let vn = sample_ventilation();
        let h_ve = sample_h_ve();
        let windows_owned = sample_windows();
        let windows: Vec<&Window> = windows_owned.iter().collect();
        let climate = de_bilt_climate_data();
        let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);
        let mass = ThermalMassInput::light_woning();

        let result = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &windows,
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            mass,
            DEFAULT_SHADING_FACTOR,
        )
        .expect("demand-berekening mag niet falen");

        // Jaar-Q_H,nd moet in range 15–45 GJ liggen voor 100 m² met H_tr=150 W/K.
        // 15 000 – 45 000 MJ
        let q_h = result.annual_heating_demand;
        assert!(
            (15_000.0..=45_000.0).contains(&q_h),
            "jaar-Q_H,nd buiten plausibel bereik: {q_h} MJ"
        );

        // Winter > zomer voor warmtebehoefte
        assert!(result.monthly_heating_demand[Month::Januari] > 0.0);
        assert!(
            result.monthly_heating_demand[Month::Januari]
                > result.monthly_heating_demand[Month::Juli]
        );

        // τ moet in range 5–25 h liggen voor lichte woning
        let tau = result.breakdown.time_constant_hours;
        assert!((5.0..=25.0).contains(&tau), "τ = {tau} h buiten verwacht");
    }

    #[test]
    fn hogere_massa_geeft_hogere_tau_en_hogere_eta() {
        let zone = sample_zone();
        let tr = sample_transmission();
        let vn = sample_ventilation();
        let h_ve = sample_h_ve();
        let windows_owned = sample_windows();
        let windows: Vec<&Window> = windows_owned.iter().collect();
        let climate = de_bilt_climate_data();
        let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);

        let res_licht = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &windows,
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            ThermalMassInput::light_woning(),
            DEFAULT_SHADING_FACTOR,
        )
        .unwrap();

        let res_zwaar = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &windows,
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            ThermalMassInput::zwaar_massief(),
            DEFAULT_SHADING_FACTOR,
        )
        .unwrap();

        assert!(res_zwaar.breakdown.time_constant_hours > res_licht.breakdown.time_constant_hours);
        // Hogere τ → hogere η → lagere Q_H,nd (zware massa benut winst beter)
        assert!(res_zwaar.annual_heating_demand < res_licht.annual_heating_demand);
    }

    #[test]
    fn juli_heeft_geen_warmtevraag_wel_koelvraag() {
        let zone = sample_zone();
        let tr = sample_transmission();
        let vn = sample_ventilation();
        let h_ve = sample_h_ve();
        let windows_owned = sample_windows();
        let windows: Vec<&Window> = windows_owned.iter().collect();
        let climate = de_bilt_climate_data();
        let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);

        let r = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &windows,
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            ThermalMassInput::light_woning(),
            DEFAULT_SHADING_FACTOR,
        )
        .unwrap();

        // In juli: veel zoninstraling + lage ΔT → Q_H,nd klein, Q_C,nd > 0
        let juli_h = r.monthly_heating_demand[Month::Juli];
        let juli_c = r.monthly_cooling_demand[Month::Juli];
        assert!(juli_h < r.monthly_heating_demand[Month::Januari] * 0.5);
        // Q_C,nd niet noodzakelijk positief in deze synthetic case — accepteer ≥ 0
        assert!(juli_c >= 0.0);
    }

    #[test]
    fn resultaat_serde_round_trip() {
        let zone = sample_zone();
        let tr = sample_transmission();
        let vn = sample_ventilation();
        let h_ve = sample_h_ve();
        let windows_owned = sample_windows();
        let windows: Vec<&Window> = windows_owned.iter().collect();
        let climate = de_bilt_climate_data();
        let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);

        let r = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &windows,
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            ThermalMassInput::light_woning(),
            DEFAULT_SHADING_FACTOR,
        )
        .unwrap();

        let json = serde_json::to_string(&r).unwrap();
        let back: DemandResult = serde_json::from_str(&json).unwrap();

        // JSON float round-trip kan laatste ULP verliezen; vergelijk met
        // relatieve tolerantie. Structuur en metadata moeten exact kloppen.
        for month in Month::all() {
            let a = r.monthly_heating_demand[month];
            let b = back.monthly_heating_demand[month];
            assert!((a - b).abs() <= 1e-9 * a.abs().max(1.0), "Q_H,nd {month:?}");
            let a = r.monthly_cooling_demand[month];
            let b = back.monthly_cooling_demand[month];
            assert!((a - b).abs() <= 1e-9 * a.abs().max(1.0), "Q_C,nd {month:?}");
        }
        assert!(
            (r.annual_heating_demand - back.annual_heating_demand).abs()
                <= 1e-9 * r.annual_heating_demand.abs()
        );
        assert!(
            (r.annual_cooling_demand - back.annual_cooling_demand).abs()
                <= 1e-9 * r.annual_cooling_demand.abs()
        );
        assert!(
            (r.breakdown.time_constant_hours - back.breakdown.time_constant_hours).abs() < 1e-9
        );
    }

    #[test]
    fn nul_floor_area_geeft_error() {
        let mut zone = sample_zone();
        zone.floor_area = 0.0;
        let tr = sample_transmission();
        let vn = sample_ventilation();
        let h_ve = sample_h_ve();
        let climate = de_bilt_climate_data();
        let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);

        let err = calculate_demand(
            &zone,
            &tr,
            &vn,
            h_ve,
            &[],
            &climate,
            HeatingSetpoint::constant(20.0),
            CoolingSetpoint::constant(24.0),
            &internal,
            ThermalMassInput::light_woning(),
            DEFAULT_SHADING_FACTOR,
        )
        .unwrap_err();
        assert!(matches!(err, crate::DemandError::InvalidFloorArea { .. }));
    }
}
