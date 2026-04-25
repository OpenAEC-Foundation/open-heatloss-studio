//! Integratietests voor `nta8800-automation`.

use nta8800_automation::{
    calculate_automation_factors, AutomationConfig, BacsClass,
};
use nta8800_model::zoning::UsageFunction;

#[test]
fn residential_baseline_calculation() {
    // Test standaard woningconfiguratie met klasse C (referentie)
    let config = AutomationConfig::standard();
    let factors = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();

    // Alle factoren moeten 1.0 zijn voor klasse C
    assert_eq!(factors.f_bac_heating, 1.0);
    assert_eq!(factors.f_bac_cooling, 1.0);
    assert_eq!(factors.f_bac_lighting, 1.0);
    assert_eq!(factors.f_bac_dhw, 1.0);
    assert_eq!(factors.f_bac_ventilation, 1.0);
    assert_eq!(factors.average_factor(), 1.0);
    assert_eq!(factors.count_energy_saving_services(), 0);
}

#[test]
fn office_high_performance_system() {
    // Test kantoor met geavanceerde automatisering
    let config = AutomationConfig {
        heating: BacsClass::A,
        cooling: BacsClass::A,
        lighting: BacsClass::A,
        dhw: BacsClass::B,        // DHW meestal minder geavanceerd
        ventilation: BacsClass::B,
    };

    let factors = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();

    // Alle factoren moeten energiebesparing opleveren
    assert!(factors.f_bac_heating < 1.0);
    assert!(factors.f_bac_cooling < 1.0);
    assert!(factors.f_bac_lighting < 1.0);
    assert!(factors.f_bac_dhw < 1.0);
    assert!(factors.f_bac_ventilation < 1.0);
    assert!(factors.average_factor() < 1.0);
    assert_eq!(factors.count_energy_saving_services(), 5);
}

#[test]
fn legacy_building_energy_waste() {
    // Test verouderd gebouw met slechte automatisering
    let config = AutomationConfig::non_efficient();
    let factors = calculate_automation_factors(&config, UsageFunction::Onderwijsfunctie).unwrap();

    // Alle factoren moeten energieverspilling tonen
    assert!(factors.f_bac_heating > 1.0);
    assert!(factors.f_bac_cooling > 1.0);
    assert!(factors.f_bac_lighting > 1.0);
    assert!(factors.f_bac_dhw > 1.0);
    assert!(factors.f_bac_ventilation > 1.0);
    assert!(factors.average_factor() > 1.0);
    assert_eq!(factors.count_energy_saving_services(), 0);
}

#[test]
fn mixed_automation_levels() {
    // Realistisch scenario: gemengde automatiseringsniveaus
    let config = AutomationConfig {
        heating: BacsClass::B,    // Goed
        cooling: BacsClass::A,    // Excellent (nieuwe installatie)
        lighting: BacsClass::C,   // Standaard
        dhw: BacsClass::D,        // Verouderd
        ventilation: BacsClass::C, // Standaard
    };

    let factors = calculate_automation_factors(&config, UsageFunction::Gezondheidszorgfunctie).unwrap();

    assert!(factors.f_bac_heating < 1.0);   // B = besparing
    assert!(factors.f_bac_cooling < 1.0);   // A = grote besparing
    assert_eq!(factors.f_bac_lighting, 1.0);   // C = referentie
    assert!(factors.f_bac_dhw > 1.0);       // D = verspilling
    assert_eq!(factors.f_bac_ventilation, 1.0); // C = referentie

    // Gemengd resultaat
    assert_eq!(factors.count_energy_saving_services(), 2);
}

#[test]
fn residential_vs_non_residential_differences() {
    let config = AutomationConfig::uniform(BacsClass::A);

    let residential = calculate_automation_factors(&config, UsageFunction::Woonfunctie).unwrap();
    let office = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();

    // Non-residential heeft meestal meer besparingspotentieel
    // vooral voor verlichting door meer complexe gebruikspatronen
    assert!(office.f_bac_lighting <= residential.f_bac_lighting);
}

#[test]
fn all_bac_classes_produce_valid_results() {
    let functions = [UsageFunction::Woonfunctie, UsageFunction::Kantoorfunctie];

    for usage_function in functions {
        for heating_class in BacsClass::all_ordered() {
            for lighting_class in BacsClass::all_ordered() {
                let config = AutomationConfig {
                    heating: heating_class,
                    cooling: BacsClass::C,
                    lighting: lighting_class,
                    dhw: BacsClass::C,
                    ventilation: BacsClass::C,
                };

                let result = calculate_automation_factors(&config, usage_function);
                assert!(result.is_ok(),
                    "Configuratie {heating_class:?}/{lighting_class:?} voor {usage_function:?} faalt");

                let factors = result.unwrap();
                assert!(factors.is_physically_realistic());
            }
        }
    }
}

#[test]
fn factor_boundary_validation() {
    // Test dat extreme combinaties nog steeds realistische factoren geven
    let extreme_good = AutomationConfig::uniform(BacsClass::A);
    let extreme_bad = AutomationConfig::uniform(BacsClass::D);

    for usage_function in [UsageFunction::Woonfunctie, UsageFunction::Industriefunctie] {
        let good_factors = calculate_automation_factors(&extreme_good, usage_function).unwrap();
        let bad_factors = calculate_automation_factors(&extreme_bad, usage_function).unwrap();

        // Alle factoren binnen [0.5, 2.0]
        assert!(good_factors.is_physically_realistic());
        assert!(bad_factors.is_physically_realistic());

        // Goede automatisering moet altijd beter zijn dan slechte
        assert!(good_factors.average_factor() < bad_factors.average_factor());
    }
}

#[test]
fn energy_service_coverage() {
    let config = AutomationConfig::uniform(BacsClass::B);
    let factors = calculate_automation_factors(&config, UsageFunction::Winkelfunctie).unwrap();

    // Alle energiediensten moeten een factor hebben
    assert!(factors.get_factor_for_service("heating").is_some());
    assert!(factors.get_factor_for_service("cooling").is_some());
    assert!(factors.get_factor_for_service("lighting").is_some());
    assert!(factors.get_factor_for_service("dhw").is_some());
    assert!(factors.get_factor_for_service("ventilation").is_some());
    assert!(factors.get_factor_for_service("invalid").is_none());
}