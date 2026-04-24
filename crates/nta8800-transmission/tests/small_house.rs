//! Integration-test — kleine woning met De Bilt klimaat.
//!
//! Simuleert één rekenzone van 100 m² vloeroppervlak met 4 gevels, 2 ramen,
//! 1 hellend dak (outdoor) en 1 vloer-op-grond. Controleert dat:
//!
//! - De annual Q_T in een plausibel bereik ligt voor een naoorlogse woning
//!   met Rc ≈ 2,5 m²K/W (geen passiefhuis, geen slechte schil)
//! - De breakdown-som exact gelijk is aan monthly_q_t per maand
//! - Januari hogere transmissie heeft dan juli (voor de hand liggende
//!   sanity-check op de maandcurve)

use std::collections::HashMap;

use approx::assert_abs_diff_eq;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::{
    geometry::ThermalBridgeCategory, Rekenzone, ThermalBridgeLinear, ThermalBridgePoint,
};
use nta8800_tables::climate::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};

fn sample_zone() -> Rekenzone {
    Rekenzone {
        id: "rz-1".into(),
        name: "woonlaag".into(),
        gebouw_id: "g-1".into(),
        floor_area: 100.0,
        volume: 250.0,
        efr_ids: vec!["efr-1".into()],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    }
}

fn outdoor(id: &str, area: f64, u: f64) -> TransmissionElement {
    TransmissionElement {
        id: id.into(),
        area,
        u_value: u,
        boundary_type: BoundaryType::Outdoor,
        construction_id: None,
    }
}

fn ground_el(id: &str, area: f64, u: f64) -> TransmissionElement {
    TransmissionElement {
        id: id.into(),
        area,
        u_value: u,
        boundary_type: BoundaryType::Ground,
        construction_id: None,
    }
}

#[test]
fn small_house_annual_in_plausible_range() {
    let zone = sample_zone();

    // Schil Rc ≈ 2.5 → U ≈ 0.4 W/(m²·K); ramen U=2.0 (dubbelglas HR).
    let elements = vec![
        outdoor("gevel-zuid", 25.0, 0.4),
        outdoor("gevel-west", 25.0, 0.4),
        outdoor("gevel-noord", 20.0, 0.4),
        outdoor("gevel-oost", 20.0, 0.4),
        outdoor("dak", 60.0, 0.3),
        outdoor("raam-z", 6.0, 2.0),
        outdoor("raam-w", 4.0, 2.0),
        ground_el("vloer", 100.0, 0.3),
    ];

    let bridges_lin = vec![
        ThermalBridgeLinear {
            id: "vloerrand".into(),
            length: 40.0, // omtrek woning
            psi: 0.10,
            category: ThermalBridgeCategory::AansluitingVloerGevel,
        },
        ThermalBridgeLinear {
            id: "kozijn-aansluiting".into(),
            length: 30.0,
            psi: 0.05,
            category: ThermalBridgeCategory::RaamKader,
        },
    ];
    let bridges_pt: Vec<ThermalBridgePoint> = vec![];

    let climate = de_bilt_climate_data();

    // Indoor setpoint 20 °C constant
    let indoor = MonthlyProfile::from_constant(20.0);

    // Vereenvoudigde H_g;an voor vloer 100 m², U=0.3, b_g ≈ 0.55 → ~16.5 W/K
    let h_g_an = 100.0 * 0.3 * 0.55;

    let b_factors: HashMap<String, f64> = HashMap::new();
    let adj: HashMap<String, MonthlyProfile<f64>> = HashMap::new();

    let result = calculate_transmission(
        &zone,
        &elements,
        &bridges_lin,
        &bridges_pt,
        &indoor,
        &climate,
        h_g_an,
        &b_factors,
        &adj,
    )
    .expect("calculation should succeed");

    // --- Check H_D ---
    // Elements outdoor: 25·0.4 + 25·0.4 + 20·0.4 + 20·0.4 + 60·0.3 + 6·2 + 4·2
    //  = 10 + 10 + 8 + 8 + 18 + 12 + 8 = 74 W/K
    // Plus bruggen: 40·0.10 + 30·0.05 = 4.0 + 1.5 = 5.5
    // Totaal H_D = 79.5 W/K
    assert_abs_diff_eq!(result.h_d, 79.5, epsilon = 1e-9);

    // --- Check H_g_an ---
    assert_abs_diff_eq!(result.h_g_an, h_g_an, epsilon = 1e-9);

    // --- Check H_U en H_A default 0 ---
    assert_abs_diff_eq!(result.h_u, 0.0, epsilon = 1e-9);
    assert_abs_diff_eq!(result.h_a, 0.0, epsilon = 1e-9);

    // --- Breakdown som = totaal per maand ---
    for m in Month::all() {
        let bd_sum = result.breakdown.outdoor[m]
            + result.breakdown.unheated_space[m]
            + result.breakdown.ground[m]
            + result.breakdown.adjacent_zone[m]
            + result.breakdown.thermal_bridges[m];
        assert_abs_diff_eq!(result.monthly_q_t[m], bd_sum, epsilon = 1e-6);
    }

    // --- Januari > juli (winter vs zomer) ---
    assert!(
        result.monthly_q_t[Month::Januari] > result.monthly_q_t[Month::Juli],
        "januari {} moet > juli {}",
        result.monthly_q_t[Month::Januari],
        result.monthly_q_t[Month::Juli]
    );

    // --- Plausibel jaar-bereik voor deze naoorlogse woning ---
    // Ruwe schatting: H_total ≈ 95 W/K, gemiddeld ΔT ≈ 10 K jaar-round,
    //   Q ≈ 95 · 10 · 8760 · 0.001 · 3.6 ≈ 30 000 MJ
    // Accepteer 10 000 – 60 000 MJ bandbreedte.
    assert!(
        (10_000.0..=60_000.0).contains(&result.annual_q_t),
        "annual_q_t = {} MJ buiten plausibel bereik",
        result.annual_q_t
    );
}

#[test]
fn small_house_with_unheated_garage() {
    let zone = sample_zone();

    let elements = vec![
        outdoor("gevel-zuid", 25.0, 0.4),
        outdoor("dak", 60.0, 0.3),
        ground_el("vloer", 100.0, 0.3),
        TransmissionElement {
            id: "wand-garage".into(),
            area: 15.0,
            u_value: 0.5,
            boundary_type: BoundaryType::UnheatedSpace {
                id: "garage".into(),
            },
            construction_id: None,
        },
    ];
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);

    let mut b_factors = HashMap::new();
    b_factors.insert("garage".into(), 0.8);

    let result = calculate_transmission(
        &zone,
        &elements,
        &[],
        &[],
        &indoor,
        &climate,
        10.0,
        &b_factors,
        &HashMap::new(),
    )
    .unwrap();

    // H_U = 15·0.5·0.8 = 6.0 W/K
    assert_abs_diff_eq!(result.h_u, 6.0, epsilon = 1e-9);
    // unheated_space breakdown moet non-zero zijn voor winter-maanden
    assert!(result.breakdown.unheated_space[Month::Januari] > 0.0);
}

#[test]
fn missing_b_factor_returns_error() {
    let zone = sample_zone();
    let elements = vec![TransmissionElement {
        id: "wand-garage".into(),
        area: 15.0,
        u_value: 0.5,
        boundary_type: BoundaryType::UnheatedSpace {
            id: "garage".into(),
        },
        construction_id: None,
    }];
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);

    let err = calculate_transmission(
        &zone,
        &elements,
        &[],
        &[],
        &indoor,
        &climate,
        0.0,
        &HashMap::new(),
        &HashMap::new(),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        nta8800_transmission::TransmissionError::MissingUnheatedBFactor { ref id } if id == "garage"
    ));
}

#[test]
fn adjacent_zone_optin_with_profile_reduces_transmission() {
    let zone = sample_zone();

    // Zelfde setup maar één wand naar buurwoning
    let elements = vec![
        outdoor("gevel-zuid", 25.0, 0.4),
        TransmissionElement {
            id: "wand-buur".into(),
            area: 30.0,
            u_value: 1.0,
            boundary_type: BoundaryType::AdjacentZone { id: "buur".into() },
            construction_id: None,
        },
    ];
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);

    // Buur verwarmd op 18°C (lagere setpoint)
    let mut adj = HashMap::new();
    adj.insert("buur".into(), MonthlyProfile::from_constant(18.0));

    let with_adj = calculate_transmission(
        &zone,
        &elements,
        &[],
        &[],
        &indoor,
        &climate,
        0.0,
        &HashMap::new(),
        &adj,
    )
    .unwrap();

    // Zonder adjacent profile = NTA default 0
    let without_adj = calculate_transmission(
        &zone,
        &elements,
        &[],
        &[],
        &indoor,
        &climate,
        0.0,
        &HashMap::new(),
        &HashMap::new(),
    )
    .unwrap();

    // Adjacent-breakdown moet iets opleveren (warmte stroomt naar de koudere buur)
    assert!(with_adj.breakdown.adjacent_zone[Month::Januari] > 0.0);
    // Zonder profile = 0
    assert_abs_diff_eq!(
        without_adj.breakdown.adjacent_zone[Month::Januari],
        0.0,
        epsilon = 1e-9
    );
}
