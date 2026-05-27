//! Vabi-referentie verificatietests voor TO-juli (NTA 8800 H.10) berekeningen.
//!
//! Dit volgt het feedback_norm_voor_vabi.md principe: bereken met onze engine,
//! vergelijk met Vabi Elements output binnen gedefinieerde toleranties.
//!
//! ## Stappenplan voor gebruik
//!
//! Zodra Vabi-uitvoer beschikbaar is:
//! 1. Open `fixtures/vabi_tojuli_woning_120m2_expected.json`
//! 2. Vervang alle `placeholder_value` en `placeholder_values` met echte Vabi-cijfers
//! 3. Verwijder `#[ignore = "..."]` van de `vabi_tojuli_woning_120m2_matches` test
//! 4. Run `cargo test -p nta8800-cooling vabi_tojuli_woning_120m2_matches`
//!
//! ## KPI vergelijking
//!
//! De test vergelijkt 6 hoofdindicatoren:
//! - `annual_q_c_use_mj`: Jaarlijkse koel-energie in MJ (na COP-correctie)
//! - `annual_q_c_use_kwh`: Zelfde in kWh (conversie-check)
//! - `transmission_h_t_w_per_k`: Transmissie-warmteoverdrachtscoëfficiënt in W/K
//! - `ventilation_h_v_w_per_k`: Ventilatie-warmteoverdrachtscoëfficiënt in W/K
//! - `tau_hours`: Gebouw-tijdconstante in uren
//! - `monthly_*`: Maandelijkse verdelingen (12 waardes per indicator)
//!
//! ## Toleranties
//!
//! - KPI's: 10% (norm-vs-software variatie, conservatief)
//! - Maandwaarden: 15% (hoger door seizoenspieken en klimaat-interpolaties)
//!
//! Rationale: TO-juli is een complexe keten (transmissie + ventilatie + demand +
//! cooling) waar kleine verschillen in tabellen, klimaat of ventilatie-algoritmes
//! kunnen opstapelen. 10% KPI-tolerantie geeft ruimte voor legitieme
//! implementatie-verschillen terwijl grove fouten alsnog opvallen.

use openaec_project_shared::project::ProjectV2;
use openaec_project_shared::tojuli::{compute_tojuli_full, TojuliFullInputs};
use nta8800_cooling::{CoolingSystem, CoolingDistribution, CoolingEmission};
use serde_json;

/// Tolerantie-helper voor KPI vergelijking.
///
/// Vergelijkt `got` vs `want` binnen `tol_pct` percentage tolerantie.
/// Speciale behandeling voor waarde ≈ 0: absolute tolerantie 0.1 i.p.v. delen door 0.
fn close_kpi(label: &str, got: f64, want: f64, tol_pct: f64) {
    // Speciale case: want ≈ 0 → absolute tolerantie 0.1
    if want.abs() < f64::EPSILON {
        assert!(
            got.abs() < 0.1,
            "{label}: got {got:.2}, want ≈0 (absolute tolerantie >0.1)"
        );
        return;
    }

    let diff_pct = ((got - want) / want).abs() * 100.0;
    assert!(
        diff_pct < tol_pct,
        "{label}: got {got:.2}, want {want:.2} ({diff_pct:.1}% > {tol_pct}%)"
    );
}

/// Tolerantie-helper voor maandelijkse MonthlyProfile.
///
/// Vergelijkt MonthlyProfile vs array element-per-element binnen tolerantie.
fn close_monthly(label: &str, got_profile: &nta8800_model::time::MonthlyProfile<f64>, want: &[f64], tol_pct: f64) {
    assert_eq!(want.len(), 12, "{label}: want {} maanden, verwacht 12", want.len());

    let got_array = got_profile.as_array();
    for (month, (g, w)) in got_array.iter().zip(want.iter()).enumerate() {
        let month_name = [
            "jan", "feb", "mar", "apr", "mei", "jun",
            "jul", "aug", "sep", "okt", "nov", "dec"
        ][month];

        close_kpi(&format!("{label}[{month_name}]"), *g, *w, tol_pct);
    }
}

/// Load fixture input + expected output.
fn load_fixtures() -> (ProjectV2, TojuliFullInputs, serde_json::Value) {
    // Load ProjectV2 van fixture input JSON
    let input_json = include_str!(
        "../../../tests/verification/tojuli_vabi3.12.0.127_dr-engineering-woningbouw/input.json"
    );
    let project: ProjectV2 = serde_json::from_str(input_json)
        .expect("fixture input moet valide ProjectV2 JSON zijn");

    // Default TO-juli inputs - vergelijkbaar met Vabi standaard installatie
    let inputs = TojuliFullInputs {
        system: CoolingSystem::CompressionCooling { scop_cooling: 3.5 },
        distribution: CoolingDistribution::default_insulated(),
        emission: CoolingEmission {
            efficiency: 0.95,
            regulation_factor: 0.95,
        },
        shading_factor: 1.0, // geen schaduw
        heating_setpoint_c: 20.0,
        cooling_setpoint_c: 24.0,
    };

    // Load expected output
    let expected_json = include_str!(
        "../../../tests/verification/tojuli_vabi3.12.0.127_dr-engineering-woningbouw/expected.json"
    );
    let expected: serde_json::Value = serde_json::from_str(expected_json)
        .expect("fixture expected moet valide JSON zijn");

    (project, inputs, expected)
}

/// Test: input fixture compileert en compute_tojuli_full draait zonder error.
///
/// Deze test is NIET ignored - moet altijd slagen, ook met placeholder data.
/// Doel: schema-validatie + verifieer dat de TO-juli keten werkt met fixture input.
#[test]
fn vabi_tojuli_woning_120m2_compiles_and_runs() {
    let (project, inputs, _expected) = load_fixtures();

    // Dit moet gewoon werken - geen panic/error
    let result = compute_tojuli_full(&project, &inputs)
        .expect("compute_tojuli_full moet slagen met fixture input");

    // Sanity checks: geen NaN/Inf, basale grenzen
    assert!(result.transmission_h_t_w_per_k.is_finite());
    assert!(result.ventilation_h_v_w_per_k.is_finite());
    assert!(result.annual_q_c_use_mj.is_finite());
    assert!(result.annual_q_c_use_kwh.is_finite());
    assert!(result.tau_hours.is_finite());

    assert!(result.transmission_h_t_w_per_k >= 0.0);
    assert!(result.ventilation_h_v_w_per_k >= 0.0);
    assert!(result.annual_q_c_use_mj >= 0.0);
    assert!(result.annual_q_c_use_kwh >= 0.0);
    assert!(result.tau_hours > 0.0);

    // Maandelijkse arrays zijn 12 lang
    assert_eq!(result.monthly_q_c_nd_mj.as_array().len(), 12);
    assert_eq!(result.monthly_q_c_use_mj.as_array().len(), 12);
    assert_eq!(result.monthly_q_h_nd_mj.as_array().len(), 12);
    assert_eq!(result.monthly_theta_e_c.as_array().len(), 12);
}

/// Test: KPI's binnen plausibele ranges (geen Vabi-vergelijking).
///
/// Deze test is NIET ignored - werkt met placeholder data.
/// Doel: vroege detectie van grove regressies in de TO-juli keten.
#[test]
fn vabi_tojuli_woning_120m2_kpis_in_plausible_range() {
    let (project, inputs, _expected) = load_fixtures();
    let result = compute_tojuli_full(&project, &inputs).unwrap();

    // Plausibiliteits-ranges voor 120m² woning in NL

    // Q_C;use jaar: 100-20000 MJ (≈28-5556 kWh) - conservatief bereik
    // Note: fixture heeft veel raamoppervlak (21 m²) en groot dak (72 m²) → hoge zonbelasting
    assert!(result.annual_q_c_use_mj >= 100.0,
        "Q_C;use te laag: {} MJ", result.annual_q_c_use_mj);
    assert!(result.annual_q_c_use_mj <= 20000.0,
        "Q_C;use te hoog: {} MJ", result.annual_q_c_use_mj);

    // Conversie kWh ≈ MJ/3.6
    let expected_kwh = result.annual_q_c_use_mj / 3.6;
    assert!(
        (result.annual_q_c_use_kwh - expected_kwh).abs() < 0.1,
        "kWh conversie: got {}, verwacht {expected_kwh}",
        result.annual_q_c_use_kwh
    );

    // H_T: 30-200 W/K (klein-groot huis, isolatie-spreiding)
    assert!(result.transmission_h_t_w_per_k >= 30.0);
    assert!(result.transmission_h_t_w_per_k <= 200.0);

    // H_V: 20-150 W/K (ventilatie-spreiding)
    assert!(result.ventilation_h_v_w_per_k >= 20.0);
    assert!(result.ventilation_h_v_w_per_k <= 150.0);

    // τ: 5-500 uur (licht-zwaar gebouw) - lager minimum voor lichte constructie
    assert!(result.tau_hours >= 5.0,
        "τ te laag: {} uur", result.tau_hours);
    assert!(result.tau_hours <= 500.0,
        "τ te hoog: {} uur", result.tau_hours);

    // Maandelijks: geen negatieve waarden
    for (_, &q) in result.monthly_q_c_nd_mj.iter() {
        assert!(q >= 0.0, "Q_C;nd maandelijks mag niet negatief zijn: {q}");
    }
    for (_, &q) in result.monthly_q_c_use_mj.iter() {
        assert!(q >= 0.0, "Q_C;use maandelijks mag niet negatief zijn: {q}");
    }
    // Q_H;nd mag negatief zijn (passiefhuis zomer), maar niet extreem
    // Breder bereik: woning met veel glas kan hoge verwarmingsbehoefte hebben
    for (_, &q) in result.monthly_q_h_nd_mj.iter() {
        assert!(q >= -500.0 && q <= 6000.0, "Q_H;nd buiten plausibel bereik: {q}");
    }

    // θ_e: klimaat De Bilt ongeveer -5 tot +25°C
    for (_, &t) in result.monthly_theta_e_c.iter() {
        assert!(t >= -10.0 && t <= 30.0, "θ_e buiten plausibel bereik: {t}°C");
    }
}

/// Test: exacte Vabi-match binnen toleranties.
///
/// IGNORED tot Vabi-data beschikbaar. Zodra expected.json gevuld:
/// - Verwijder ignore-attribute
/// - Run test → moet groen worden binnen toleranties
#[test]
#[ignore = "fixture bevat placeholder values — vervang expected.json met echte Vabi-output en verwijder ignore"]
fn vabi_tojuli_woning_120m2_matches() {
    let (project, inputs, expected) = load_fixtures();
    let result = compute_tojuli_full(&project, &inputs).unwrap();

    // KPI's uit expected JSON
    let kpis = &expected["kpis"];

    close_kpi(
        "annual_q_c_use_mj",
        result.annual_q_c_use_mj,
        kpis["annual_q_c_use_mj"]["placeholder_value"].as_f64().unwrap(),
        10.0
    );

    close_kpi(
        "annual_q_c_use_kwh",
        result.annual_q_c_use_kwh,
        kpis["annual_q_c_use_kwh"]["placeholder_value"].as_f64().unwrap(),
        10.0
    );

    close_kpi(
        "transmission_h_t_w_per_k",
        result.transmission_h_t_w_per_k,
        kpis["transmission_h_t_w_per_k"]["placeholder_value"].as_f64().unwrap(),
        10.0
    );

    close_kpi(
        "ventilation_h_v_w_per_k",
        result.ventilation_h_v_w_per_k,
        kpis["ventilation_h_v_w_per_k"]["placeholder_value"].as_f64().unwrap(),
        10.0
    );

    close_kpi(
        "tau_hours",
        result.tau_hours,
        kpis["tau_hours"]["placeholder_value"].as_f64().unwrap(),
        10.0
    );

    // Maandelijkse data
    let monthly = &expected["monthly_data"];

    let want_q_c_nd: Vec<f64> = monthly["monthly_q_c_nd_mj"]["placeholder_values"]
        .as_array().unwrap()
        .iter().map(|v| v.as_f64().unwrap()).collect();
    close_monthly("monthly_q_c_nd_mj", &result.monthly_q_c_nd_mj, &want_q_c_nd, 15.0);

    let want_q_c_use: Vec<f64> = monthly["monthly_q_c_use_mj"]["placeholder_values"]
        .as_array().unwrap()
        .iter().map(|v| v.as_f64().unwrap()).collect();
    close_monthly("monthly_q_c_use_mj", &result.monthly_q_c_use_mj, &want_q_c_use, 15.0);

    let want_q_h_nd: Vec<f64> = monthly["monthly_q_h_nd_mj"]["placeholder_values"]
        .as_array().unwrap()
        .iter().map(|v| v.as_f64().unwrap()).collect();
    close_monthly("monthly_q_h_nd_mj", &result.monthly_q_h_nd_mj, &want_q_h_nd, 15.0);

    let want_theta_e: Vec<f64> = monthly["monthly_theta_e_c"]["placeholder_values"]
        .as_array().unwrap()
        .iter().map(|v| v.as_f64().unwrap()).collect();
    close_monthly("monthly_theta_e_c", &result.monthly_theta_e_c, &want_theta_e, 15.0);
}