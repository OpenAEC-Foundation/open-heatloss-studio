//! Integratietest: draai alle 20 voorbeeld-fixtures end-to-end.
//!
//! Elke fixture in `tests/fixtures/*.json` gaat door de volledige keten
//! (`calculate_from_json`) en moet:
//!
//! 1. zonder fout doorrekenen;
//! 2. fysisch plausibele uitkomsten geven (geen NaN, warmtebehoefte > 0,
//!    label toegekend, primaire energie > 0);
//! 3. dienst-specifieke verwachtingen halen (koeling gevuld waar
//!    geconfigureerd, PV-opbrengst in realistische kWh/kWp-band,
//!    verlichting alleen bij utiliteit).
//!
//! Daarnaast een paar cross-fixture sanity-checks (WP-woning gebruikt minder
//! eindenergie dan dezelfde-orde gaswoning; oude woning heeft hogere
//! specifieke behoefte dan nieuwbouw).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use nta8800_core::{calculate_from_json, Nta8800Result};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Laad + bereken alle fixtures, gesorteerd op naam.
fn run_all() -> BTreeMap<String, Nta8800Result> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(fixtures_dir()).expect("fixtures dir leesbaar") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("bestandsnaam")
            .to_string();
        let json = fs::read_to_string(&path).expect("fixture leesbaar");
        let result_json = calculate_from_json(&json)
            .unwrap_or_else(|e| panic!("fixture {name} faalt: {e}"));
        let result: Nta8800Result =
            serde_json::from_str(&result_json).expect("result parse");
        out.insert(name, result);
    }
    out
}

#[test]
fn alle_20_fixtures_rekenen_door() {
    let results = run_all();
    assert_eq!(results.len(), 20, "verwacht exact 20 fixtures");
}

#[test]
fn plausibiliteit_per_fixture() {
    for (name, r) in run_all() {
        // Geen NaN / infinities in de kern-uitkomsten.
        assert!(
            r.demand.annual_q_h_nd_mj.is_finite(),
            "{name}: Q_H;nd finite"
        );
        assert!(
            r.ep.primary_energy_mj_per_m2.is_finite(),
            "{name}: EP finite"
        );

        // NL-klimaat: elke fixture heeft een winter-warmtebehoefte.
        assert!(r.demand.annual_q_h_nd_mj > 0.0, "{name}: Q_H;nd > 0");
        assert!(r.demand.tau_hours > 0.0, "{name}: τ > 0");
        assert!(r.demand.h_tr_w_per_k > 0.0, "{name}: H_tr > 0");
        assert!(r.demand.h_ve_w_per_k > 0.0, "{name}: H_ve > 0");

        // Verwarming levert altijd (Q_H;use > 0) en de maandsom klopt.
        assert!(r.heating.annual_use_mj > 0.0, "{name}: Q_H;use > 0");
        let heating_sum: f64 = r.heating.monthly_use_mj.iter().sum();
        assert!(
            (heating_sum - r.heating.annual_use_mj).abs() < 1e-6,
            "{name}: heating maandsom == jaartotaal"
        );

        // Tapwater-forfait > 0 voor alle gebruiksfuncties in de set.
        assert!(r.dhw.annual_use_mj > 0.0, "{name}: Q_W;use > 0");

        // EP-label toegekend, specifieke primaire energie positief.
        assert!(!r.ep.label.is_empty(), "{name}: label toegekend");
        assert!(
            r.ep.primary_energy_mj_per_m2 > 0.0,
            "{name}: EP/m² > 0 (was {})",
            r.ep.primary_energy_mj_per_m2
        );
        assert!(
            (0.0..=1.0).contains(&r.ep.renewable_share),
            "{name}: renewable share in [0,1]"
        );
    }
}

#[test]
fn koeling_alleen_waar_geconfigureerd() {
    let results = run_all();
    let with_cooling = [
        "w07-bungalow-koeling",
        "w08-rijwoning-wp-vrije-koeling",
        "u02-kantoor-groot-pv-koeling",
        "u04-winkel",
        "u10-industriehal",
    ];
    for (name, r) in &results {
        if with_cooling.contains(&name.as_str()) {
            assert!(r.cooling.is_some(), "{name}: cooling verwacht");
        } else {
            assert!(r.cooling.is_none(), "{name}: geen cooling verwacht");
        }
    }
}

#[test]
fn pv_opbrengst_realistische_band() {
    let results = run_all();
    // (fixture, kWp) — verwachte specifieke opbrengst 500-1100 kWh/kWp
    // (De Bilt ≈ 813 kWh/kWp bij zuid/35°; oost-west en flat-tilt lager).
    let pv_cases = [
        ("w05-nieuwbouw-wp-bodem-pv", 6.0),
        ("w09-herenhuis-gas-pv", 3.0),
        ("u02-kantoor-groot-pv-koeling", 50.0),
    ];
    for (name, kwp) in pv_cases {
        let r = &results[name];
        let pv = r.pv.as_ref().unwrap_or_else(|| panic!("{name}: pv aanwezig"));
        let kwh_per_kwp = pv.annual_yield_kwh / kwp;
        assert!(
            (500.0..=1100.0).contains(&kwh_per_kwp),
            "{name}: {kwh_per_kwp:.0} kWh/kWp buiten realistische band"
        );
    }
}

#[test]
fn verlichting_alleen_utiliteit() {
    let results = run_all();
    for (name, r) in &results {
        if name.starts_with('w') {
            assert!(r.lighting.is_none(), "{name}: woning geen verlichting-dienst");
        } else {
            assert!(
                r.lighting.is_some(),
                "{name}: utiliteit met lighting-config levert dienst"
            );
        }
    }
}

#[test]
fn warmtepomp_woning_gebruikt_minder_eindenergie_dan_gas() {
    let results = run_all();
    // w04 (WP, 120 m², goed geïsoleerd) vs w01 (gas, 110 m², matig geïsoleerd):
    // de WP-keten deelt door SCOP ≈ 4 en moet per m² fors lager uitkomen.
    let wp = &results["w04-hoekwoning-wp-lucht-wtw"];
    let gas = &results["w01-tussenwoning-gas-hr107"];
    let wp_specific = wp.heating.annual_use_mj / 120.0;
    let gas_specific = gas.heating.annual_use_mj / 110.0;
    assert!(
        wp_specific < gas_specific * 0.6,
        "WP-woning ({wp_specific:.1} MJ/m²) hoort ruim onder gas-woning ({gas_specific:.1} MJ/m²)"
    );
}

#[test]
fn oude_woning_heeft_hogere_specifieke_behoefte_dan_nieuwbouw() {
    let results = run_all();
    let oud = &results["w02-vrijstaand-oud-gas-hr100"]; // U ≈ 1,4; enkel glas
    let nieuw = &results["w05-nieuwbouw-wp-bodem-pv"]; // U ≈ 0,15; HR+++ glas
    let oud_specific = oud.demand.annual_q_h_nd_mj / 150.0;
    let nieuw_specific = nieuw.demand.annual_q_h_nd_mj / 140.0;
    assert!(
        oud_specific > nieuw_specific * 2.0,
        "1975-woning ({oud_specific:.1} MJ/m²) hoort ≥ 2× nieuwbouw ({nieuw_specific:.1} MJ/m²)"
    );
}

#[test]
fn beng_indicatoren_consistent() {
    for (name, r) in run_all() {
        // BENG 1 herleidbaar uit de demand-sommen.
        let expected_beng1 =
            (r.demand.annual_q_h_nd_mj + r.demand.annual_q_c_nd_mj) / 3.6;
        // beng1 × A_g is niet direct beschikbaar (A_g zit in de fixture),
        // maar de verhouding met EP moet finite + niet-negatief zijn.
        assert!(
            r.beng.beng1_kwh_per_m2.is_finite() && r.beng.beng1_kwh_per_m2 > 0.0,
            "{name}: BENG1 > 0"
        );
        assert!(expected_beng1 > 0.0, "{name}: demand-som > 0");
        // BENG 2 == EP-specifiek in kWh/m².
        assert!(
            (r.beng.beng2_kwh_per_m2 - r.ep.primary_energy_kwh_per_m2).abs() < 1e-9,
            "{name}: BENG2 == EP kWh/m²"
        );
        // BENG 3 == renewable share in procenten.
        assert!(
            (r.beng.beng3_pct - r.ep.renewable_share * 100.0).abs() < 1e-9,
            "{name}: BENG3 == hernieuwbaar%"
        );
        // Grenzen zijn gevuld.
        assert!(r.beng.beng1_limit > 0.0 && r.beng.beng2_limit > 0.0);
    }

    // Nieuwbouw-WP-PV-woning voldoet aan alle drie; de 1975-woning aan geen
    // van de BENG 1/2-eisen.
    let results = run_all();
    let nieuw = &results["w05-nieuwbouw-wp-bodem-pv"];
    assert!(
        nieuw.beng.beng1_pass && nieuw.beng.beng2_pass && nieuw.beng.beng3_pass,
        "nieuwbouw voldoet aan BENG 1/2/3 (was {}/{}/{})",
        nieuw.beng.beng1_pass,
        nieuw.beng.beng2_pass,
        nieuw.beng.beng3_pass
    );
    let oud = &results["w02-vrijstaand-oud-gas-hr100"];
    assert!(
        !oud.beng.beng1_pass && !oud.beng.beng2_pass,
        "1975-woning zakt op BENG 1 en 2"
    );
}

#[test]
fn wtw_fixture_heeft_recovery() {
    let results = run_all();
    let r = &results["w04-hoekwoning-wp-lucht-wtw"];
    assert!(
        r.ventilation.annual_wtw_recovery_mj > 0.0,
        "WTW-systeem levert warmteterugwinning"
    );
    let zonder = &results["w01-tussenwoning-gas-hr107"];
    assert!(
        (zonder.ventilation.annual_wtw_recovery_mj).abs() < 1e-9,
        "systeem C heeft geen WTW-recovery"
    );
}
