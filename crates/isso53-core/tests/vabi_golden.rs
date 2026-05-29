//! Vabi-referentie verificatietests voor isso53-core.
//!
//! Bron: Vabi Elements 3.11.2.23 rapport TR02 - Houtfabriek (29-11-2024).
//! Bedrijfsruimte4 - 16p, ISSO 53 Industriefunctie/Verblijfsgebied.

use isso53_core::calculate_from_json;

fn close(label: &str, got: f64, want: f64, tol_pct: f64) {
    // Speciale case: want=0 → eis absolute tolerantie 1 W (i.p.v. delen door 0).
    if want.abs() < f64::EPSILON {
        assert!(
            got.abs() < 1.0,
            "{label}: got {got:.2}, want 0 (>1 W absolute tolerantie)"
        );
        return;
    }
    let diff = ((got - want) / want).abs() * 100.0;
    assert!(
        diff < tol_pct,
        "{label}: got {got:.0}, want {want:.0} ({diff:.1}% > {tol_pct}%)"
    );
}

fn load_result() -> (serde_json::Value, serde_json::Value) {
    let input = include_str!(
        "../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/input.json"
    );
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/expected.json"
    ))
    .unwrap();
    let result_json = calculate_from_json(input).expect("calc");
    let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
    (result, expected)
}

/// Φ_V + Φ_I matcht Vabi binnen 10% na fixes:
/// - §4.6 embedded heating (sessie 1)
/// - Formule 4.38 WTW f_v omkering (sessie 2)
/// - A_u/A_g omdraai in formule 4.28/4.29 (sessie 2 vervolg)
/// - Building.building_height-veld voor q_is-lookup (sessie 2 vervolg)
/// - Fixture: supplyTemperature=21°C (luchtverwarming) + buildingHeight=10,9 m
#[test]
fn vabi_bedrijfsruimte4_phi_vi_combined_matches() {
    let (result, _) = load_result();
    let room = &result["rooms"][0];
    let phi_vi = room["phiV"].as_f64().unwrap() + room["phiI"].as_f64().unwrap();
    close("phiV+phiI", phi_vi, 3080.0, 10.0);
}

/// Verifieert dat opwarmtoeslag binnen 5% blijft.
/// Tautologisch (P=10 W/m² is direct uit Vabi overgenomen omdat onze P-tabel
/// nog niet uit PDF p.51-53 is ingelezen) — maar bevestigt de
/// Φ_hu = P × A_floor formule.
#[test]
fn vabi_bedrijfsruimte4_phi_hu_matches() {
    let (result, expected) = load_result();
    let room = &result["rooms"][0];

    close(
        "phiHu",
        room["phiHu"].as_f64().unwrap(),
        expected["room"]["phiHu"].as_f64().unwrap(),
        5.0,
    );
}

/// Transmissie test — na sessie 14 fixture-decompositie (spoor 4 gesloten).
///
/// **Status sessie 14 (2026-05-29):** fixture-bundel `bundel-binnenwanden` (30 m² @U=0,40 @7°C)
/// vervangen door 25 individuele Vabi-elementen uit PDF p.18-20. Calc 3025 W vs Vabi 2919 W
/// = +3,6%. Binnen 5% tolerance. Restgap komt uit Vabi's verwarmd-plafond convention (corr=0
/// voor verwarmde buurruimtes met 18°C; onze norm-strikte calc geeft f_ia,k=2/29=0,069 → ~95W
/// over 99,74 m² verwarmd-plafond elementen). Accepteer als documented Vabi-anomaly.
#[test]
fn vabi_bedrijfsruimte4_phi_t_matches() {
    let (result, expected) = load_result();
    let room = &result["rooms"][0];

    close(
        "phiT",
        room["phiT"].as_f64().unwrap(),
        expected["room"]["phiT"].as_f64().unwrap(),
        5.0,
    );
}

/// Snapshot van werkelijke waarden voor regressie-detectie. Faalt als de
/// rekenkern wijzigt zónder dat we het verwachten — onafhankelijk van of de
/// waarden Vabi matchen.
#[test]
fn vabi_bedrijfsruimte4_snapshot() {
    let (result, _) = load_result();
    let room = &result["rooms"][0];

    let phi_t = room["phiT"].as_f64().unwrap();
    let phi_v = room["phiV"].as_f64().unwrap();
    let phi_i = room["phiI"].as_f64().unwrap_or(0.0);
    let phi_hu = room["phiHu"].as_f64().unwrap();
    let total = room["totalHeatLoss"].as_f64().unwrap();


    // Snapshot sessie 14 (2026-05-29): phi_T herzien van 2737→3025 na fixture-decompositie
    // (bundel-binnenwanden → 25 individuele Vabi-elementen). totalHeatLoss herzien daarmee.
    close("phiT", phi_t, 3025.0, 2.0);
    close("phiV", phi_v, 0.0, 1.0);     // luchtverwarming θ_t=21°C → f_v=0
    close("phiI", phi_i, 3134.0, 2.0);
    close("phiHu", phi_hu, 2163.0, 2.0);
    close("totalHeatLoss", total, 8322.0, 2.0);
}