//! Golden tests against ISSO 53 norm examples
//!
//! These tests verify that our implementation matches the expected results
//! from the official ISSO 53 publication examples (pages 59-75).

use isso53_core::calculate_from_json;
use serde::{Deserialize, Serialize};

/// Expected result format for fixtures
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    tolerance_pct: f64,
    summary: ExpectedSummary,
    rooms: Vec<ExpectedRoom>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedSummary {
    total_building_heat_loss: Option<f64>,
    shell_heat_loss: Option<f64>,
    total_transmission_loss: Option<f64>,
    total_ventilation_loss: Option<f64>,
    total_infiltration_loss: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedRoom {
    room_id: String,
    phi_t: Option<f64>,
    phi_v: Option<f64>,
    phi_i: Option<f64>,
    total_heat_loss: Option<f64>,
}

/// Assert that two floats are close within a tolerance percentage
fn close(label: &str, got: f64, want: f64, tolerance_pct: f64) {
    let diff_pct = ((got - want) / want).abs() * 100.0;
    assert!(
        diff_pct < tolerance_pct,
        "{}: got {:.1}, want {:.1} (diff {:.1}% > {:.1}%)",
        label, got, want, diff_pct, tolerance_pct
    );
}

/// Test against ISSO 53 voorbeeld 6.1 (schilberekening kantoorgebouw 50x20x21 m)
//
// BLOCKED (blijft #[ignore]). De engine HEEFT een schilmethode
// (`calc::shell::calculate_shell` → `summary.shellHeatLoss`), maar het
// gepubliceerde voorbeeld is niet 1-op-1 reproduceerbaar door twee
// publicatie-anomalieën die in `voorbeeld_61_expected.json`
// (`_gepubliceerde_tussenwaarden`) staan gedocumenteerd:
//   1. De gepubliceerde ΣH_T,ie (2452 W/K) telt ALLEEN de gevels — het dak
//      (Rc=6, 1000 m²) ontbreekt bewust in de norm-som. Een engine die het
//      dak wél meerekent komt hoger uit dan de 236,1 kW.
//   2. θ_e is tijdconstante-afgeleid (τ=84,3 h → θ_e=-9,5 °C); de engine
//      neemt θ_e als directe input i.p.v. het uit τ te herleiden.
// Bovendien modelleert `voorbeeld_61_input.json` nog een enkel 20 m²-vertrek
// i.p.v. de gebouwschil — een input-rebuild (schil als één "room" met alle
// envelop-constructies, dak bewust weggelaten conform de publicatie) is een
// apart werkpakket. Zie tests/PDF_GAPS.md §6.1.
#[test]
#[ignore]
fn voorbeeld_61() {
    let input = include_str!("fixtures/voorbeeld_61_input.json");
    let expected: Expected =
        serde_json::from_str(include_str!("fixtures/voorbeeld_61_expected.json"))
        .expect("Failed to parse expected results");

    let result_json = calculate_from_json(input).expect("Calculation failed");
    let result: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    // Check building-level summary values
    if let Some(want) = expected.summary.total_building_heat_loss {
        let got = result["summary"]["totalBuildingHeatLoss"].as_f64()
            .expect("totalBuildingHeatLoss missing");
        close("totalBuildingHeatLoss", got, want, expected.tolerance_pct);
    }

    if let Some(want) = expected.summary.shell_heat_loss {
        let got = result["summary"]["shellHeatLoss"].as_f64()
            .expect("shellHeatLoss missing");
        close("shellHeatLoss", got, want, expected.tolerance_pct);
    }

    if let Some(want) = expected.summary.total_transmission_loss {
        let got = result["summary"]["totalTransmissionLoss"].as_f64()
            .expect("totalTransmissionLoss missing");
        close("totalTransmissionLoss", got, want, expected.tolerance_pct);
    }

    // Check per-room values
    for expected_room in &expected.rooms {
        let room = result["rooms"]
            .as_array()
            .expect("rooms is not an array")
            .iter()
            .find(|r| r["roomId"].as_str() == Some(&expected_room.room_id))
            .unwrap_or_else(|| panic!("Room {} not found in results", expected_room.room_id));

        if let Some(want) = expected_room.phi_t {
            let got = room["phiT"].as_f64()
                .unwrap_or_else(|| panic!("phiT missing for room {}", expected_room.room_id));
            close(&format!("Room {} phiT", expected_room.room_id), got, want, 2.0); // 2% tolerance per room
        }

        if let Some(want) = expected_room.total_heat_loss {
            let got = room["totalHeatLoss"].as_f64()
                .unwrap_or_else(|| panic!("totalHeatLoss missing for room {}", expected_room.room_id));
            close(&format!("Room {} totalHeatLoss", expected_room.room_id), got, want, 2.0);
        }
    }
}

/// Test against ISSO 53 voorbeeld 6.2 (gedetailleerde methode — beganegrond tussenmoduul)
//
// BLOCKED (blijft #[ignore]). `voorbeeld_62_input.json` is nu een GETROUWE
// transcriptie van het volledig uitgewerkte beganegrond-tussenmoduul
// (PDF-index 63-65) en `voorbeeld_62_expected.json` bevat de gepubliceerde
// waarden (Φ_T 525 W, Φ_i 246 W, Φ_vent 190 W, Φ_hu 378 W, totaal 1339 W)
// mét bronverwijzingen. De engine reproduceert dit moduul echter NIET binnen
// tolerantie (empirische run 2026-07-02: Φ_T 389,7 / Φ_vent 88,9 / Φ_hu 434,7
// / totaal 1159 W; alleen Φ_i matcht op -0,1%). Drie structurele engine-gaten,
// gedocumenteerd in `voorbeeld_62_expected.json` (`_gaps`):
//   gap_1: plafond-fiak=0,105 (tussenvloer naar gelijk-temp moduul) wordt
//          niet gehonoreerd — engine negeert temperature_factor op
//          boundaryType=adjacentRoom → H_T,ia=0 i.p.v. 4,77 W/K.
//   gap_2: geen per-ruimte ventilatievolume-override in het isso53 room-model;
//          engine valt terug op Bbl-minimum i.p.v. de gegeven qv=100 m³/h.
//   gap_3: Φ_hu stapelt gap_2 + een publicatie-interne area-inconsistentie
//          (Φ_op op 20,3 m² hart-op-hart vs 18,7 m² inwendig elders).
// Activeren pas na engine-fixes gap_1 + gap_2 (zie _gaps.activatie_voorwaarde).
#[test]
#[ignore]
fn voorbeeld_62() {
    let input = include_str!("fixtures/voorbeeld_62_input.json");
    let expected: Expected =
        serde_json::from_str(include_str!("fixtures/voorbeeld_62_expected.json"))
        .expect("Failed to parse expected results");

    let result_json = calculate_from_json(input).expect("Calculation failed");
    let result: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    // Similar checks as voorbeeld_61
    if let Some(want) = expected.summary.total_building_heat_loss {
        let got = result["summary"]["totalBuildingHeatLoss"].as_f64()
            .expect("totalBuildingHeatLoss missing");
        close("totalBuildingHeatLoss", got, want, expected.tolerance_pct);
    }

    for expected_room in &expected.rooms {
        let room = result["rooms"]
            .as_array()
            .expect("rooms is not an array")
            .iter()
            .find(|r| r["roomId"].as_str() == Some(&expected_room.room_id))
            .unwrap_or_else(|| panic!("Room {} not found in results", expected_room.room_id));

        if let Some(want) = expected_room.total_heat_loss {
            let got = room["totalHeatLoss"].as_f64()
                .unwrap_or_else(|| panic!("totalHeatLoss missing for room {}", expected_room.room_id));
            close(&format!("Room {} totalHeatLoss", expected_room.room_id), got, want, 2.0);
        }
    }
}