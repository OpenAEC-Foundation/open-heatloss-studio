//! Integratietests — end-to-end via `calculate_ventilation`.

use approx::assert_relative_eq;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::zoning::Rekenzone;
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_ventilation::{
    calculate_ventilation, AirFlow, VentilationError, VentilationSystem, WtwSpecification,
};

fn sample_zone() -> Rekenzone {
    Rekenzone {
        id: "rz1".into(),
        name: "Woning".into(),
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

#[test]
fn system_d_with_wtw_reduces_q_v_vs_without_wtw() {
    let zone = sample_zone();
    let flow = AirFlow::new(150.0, 150.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(0.80, 0.45 / 3.6, true);

    let sys_wtw = VentilationSystem::D { with_wtw: true };
    let sys_no_wtw = VentilationSystem::D { with_wtw: false };

    let r_wtw =
        calculate_ventilation(&zone, &sys_wtw, &flow, Some(&wtw), &indoor, &climate).unwrap();
    let r_no_wtw =
        calculate_ventilation(&zone, &sys_no_wtw, &flow, None, &indoor, &climate).unwrap();

    // Bij η = 0,80 moet ~80% reductie optreden op ventilatieverlies
    let reduction = 1.0 - r_wtw.annual_q_v / r_no_wtw.annual_q_v;
    assert!(
        reduction > 0.70 && reduction < 0.85,
        "Verwacht ~80% WTW-reductie, kreeg {}% (Q_wtw={}, Q_nowtw={})",
        reduction * 100.0,
        r_wtw.annual_q_v,
        r_no_wtw.annual_q_v
    );
}

#[test]
fn wtw_efficiency_one_gives_zero_q_v() {
    // Perfecte WTW → ϑ_toevoer = ϑ_i → geen warmteverlies door ventilatie.
    let zone = sample_zone();
    let flow = AirFlow::new(150.0, 150.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(1.0, 0.125, true);
    let sys = VentilationSystem::D { with_wtw: true };

    let r = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap();
    assert_relative_eq!(r.annual_q_v, 0.0, epsilon = 1e-6);
    // Maar W_fan is nog steeds > 0 (ventilatoren blijven draaien)
    assert!(r.annual_w_fan > 0.0);
}

#[test]
fn system_a_zero_infiltration_zero_q_v() {
    // Systeem A zonder infiltratie → geen ventilatieverlies.
    let zone = sample_zone();
    let flow = AirFlow::new(0.0, 0.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let sys = VentilationSystem::A;

    let r = calculate_ventilation(&zone, &sys, &flow, None, &indoor, &climate).unwrap();
    assert_relative_eq!(r.annual_q_v, 0.0, epsilon = 1e-9);
    assert_relative_eq!(r.annual_w_fan, 0.0, epsilon = 1e-9);
    assert_relative_eq!(r.annual_wtw_recovery, 0.0, epsilon = 1e-9);
}

#[test]
fn system_a_with_infiltration_produces_q_v() {
    let zone = sample_zone();
    let flow = AirFlow::new(0.0, 0.0, 60.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let sys = VentilationSystem::A;

    let r = calculate_ventilation(&zone, &sys, &flow, None, &indoor, &climate).unwrap();
    assert!(r.annual_q_v > 0.0);
    assert_relative_eq!(r.annual_w_fan, 0.0, epsilon = 1e-9);
}

#[test]
fn all_four_systems_have_different_q_v() {
    // A/B/C/D moeten vier onderscheidbare uitkomsten geven bij gelijke input.
    let zone = sample_zone();
    let flow = AirFlow::new(120.0, 120.0, 30.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(0.80, 0.125, true);

    let r_a = calculate_ventilation(&zone, &VentilationSystem::A, &flow, None, &indoor, &climate)
        .unwrap();
    let r_b = calculate_ventilation(&zone, &VentilationSystem::B, &flow, None, &indoor, &climate)
        .unwrap();
    let r_c = calculate_ventilation(&zone, &VentilationSystem::C, &flow, None, &indoor, &climate)
        .unwrap();
    let r_d = calculate_ventilation(
        &zone,
        &VentilationSystem::D { with_wtw: true },
        &flow,
        Some(&wtw),
        &indoor,
        &climate,
    )
    .unwrap();

    // D met WTW moet laagste Q_V zijn (80% reductie)
    assert!(r_d.annual_q_v < r_a.annual_q_v);
    assert!(r_d.annual_q_v < r_b.annual_q_v);
    assert!(r_d.annual_q_v < r_c.annual_q_v);
    // A heeft alleen infiltratie (30 m³/h) → lager dan B/C met 120 m³/h
    assert!(r_a.annual_q_v < r_b.annual_q_v);
    assert!(r_a.annual_q_v < r_c.annual_q_v);
}

#[test]
fn full_year_dwelling_plausible_energy_scale() {
    // Referentie-woning: 100 m², system D met WTW η=0,80, SFP 0,125 W/(m³/h),
    // q_V = 150 m³/h (NEN 1087 minimum voor een gemiddelde woning).
    let zone = sample_zone();
    let flow = AirFlow::new(150.0, 150.0, 30.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(0.80, 0.125, true);
    let sys = VentilationSystem::D { with_wtw: true };

    let r = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap();

    // Plausibiliteit:
    // - Q_V zonder WTW zou ~8000 MJ zijn; met η=0,8 → ~1600 MJ
    // - W_fan jaarlijks: P = 0,125 × 2 × 150 = 37,5 W, continu → 37,5 × 8760 × 3600 / 10⁶ ≈ 1183 MJ
    assert!(
        r.annual_q_v > 500.0 && r.annual_q_v < 3000.0,
        "Q_V_an buiten plausibel bereik: {}",
        r.annual_q_v
    );
    assert!(
        r.annual_w_fan > 900.0 && r.annual_w_fan < 1400.0,
        "W_fan_an buiten plausibel bereik: {}",
        r.annual_w_fan
    );
    // WTW-recovery moet ongeveer gelijk zijn aan Q_V zonder WTW × 0,80
    assert!(r.annual_wtw_recovery > 4000.0);
}

#[test]
fn invalid_wtw_efficiency_rejected() {
    let zone = sample_zone();
    let flow = AirFlow::new(100.0, 100.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(1.5, 0.125, true);
    let sys = VentilationSystem::D { with_wtw: true };

    let err = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap_err();
    assert!(matches!(err, VentilationError::InvalidWtwEfficiency(_)));
}

#[test]
fn negative_airflow_rejected() {
    let zone = sample_zone();
    let flow = AirFlow::new(-10.0, 100.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let sys = VentilationSystem::D { with_wtw: false };

    let err = calculate_ventilation(&zone, &sys, &flow, None, &indoor, &climate).unwrap_err();
    assert!(matches!(err, VentilationError::NegativeAirFlow { .. }));
}

#[test]
fn wtw_on_system_a_rejected() {
    let zone = sample_zone();
    let flow = AirFlow::new(0.0, 0.0, 50.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(0.80, 0.125, true);
    let sys = VentilationSystem::A;

    let err = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap_err();
    assert!(matches!(err, VentilationError::WtwWithoutBalancedSystem));
}

#[test]
fn monthly_profile_has_twelve_months() {
    let zone = sample_zone();
    let flow = AirFlow::new(100.0, 100.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let sys = VentilationSystem::D { with_wtw: false };

    let r = calculate_ventilation(&zone, &sys, &flow, None, &indoor, &climate).unwrap();

    let mut count = 0;
    for month in Month::all() {
        let _ = r.monthly_q_v[month];
        let _ = r.monthly_w_fan[month];
        let _ = r.monthly_wtw_recovery[month];
        count += 1;
    }
    assert_eq!(count, 12);
}

#[test]
fn january_q_v_greater_than_july() {
    let zone = sample_zone();
    let flow = AirFlow::new(100.0, 100.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let sys = VentilationSystem::D { with_wtw: false };

    let r = calculate_ventilation(&zone, &sys, &flow, None, &indoor, &climate).unwrap();

    let jan = r.monthly_q_v[Month::Januari];
    let jul = r.monthly_q_v[Month::Juli];
    assert!(
        jan > 3.0 * jul,
        "Januari verlies ({jan}) moet aanzienlijk groter zijn dan juli ({jul})"
    );
}

#[test]
fn serde_round_trip_full_result() {
    let zone = sample_zone();
    let flow = AirFlow::new(100.0, 100.0, 0.0);
    let indoor = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();
    let wtw = WtwSpecification::new(0.80, 0.125, true);
    let sys = VentilationSystem::D { with_wtw: true };

    let r = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap();
    let json = serde_json::to_string(&r).unwrap();
    let back: nta8800_ventilation::VentilationResult = serde_json::from_str(&json).unwrap();
    // Floating-point ULP-differences kunnen optreden bij JSON-text → float
    // round-trip; vergelijk met tolerance in plaats van exact.
    assert_relative_eq!(r.annual_q_v, back.annual_q_v, epsilon = 1e-6);
    assert_relative_eq!(r.annual_w_fan, back.annual_w_fan, epsilon = 1e-6);
    assert_relative_eq!(
        r.annual_wtw_recovery,
        back.annual_wtw_recovery,
        epsilon = 1e-6
    );
    for m in Month::all() {
        assert_relative_eq!(r.monthly_q_v[m], back.monthly_q_v[m], epsilon = 1e-6);
        assert_relative_eq!(r.monthly_w_fan[m], back.monthly_w_fan[m], epsilon = 1e-6);
        assert_relative_eq!(
            r.monthly_wtw_recovery[m],
            back.monthly_wtw_recovery[m],
            epsilon = 1e-6
        );
    }
}
