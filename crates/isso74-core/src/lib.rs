//! # ISSO 74 Thermal Comfort Assessment Engine
//!
//! Pure Rust implementation of the ISSO 74 (2e druk) thermisch-comfort /
//! oververhittingstoets for utility/office buildings.
//!
//! This is a **toets-laag** (assessment layer): the engineer supplies hourly
//! operative temperatures θ_o per room (from an external dynamic simulation,
//! via CSV) plus the outdoor air temperature. The crate then computes:
//!
//! * **RMOT** — running mean outdoor temperature (Kader 3.2, formule 3.1),
//! * **ATG** — the adaptive temperature bandwidth test (Tabel 3.3),
//! * **TO-uren** — overheating-hour counts >25/>28 °C (Bijlage A.1),
//! * **GTO** — weighted exceedance hours via Fanger PMV/PPD (Bijlage A.2),
//!
//! and returns a per-room verdict plus ATG scatter-plot data (afb. 4.1).
//!
//! It does **not** run a dynamic simulation — that is Fase 2.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use isso74_core::calculate_from_json;
//!
//! let input_json = r#"{ "csv": "...", "config": { } }"#;
//! let result_json = calculate_from_json(input_json).unwrap();
//! ```
//!
//! ## Architecture
//!
//! Pure computation library — no I/O, no async, no unsafe. JSON in, JSON out.

pub mod calc;
pub mod error;
pub mod model;
pub mod result;
pub mod tables;

use error::Result;
use model::Isso74Request;
use result::Isso74Result;

/// Run an ISSO 74 assessment from a JSON request string.
///
/// The JSON must deserialize to [`Isso74Request`] (CSV content + config).
/// Returns the [`Isso74Result`] as a pretty-printed JSON string.
///
/// # Errors
/// Returns [`error::Isso74Error`] if the JSON or the embedded CSV is invalid.
pub fn calculate_from_json(input_json: &str) -> Result<String> {
    let request: Isso74Request = serde_json::from_str(input_json)?;
    let result = assess_request(&request)?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Run an ISSO 74 assessment from a typed request.
pub fn assess_request(request: &Isso74Request) -> Result<Isso74Result> {
    let parsed = calc::csv::parse_csv(&request.csv)?;
    calc::assess::assess(&parsed, &request.config)
}

/// Base URL for published schemas.
const SCHEMA_BASE_URL: &str = "https://warmteverlies.open-aec.com/schemas/v1";
/// Current schema version.
const SCHEMA_VERSION: &str = "1.0.0";

/// Generate the JSON schema for the [`Isso74Request`] input type.
pub fn request_schema() -> String {
    let schema = schemars::schema_for!(Isso74Request);
    add_schema_metadata(schema, "isso74-request")
}

/// Generate the JSON schema for the [`Isso74Result`] output type.
pub fn result_schema() -> String {
    let schema = schemars::schema_for!(Isso74Result);
    add_schema_metadata(schema, "isso74-result")
}

fn add_schema_metadata(schema: schemars::schema::RootSchema, name: &str) -> String {
    let mut value = serde_json::to_value(&schema).unwrap_or_default();
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            serde_json::Value::String(format!("{SCHEMA_BASE_URL}/{name}.schema.json")),
        );
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(SCHEMA_VERSION.to_string()),
        );
    }
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_from_fixture() {
        let csv = include_str!("../tests/fixtures/example_hourly.csv");
        let request = Isso74Request {
            csv: csv.to_string(),
            config: model::Isso74Config::default(),
        };
        let json = serde_json::to_string(&request).unwrap();
        let out = calculate_from_json(&json).unwrap();
        assert!(out.contains("\"rooms\""));
        assert!(out.contains("\"summary\""));
        assert!(out.contains("\"assumptions\""));

        // Also exercise the typed path and check structural invariants.
        let result = assess_request(&request).unwrap();
        assert_eq!(result.summary.rooms_total, result.rooms.len() as u32);
        assert_eq!(
            result.summary.rooms_passing + result.summary.rooms_failing,
            result.summary.rooms_total
        );
        for room in &result.rooms {
            assert_eq!(room.atg.assessed_hours as usize, room.plot.len());
        }
    }

    #[test]
    fn schemas_generate() {
        assert!(request_schema().contains("isso74-request"));
        assert!(result_schema().contains("isso74-result"));
    }
}
