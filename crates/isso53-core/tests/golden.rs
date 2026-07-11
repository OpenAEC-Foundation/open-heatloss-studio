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
// ACTIEF (M4a+M4b, 2026-07-10/11). `voorbeeld_62_input.json` is een GETROUWE
// transcriptie van het volledig uitgewerkte beganegrond-tussenmoduul
// (PDF-index 63-65); `voorbeeld_62_expected.json` bevat de gepubliceerde
// waarden mét bronverwijzingen.
//
// Twee engine-gaten uit de eerdere blokkade zijn opgelost:
//   M4a: `calculate_h_t_adjacent_rooms` (transmission.rs) honoreert nu een
//        expliciete `temperature_factor` DIRECT als f_ia,k (voorrang boven de
//        ΔT-afleiding) — analoog aan het bestaande `Unheated`-pad. Dekt de
//        gepubliceerde plafond-fiak=0,105 naar een gelijk-temperatuur
//        bovenmoduul, die niet uit een temperatuurverschil volgt.
//        Φ_T: 389,7 → 525,65 W (publicatie 525,0 W, +0,12%).
//   M4b: `Room.ventilation_q_v_established` bestond al in het model en werd
//        al direct gebruikt in `calculate_ventilation_flow_rate` — de fix was
//        uitsluitend het invullen van dat fixture-veld (qv=100 m³/h =
//        0,027778 m³/s) i.p.v. een engine-wijziging.
//        Φ_vent: 88,9 → 190,00 W (publicatie 190,0 W, +0,001%).
//
// Φ_i matchte al vóór deze fix (245,8 vs 246 W, -0,1%).
//
// gap_3 (Φ_hu / totaal) blijft BEWUST ONGEVALIDEERD — geen engine-bug maar
// een interne inconsistentie in de PUBLICATIE zelf: Φ_op = 5,8×3,5×28 = 568 W
// gebruikt de hart-op-hart moduulvloer (20,3 m²), terwijl elke andere term
// (ventilatie, grond, Φ_T) de inwendige 18,7 m² gebruikt — hetzelfde
// `floorArea`-veld kan beide niet eren. Met floorArea=18,7 m² (norm-conforme,
// inwendige maat — consistent met alle andere termen) geeft de engine
// Φ_hu = 18,7×28 − 6,672×28,5 = 523,6 − 190,0 = 333,6 W, tegenover de
// gepubliceerde 378 W (-11,7%, ruim buiten tolerantie). Fudgen van de
// expected-waarde of een tweede area-veld toevoegen om dit te verbergen is
// NIET gedaan (PM-instructie + eerder precedent bij voorbeeld_61). Φ_hu en
// totalBuildingHeatLoss/totalHeatLoss staan daarom op `null` in
// `voorbeeld_62_expected.json` (Option-velden — de close()-checks worden
// hierdoor overgeslagen) en zijn gedocumenteerd in `_gaps.gap_3`.
#[test]
fn voorbeeld_62() {
    let input = include_str!("fixtures/voorbeeld_62_input.json");
    let expected: Expected =
        serde_json::from_str(include_str!("fixtures/voorbeeld_62_expected.json"))
        .expect("Failed to parse expected results");

    let result_json = calculate_from_json(input).expect("Calculation failed");
    let result: serde_json::Value =
        serde_json::from_str(&result_json).expect("Failed to parse result JSON");

    // Building-level: alleen de drie opgeloste termen. totalBuildingHeatLoss
    // is bewust `null` in de fixture (bevat gap_3's Φ_hu) → check wordt door
    // de `if let Some` overgeslagen.
    if let Some(want) = expected.summary.total_building_heat_loss {
        let got = result["summary"]["totalBuildingHeatLoss"].as_f64()
            .expect("totalBuildingHeatLoss missing");
        close("totalBuildingHeatLoss", got, want, expected.tolerance_pct);
    }
    if let Some(want) = expected.summary.total_transmission_loss {
        let got = result["summary"]["totalTransmissionLoss"].as_f64()
            .expect("totalTransmissionLoss missing");
        close("totalTransmissionLoss", got, want, expected.tolerance_pct);
    }
    if let Some(want) = expected.summary.total_ventilation_loss {
        let got = result["summary"]["totalVentilationLoss"].as_f64()
            .expect("totalVentilationLoss missing");
        close("totalVentilationLoss", got, want, expected.tolerance_pct);
    }
    if let Some(want) = expected.summary.total_infiltration_loss {
        let got = result["summary"]["totalInfiltrationLoss"].as_f64()
            .expect("totalInfiltrationLoss missing");
        close("totalInfiltrationLoss", got, want, expected.tolerance_pct);
    }

    for expected_room in &expected.rooms {
        let room = result["rooms"]
            .as_array()
            .expect("rooms is not an array")
            .iter()
            .find(|r| r["roomId"].as_str() == Some(&expected_room.room_id))
            .unwrap_or_else(|| panic!("Room {} not found in results", expected_room.room_id));

        // Φ_T, Φ_vent, Φ_i per ruimte — de drie termen die de publicatie
        // zonder interne tegenspraak reproduceert.
        if let Some(want) = expected_room.phi_t {
            let got = room["phiT"].as_f64()
                .unwrap_or_else(|| panic!("phiT missing for room {}", expected_room.room_id));
            close(&format!("Room {} phiT", expected_room.room_id), got, want, 2.0);
        }
        if let Some(want) = expected_room.phi_v {
            let got = room["phiV"].as_f64()
                .unwrap_or_else(|| panic!("phiV missing for room {}", expected_room.room_id));
            close(&format!("Room {} phiV", expected_room.room_id), got, want, 2.0);
        }
        if let Some(want) = expected_room.phi_i {
            let got = room["phiI"].as_f64()
                .unwrap_or_else(|| panic!("phiI missing for room {}", expected_room.room_id));
            close(&format!("Room {} phiI", expected_room.room_id), got, want, 2.0);
        }

        // totalHeatLoss is bewust `null` in de fixture (bevat gap_3's Φ_hu) →
        // check wordt overgeslagen; zie doc-comment boven deze test.
        if let Some(want) = expected_room.total_heat_loss {
            let got = room["totalHeatLoss"].as_f64()
                .unwrap_or_else(|| panic!("totalHeatLoss missing for room {}", expected_room.room_id));
            close(&format!("Room {} totalHeatLoss", expected_room.room_id), got, want, 2.0);
        }
    }
}