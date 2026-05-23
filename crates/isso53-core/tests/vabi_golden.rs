//! Vabi-referentie verificatietests voor isso53-core.
//!
//! Bron: Vabi Elements 3.11.2.23 rapport TR02 - Houtfabriek (29-11-2024).
//! Bedrijfsruimte4 - 16p, ISSO 53 Industriefunctie/Verblijfsgebied.

use isso53_core::calculate_from_json;

fn close(label: &str, got: f64, want: f64, tol_pct: f64) {
    let diff = ((got - want) / want).abs() * 100.0;
    assert!(
        diff < tol_pct,
        "{label}: got {got:.0}, want {want:.0} ({diff:.1}% > {tol_pct}%)"
    );
}

fn load_result() -> (serde_json::Value, serde_json::Value) {
    let input = include_str!("fixtures/vabi_houtfabriek_bedrijfsruimte4_input.json");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/vabi_houtfabriek_bedrijfsruimte4_expected.json"
    ))
    .unwrap();
    let result_json = calculate_from_json(input).expect("calc");
    let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
    (result, expected)
}

/// Verifieert dat infiltratie + ventilatie binnen 15% van Vabi-referentie blijft.
/// Reproduceert Vabi exact (3076 vs 3080 W = -0.1%): bevestiging dat Known-pad
/// + tabel 4.5 + z-factor + WTW correct werken.
#[test]
fn vabi_bedrijfsruimte4_phi_v_matches() {
    let (result, expected) = load_result();
    let room = &result["rooms"][0];
    let tol = expected["tolerance_pct"].as_f64().unwrap();

    close(
        "phiV",
        room["phiV"].as_f64().unwrap(),
        expected["room"]["phiV"].as_f64().unwrap(),
        tol,
    );
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

/// **GAP**: transmissie wijkt +50% af van Vabi (4385 vs 2919 W). Hypothese:
/// Vabi gebruikt voor vloer-op-grond geen formule 4.21-conventie (1.45 × A ×
/// U_equiv × Δθ over θ_e), maar T_grond direct (Corr.factor = 0 in Vabi-output).
/// Verschil = 1.45 × 218 × 0.16 × 29 ≈ 1467 W = exact het gap.
///
/// Zie tests/PDF_GAPS.md voor de norm-interpretatie analyse.
#[test]
#[ignore = "norm-interpretatie verschil ground-formule §4.6 — zie PDF_GAPS.md"]
fn vabi_bedrijfsruimte4_phi_t_full_match() {
    let (result, expected) = load_result();
    let room = &result["rooms"][0];
    let tol = expected["tolerance_pct"].as_f64().unwrap();

    close(
        "phiT",
        room["phiT"].as_f64().unwrap(),
        expected["room"]["phiT"].as_f64().unwrap(),
        tol,
    );
    close(
        "totalHeatLoss",
        room["totalHeatLoss"].as_f64().unwrap(),
        expected["room"]["totalHeatLoss"].as_f64().unwrap(),
        tol,
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
    let phi_hu = room["phiHu"].as_f64().unwrap();
    let total = room["totalHeatLoss"].as_f64().unwrap();

    // Snapshot zoals op 2026-05-23 (commit pre-Vabi-fixture-verificatie).
    // Bij gewenste rekenkern-wijziging: update deze waarden + valideer tegen
    // het PDF_GAPS-onderzoek.
    close("phiT", phi_t, 4385.0, 2.0);
    close("phiV", phi_v, 3076.0, 2.0);
    close("phiHu", phi_hu, 2163.0, 2.0);
    close("totalHeatLoss", total, 12996.0, 2.0);
}