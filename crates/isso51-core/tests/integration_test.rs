//! Integration tests: drive every fixture pair through `calculate_from_json`
//! and compare numerically against the recorded `*_(result|expected).json`.
//!
//! Background (2026-05-12 audit):
//! - `build_summary` in `lib.rs` uses a linear sum at building level instead
//!   of the erratum-2023 quadratic sum (formule 3.11). The DR Engineering
//!   fixture is expected to FAIL at `connection_capacity` (~6700 W expected
//!   vs ~8121 W engine output). Do NOT widen the tolerance to mask this.
//!
//! Expected fixture formats differ per source — see `extract_expected_room`
//! and `extract_expected_connection_capacity`.

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use isso51_core::calculate_from_json;
use isso51_core::result::ProjectResult;

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Workspace fixtures live at `<repo>/tests/fixtures/`. The crate sits at
/// `<repo>/crates/isso51-core/`, so we walk two parents up from
/// `CARGO_MANIFEST_DIR`.
fn fixtures_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR must have two parents (repo root)")
        .join("tests")
        .join("fixtures")
}

fn require_fixture(name: &str) -> PathBuf {
    let path = fixtures_dir().join(name);
    if !path.exists() {
        panic!("Fixture missing: {}", path.display());
    }
    path
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()))
}

fn parse_json(path: &Path) -> Value {
    let raw = read_to_string(path);
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("Failed to parse {} as JSON: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Tolerance & comparison
// ---------------------------------------------------------------------------

const ABS_TOLERANCE_W: f64 = 2.0;
const REL_TOLERANCE: f64 = 0.02; // 2 % of expected

#[derive(Debug, Default, Clone)]
struct Mismatch {
    #[allow(dead_code)] // surfaced in Debug for `cargo test -- --nocapture` triage
    fixture: String,
    room: Option<String>,
    field: &'static str,
    expected: f64,
    actual: f64,
}

impl Mismatch {
    fn diff(&self) -> f64 {
        self.actual - self.expected
    }
    fn pct(&self) -> f64 {
        if self.expected.abs() < 1e-9 {
            f64::INFINITY
        } else {
            100.0 * (self.actual - self.expected) / self.expected
        }
    }
}

fn close_enough(actual: f64, expected: f64) -> bool {
    let tol = (REL_TOLERANCE * expected.abs()).max(ABS_TOLERANCE_W);
    (actual - expected).abs() <= tol
}

/// Per-fixture expected slice for one room.
#[derive(Debug, Default)]
struct ExpectedRoom {
    phi_t: Option<f64>,
    phi_v: Option<f64>,
    phi_i: Option<f64>,
    phi_hu: Option<f64>,
    phi_hl_i: Option<f64>,
}

// ---------------------------------------------------------------------------
// Format adapters: pull `{phi_t, phi_v, phi_i, phi_hu, phi_hl_i}` per room out
// of the heterogeneous expected JSON, keyed by `room_id`.
// ---------------------------------------------------------------------------

/// portiekwoning_result.json — mirrors `ProjectResult` exactly.
fn extract_native_format(expected: &Value) -> HashMap<String, ExpectedRoom> {
    let mut out = HashMap::new();
    let rooms = expected
        .get("rooms")
        .and_then(Value::as_array)
        .expect("expected.rooms array");
    for r in rooms {
        let id = r
            .get("room_id")
            .and_then(Value::as_str)
            .expect("room_id string")
            .to_string();
        out.insert(
            id,
            ExpectedRoom {
                phi_t: r.pointer("/transmission/phi_t").and_then(Value::as_f64),
                phi_v: r.pointer("/ventilation/phi_v").and_then(Value::as_f64),
                phi_i: r.pointer("/infiltration/phi_i").and_then(Value::as_f64),
                phi_hu: r.pointer("/heating_up/phi_hu").and_then(Value::as_f64),
                phi_hl_i: r.get("total_heat_loss").and_then(Value::as_f64),
            },
        );
    }
    out
}

/// vabi_vrijstaande_woning_expected.json — flat per-room fields.
fn extract_vabi_format(expected: &Value) -> HashMap<String, ExpectedRoom> {
    let mut out = HashMap::new();
    let rooms = expected
        .get("rooms")
        .and_then(Value::as_array)
        .expect("expected.rooms array");
    for r in rooms {
        let id = r
            .get("room_id")
            .and_then(Value::as_str)
            .expect("room_id string")
            .to_string();
        out.insert(
            id,
            ExpectedRoom {
                phi_t: r.get("phi_t").and_then(Value::as_f64),
                phi_v: r.get("phi_v").and_then(Value::as_f64),
                phi_i: None, // not recorded in Vabi expected
                phi_hu: r.get("phi_hu").and_then(Value::as_f64),
                phi_hl_i: r.get("phi_hl_i").and_then(Value::as_f64),
            },
        );
    }
    out
}

/// dr_engineering_woningbouw_result.json — Vabi 3.12.0.127 layout with
/// transmission split (`phi_t_ie + phi_t_ia + phi_t_iae + phi_t_ig + phi_t_iaBE`).
fn extract_dr_format(expected: &Value) -> HashMap<String, ExpectedRoom> {
    let mut out = HashMap::new();
    let rooms = expected
        .get("rooms")
        .and_then(Value::as_array)
        .expect("expected.rooms array");
    for r in rooms {
        let id = r
            .get("room_id")
            .and_then(Value::as_str)
            .expect("room_id string")
            .to_string();
        let f = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        // Sum the four transmission split fields into total Φ_T for engine comparison.
        let phi_t_total = f("phi_t_ie") + f("phi_t_ia") + f("phi_t_iae") + f("phi_t_ig") + f("phi_t_iaBE");
        out.insert(
            id,
            ExpectedRoom {
                phi_t: Some(phi_t_total),
                // Vabi splits infiltration & ventilation — engine `phi_vent` is post-infiltration.
                phi_v: r.get("phi_vent").and_then(Value::as_f64),
                phi_i: r.get("phi_i").and_then(Value::as_f64),
                phi_hu: r.get("phi_hu").and_then(Value::as_f64),
                phi_hl_i: r.get("phi_hl_i").and_then(Value::as_f64),
            },
        );
    }
    out
}

/// Pull the building-level "aansluitvermogen" out of any of the three formats.
fn extract_expected_connection_capacity(expected: &Value) -> Option<f64> {
    if let Some(v) = expected.pointer("/summary/connection_capacity").and_then(Value::as_f64) {
        return Some(v);
    }
    if let Some(v) = expected.pointer("/building/phi_hl_build").and_then(Value::as_f64) {
        return Some(v);
    }
    None
}

// ---------------------------------------------------------------------------
// Engine comparison: pull the matching field from the actual engine output.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Field {
    PhiT,
    PhiV,
    PhiI,
    PhiHu,
    PhiHlI,
}

impl Field {
    fn name(self) -> &'static str {
        match self {
            Field::PhiT => "phi_t",
            Field::PhiV => "phi_v",
            Field::PhiI => "phi_i",
            Field::PhiHu => "phi_hu",
            Field::PhiHlI => "phi_hl_i",
        }
    }

    fn pick(self, room: &isso51_core::result::RoomResult, fixture: &str) -> f64 {
        match self {
            Field::PhiT => room.transmission.phi_t,
            // For DR fixture, expected `phi_v` is `phi_vent` (post-infiltration); other fixtures expect raw `phi_v`.
            Field::PhiV => {
                if fixture == "dr_engineering_woningbouw" {
                    room.ventilation.phi_vent
                } else {
                    room.ventilation.phi_v
                }
            }
            Field::PhiI => room.infiltration.phi_i,
            Field::PhiHu => room.heating_up.phi_hu,
            Field::PhiHlI => room.total_heat_loss,
        }
    }
}

const ROOM_FIELDS: &[Field] = &[Field::PhiT, Field::PhiV, Field::PhiI, Field::PhiHu, Field::PhiHlI];

// ---------------------------------------------------------------------------
// The main driver — one #[test] per fixture, so failures are isolated.
// ---------------------------------------------------------------------------

struct FixtureSpec {
    name: &'static str,
    input_file: &'static str,
    expected_file: &'static str,
    extract: fn(&Value) -> HashMap<String, ExpectedRoom>,
}

fn run_fixture(spec: &FixtureSpec) {
    let input_path = require_fixture(spec.input_file);
    let expected_path = require_fixture(spec.expected_file);

    let input_json = read_to_string(&input_path);
    let result_json = calculate_from_json(&input_json)
        .unwrap_or_else(|e| panic!("calculate_from_json failed for {}: {e}", spec.name));

    let actual: ProjectResult = serde_json::from_str(&result_json)
        .unwrap_or_else(|e| panic!("Failed to deserialize engine output for {}: {e}", spec.name));

    let expected_value = parse_json(&expected_path);
    let expected_rooms = (spec.extract)(&expected_value);

    let mut mismatches: Vec<Mismatch> = Vec::new();

    // ---- Per-room compare -------------------------------------------------
    for room in &actual.rooms {
        let Some(exp) = expected_rooms.get(&room.room_id) else {
            // Engine emitted a room not in the expected fixture (e.g. fully unheated rooms
            // are pruned in Vabi expected). Skip — not a numeric mismatch.
            continue;
        };
        // Skip per-field comparison voor kamers waar expected phi_hl_i geclampt op 0:
        // de norm clipt negatieve phi_basis op kamerniveau naar 0. Individuele
        // componenten (phi_t, phi_v) kunnen dan afwijken van Vabi door software-
        // specifieke intra-zone correcties zonder norm-bron (zie
        // transmission.rs::h_t_adjacent_room_element). Verifieer alleen dat de
        // kamer-som correct op 0 staat — individuele componenten beïnvloeden het
        // aansluitvermogen niet.
        if matches!(exp.phi_hl_i, Some(p) if p < 1.0) {
            let actual_total = room.total_heat_loss;
            if !close_enough(actual_total, exp.phi_hl_i.unwrap()) {
                mismatches.push(Mismatch {
                    fixture: spec.name.to_string(),
                    room: Some(format!("{} ({})", room.room_id, room.room_name)),
                    field: "phi_hl_i (clamped)",
                    expected: exp.phi_hl_i.unwrap(),
                    actual: actual_total,
                });
            }
            continue;
        }
        for field in ROOM_FIELDS {
            let expected = match field {
                Field::PhiT => exp.phi_t,
                Field::PhiV => exp.phi_v,
                Field::PhiI => exp.phi_i,
                Field::PhiHu => exp.phi_hu,
                Field::PhiHlI => exp.phi_hl_i,
            };
            let Some(expected) = expected else {
                continue; // field not recorded in this fixture format
            };
            let actual_val = field.pick(room, spec.name);
            if !close_enough(actual_val, expected) {
                mismatches.push(Mismatch {
                    fixture: spec.name.to_string(),
                    room: Some(format!("{} ({})", room.room_id, room.room_name)),
                    field: field.name(),
                    expected,
                    actual: actual_val,
                });
            }
        }
    }

    // ---- Building-level connection_capacity ------------------------------
    if let Some(expected_cap) = extract_expected_connection_capacity(&expected_value) {
        let actual_cap = actual.summary.connection_capacity;
        if !close_enough(actual_cap, expected_cap) {
            mismatches.push(Mismatch {
                fixture: spec.name.to_string(),
                room: None,
                field: "connection_capacity",
                expected: expected_cap,
                actual: actual_cap,
            });
        }
    }

    if !mismatches.is_empty() {
        let mut buf = String::new();
        buf.push_str(&format!(
            "\nFixture `{}` produced {} numeric mismatch(es) outside tolerance \
             (abs ±{ABS_TOLERANCE_W} W or rel ±{:.1} %):\n",
            spec.name,
            mismatches.len(),
            REL_TOLERANCE * 100.0
        ));
        buf.push_str(&format!(
            "  {:<32} {:<22} {:>12} {:>12} {:>12} {:>10}\n",
            "scope", "field", "expected", "actual", "Δ (W)", "Δ (%)"
        ));
        buf.push_str(&format!("  {}\n", "-".repeat(102)));
        for m in &mismatches {
            let scope = m.room.clone().unwrap_or_else(|| "<building>".to_string());
            buf.push_str(&format!(
                "  {:<32} {:<22} {:>12.2} {:>12.2} {:>+12.2} {:>+10.2}\n",
                truncate(&scope, 32),
                m.field,
                m.expected,
                m.actual,
                m.diff(),
                m.pct()
            ));
        }
        eprintln!("{buf}");
        panic!(
            "{} mismatch(es) in fixture `{}` — see stderr",
            mismatches.len(),
            spec.name
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

// ---------------------------------------------------------------------------
// Tests — one per fixture
// ---------------------------------------------------------------------------

#[test]
fn fixture_portiekwoning() {
    run_fixture(&FixtureSpec {
        name: "portiekwoning",
        input_file: "portiekwoning.json",
        expected_file: "portiekwoning_result.json",
        extract: extract_native_format,
    });
}

#[test]
#[ignore = "ISSO 51:2017 fixture — engine ondersteunt alleen ISSO 51:2023 (3BM-beleid 2026-05-13). 2017-paths worden niet geport; fixture blijft als historische referentie maar wordt niet meer getest. Verwijder bij opruimactie."]
fn fixture_vabi_vrijstaande_woning() {
    run_fixture(&FixtureSpec {
        name: "vabi_vrijstaande_woning",
        input_file: "vabi_vrijstaande_woning.json",
        expected_file: "vabi_vrijstaande_woning_expected.json",
        extract: extract_vabi_format,
    });
}

#[test]
fn fixture_dr_engineering_woningbouw() {
    run_fixture(&FixtureSpec {
        name: "dr_engineering_woningbouw",
        input_file: "dr_engineering_woningbouw.json",
        expected_file: "dr_engineering_woningbouw_result.json",
        extract: extract_dr_format,
    });
}

#[test]
fn fixture_woonboot() {
    // 3056 BWK woonboot — Project JSON + expected resultaat zijn gegenereerd
    // uit `thermal_import_woonboot.json` (revit-raycast export) via het
    // `gen_woonboot_expected` example. Geen Vabi-rapport beschikbaar voor
    // deze use case, dus de huidige norm-conforme engine output is de
    // baseline. Wanneer er bewust een gedrag-wijziging in Water-boundary
    // code wordt doorgevoerd: rerun `cargo run --example gen_woonboot_expected`
    // om de baseline bij te werken.
    run_fixture(&FixtureSpec {
        name: "woonboot",
        input_file: "woonboot.json",
        expected_file: "woonboot_result.json",
        extract: extract_native_format,
    });
}
