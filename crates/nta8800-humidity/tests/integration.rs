//! Integration tests voor nta8800-humidity.

use approx::assert_abs_diff_eq;
use nta8800_humidity::{
    calculate_humidity, DehumidificationSystem, HumidificationSystem, HumiditySystemConfig,
    HumidityTarget,
};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::Rekenzone;
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;

#[test]
fn humidity_calculation_office_winter_humidification() {
    // Setup test zone (kantoorruimte)
    let zone = Rekenzone {
        id: "test_office".into(),
        name: "Test Kantoor".into(),
        gebouw_id: "test_building".into(),
        floor_area: 100.0,
        volume: 275.0, // 2.75m hoog
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    // Systeem configuratie met steam humidifier
    let system_config = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::Steam { efficiency: 0.95 }),
        dehumidification: Some(DehumidificationSystem::Cooling { cop: 3.5 }),
        target: HumidityTarget::office(), // 6-12 g/kg
    };

    // Constante binnentemperatuur 21°C
    let indoor_temp = MonthlyProfile::from_constant(21.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate).unwrap();

    // Verwachtingen:
    // - Wintermaanden (okt-mrt) hebben bevochtigingsbehoefte omdat koude lucht droog is
    // - Zomermaanden (apr-sep) hebben mogelijk ontvochtigingsbehoefte
    // - Totale energie > 0 door activiteit
    assert!(result.annual_q_hum > 0.0, "Verwacht bevochtigingsbehoefte in winter");
    assert!(result.annual_w_hum > 0.0, "Verwacht elektrisch verbruik humidifier");
    assert!(result.has_humidity_activity(), "Verwacht enige humidity activiteit");
}

#[test]
fn humidity_calculation_no_systems() {
    let zone = Rekenzone {
        id: "test_no_sys".into(),
        name: "Test Geen Systemen".into(),
        gebouw_id: "test_building".into(),
        floor_area: 50.0,
        volume: 125.0,
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    // Geen systemen gedefinieerd
    let system_config = HumiditySystemConfig {
        humidification: None,
        dehumidification: None,
        target: HumidityTarget::office(),
    };

    let indoor_temp = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate).unwrap();

    // Zonder systemen geen elektrisch verbruik, wel mogelijk thermische behoefte berekend
    assert_eq!(result.annual_w_hum, 0.0, "Geen elektrisch verbruik zonder systemen");
    assert!(!result.has_humidity_activity() || result.annual_w_hum == 0.0);
}

#[test]
fn humidity_calculation_spray_coiler_summer() {
    let zone = Rekenzone {
        id: "test_spray".into(),
        name: "Test Sproeikoeler".into(),
        gebouw_id: "test_building".into(),
        floor_area: 200.0,
        volume: 600.0, // Grote ruimte
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    let system_config = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::SprayCoiler {
            effectiveness: 0.80,
        }),
        dehumidification: Some(DehumidificationSystem::EvaporativeCooling {
            effectiveness: 0.75,
        }),
        target: HumidityTarget::laboratory(), // Droger: 4-8 g/kg
    };

    let indoor_temp = MonthlyProfile::from_constant(22.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate).unwrap();

    // Laboratorium heeft lage vochtigheid targets → meer kans op bevochtiging
    assert!(result.annual_total_energy() >= 0.0);

    // Check dat resultaat logisch is
    if result.has_humidity_activity() {
        let dominant = result.dominant_activity();
        assert!(dominant == "humidification" || dominant == "dehumidification" || dominant == "none");
    }
}

#[test]
fn humidity_target_validation() {
    let zone = Rekenzone {
        id: "test_valid".into(),
        name: "Test Validatie".into(),
        gebouw_id: "test_building".into(),
        floor_area: 100.0,
        volume: 250.0,
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    // Ongeldige range: min >= max
    let invalid_system = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::Steam { efficiency: 0.95 }),
        dehumidification: None,
        target: HumidityTarget {
            min_g_per_kg: 12.0,
            max_g_per_kg: 6.0, // max < min → fout
        },
    };

    let indoor_temp = MonthlyProfile::from_constant(21.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &invalid_system, &indoor_temp, &climate);
    assert!(result.is_err(), "Verwacht fout bij ongeldige humidity range");
}

#[test]
fn humidity_system_efficiency_validation() {
    let zone = Rekenzone {
        id: "test_eff".into(),
        name: "Test Efficiency".into(),
        gebouw_id: "test_building".into(),
        floor_area: 100.0,
        volume: 250.0,
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    // Ongeldige efficiency > 1.0
    let invalid_system = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::Steam {
            efficiency: 1.2, // > 1.0 → fout
        }),
        dehumidification: None,
        target: HumidityTarget::office(),
    };

    let indoor_temp = MonthlyProfile::from_constant(21.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &invalid_system, &indoor_temp, &climate);
    assert!(result.is_err(), "Verwacht fout bij efficiency > 1.0");
}

#[test]
fn humidity_negative_zone_volume() {
    let zone = Rekenzone {
        id: "test_negative".into(),
        name: "Test Negatief Volume".into(),
        gebouw_id: "test_building".into(),
        floor_area: 100.0,
        volume: -50.0, // Negatief volume
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    let system_config = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::Steam { efficiency: 0.95 }),
        dehumidification: None,
        target: HumidityTarget::office(),
    };

    let indoor_temp = MonthlyProfile::from_constant(21.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate);
    assert!(result.is_err(), "Verwacht fout bij negatief zone volume");
}

#[test]
fn humidity_monthly_profile_consistency() {
    let zone = Rekenzone {
        id: "test_monthly".into(),
        name: "Test Maandelijks".into(),
        gebouw_id: "test_building".into(),
        floor_area: 100.0,
        volume: 250.0,
        efr_ids: vec![],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    };

    let system_config = HumiditySystemConfig {
        humidification: Some(HumidificationSystem::Ultrasonic { efficiency: 0.85 }),
        dehumidification: Some(DehumidificationSystem::Adsorption { cop: 4.0 }),
        target: HumidityTarget::residential(),
    };

    let indoor_temp = MonthlyProfile::from_constant(20.0);
    let climate = de_bilt_climate_data();

    let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate).unwrap();

    // Check dat jaarwaarden gelijk zijn aan som van maandwaarden
    let sum_monthly_humidification: f64 = result.monthly_humidification.as_array().iter().sum();
    let sum_monthly_dehumidification: f64 = result.monthly_dehumidification.as_array().iter().sum();
    let sum_monthly_electrical: f64 = result.monthly_electrical.as_array().iter().sum();

    assert_abs_diff_eq!(result.annual_q_hum, sum_monthly_humidification, epsilon = 0.001);
    assert_abs_diff_eq!(result.annual_q_dhum, sum_monthly_dehumidification, epsilon = 0.001);
    assert_abs_diff_eq!(result.annual_w_hum, sum_monthly_electrical, epsilon = 0.001);

    // Check dat monthly_total_energy correct is
    let monthly_total = result.monthly_total_energy();
    for i in 0..12 {
        let expected = result.monthly_humidification.as_array()[i]
            + result.monthly_dehumidification.as_array()[i]
            + result.monthly_electrical.as_array()[i];
        assert_abs_diff_eq!(monthly_total.as_array()[i], expected, epsilon = 0.001);
    }
}