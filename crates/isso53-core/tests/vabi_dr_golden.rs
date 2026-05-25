//! Vabi DR Engineering voorbeeld — cross-project verificatie Unknown-pad.
//!
//! Bron: Vabi Elements 3.12.0.127, "Voorbeeld Warmteverliesberekening Utiliteitsbouw"
//! (27-2-2025), ruimte 0.03 Kantoor West. Berekend volgens ISSO 53 met `Methode q_v;10 =
//! Forfaitair` (Unknown-pad, formule 4.31).
//!
//! Doel: cross-validatie van (a) de §4.6 ground-fix, (b) formule 4.38 WTW omkering,
//! (c) A_u/A_g + Building.building_height, (d) nieuwe Unknown-pad implementatie.
//!
//! Status (sessie 2026-05-23): Φ_V matcht exact (luchtverwarming → f_v=0). Φ_T en Φ_I
//! wijken buiten 10% af door norm-vs-Vabi keten-verschillen — gedocumenteerd in
//! PDF_GAPS.md en beide bewust op #[ignore].

use isso53_core::calculate_from_json;

fn close(label: &str, got: f64, want: f64, tol_pct: f64) {
    if want.abs() < f64::EPSILON {
        assert!(got.abs() < 1.0, "{label}: got {got:.2}, want 0");
        return;
    }
    let diff = ((got - want) / want).abs() * 100.0;
    assert!(diff < tol_pct, "{label}: got {got:.0}, want {want:.0} ({diff:.1}% > {tol_pct}%)");
}

fn load_room_0_03() -> serde_json::Value {
    let input = include_str!("fixtures/vabi_dr_engineering_kantoorwest_input.json");
    let result_json = calculate_from_json(input).expect("calc");
    let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
    let rooms = result["rooms"].as_array().unwrap();
    rooms.iter().find(|r| r["roomId"] == "0.03").unwrap().clone()
}

/// Φ_V = 0 W door luchtverwarming (θ_t = 21,5°C > θ_i = 21,5°C → f_v=0, formule 4.38).
#[test]
fn vabi_dr_kantoorwest_phi_v_zero() {
    let room = load_room_0_03();
    close("phiV", room["phiV"].as_f64().unwrap(), 0.0, 1.0);
}

/// Snapshot van werkelijke waarden voor regressie-detectie.
/// Φ_T = 3165 (was 4672 in s7; dubbeltelling adjacent-room weggewerkt in sessie 8 Optie C).
/// Φ_I = ~693 (Vabi-compat via UnknownVabiCompat variant).
#[test]
fn vabi_dr_kantoorwest_snapshot() {
    let room = load_room_0_03();
    close("phiT", room["phiT"].as_f64().unwrap(), 3165.0, 3.0);
    close("phiV", room["phiV"].as_f64().unwrap(), 0.0, 1.0);
    close("phiI", room["phiI"].as_f64().unwrap(), 693.0, 5.0); // UnknownVabiCompat
    close("totalHeatLoss", room["totalHeatLoss"].as_f64().unwrap(), 3858.0, 5.0); // 3165 + 693
}

/// Cross-validatie Φ_T — heractiveerd sessie 8 na Optie C wrapper-schrap.
///
/// Vabi: Φ_T,ie=1237 + Φ_T,ia=1507 + Φ_T,ig=315 = 3059 W.
/// Onze code na fix: 3165 W (+3,5%). Binnen 10% tolerantie.
///
/// **Sessie 8 fix:** `room_load.rs` had een wrapper `calculate_transmission_with_adjacent_rooms`
/// die de adjacent-room bijdrage een tweede keer optelde op `phi_t` bovenop wat transmission.rs
/// al berekende via `calculate_h_t_adjacent_rooms` (toegevoegd door sessie 7 C1 fix). Door de
/// wrapper te schrappen en lookup-pad te migreren naar transmission.rs (single source of truth)
/// verdwijnt de dubbeltelling. Fixture U=2,91 voor plafond-tussenvloer (Rc=0,14) is correct —
/// bevestigd in DR Engineering bron `tests/references/dr-engineering-samenvatting.md` r121.
#[test]
fn vabi_dr_kantoorwest_phi_t_matches() {
    let room = load_room_0_03();
    close("phiT", room["phiT"].as_f64().unwrap(), 3059.0, 10.0);
}

/// Cross-validatie Φ_I — nu binnen 5% tolerantie door UnknownVabiCompat.
/// Gebruikt NEN 8088-1 (f_type=0,9, f_inf=1,10) + NTA 8800 (f_jaar=0,7) + power-law (Δp/10)^0.67.
/// Vabi: 681 W, verwacht: ~693 W (+1,8%).
#[test]
fn vabi_dr_kantoorwest_phi_i_matches() {
    let room = load_room_0_03();
    close("phiI", room["phiI"].as_f64().unwrap(), 681.0, 5.0);
}
