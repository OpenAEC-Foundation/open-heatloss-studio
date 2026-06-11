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
// BLOCKED: expected-values zijn nu de ECHTE normwaarden (PDF p.59-60, zie
// `_source` in voorbeeld_61_expected.json), maar voorbeeld_61_input.json is
// geen transcriptie van par. 6.1 — het modelleert een enkel 20 m2 vertrek
// i.p.v. het gebouw uit de schilberekening (engine ~1,6 kW vs norm 236,1 kW).
// Activeren kan pas na een input-rebuild. Zie tests/PDF_GAPS.md.
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

/// Test against ISSO 53 voorbeeld 6.2 (extended example)
// BLOCKED: expected-values zijn placeholders. Zie tests/PDF_GAPS.md.
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