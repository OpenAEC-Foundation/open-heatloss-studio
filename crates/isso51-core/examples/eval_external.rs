//! Snelle evaluatie van een externe ISSO 51 project-JSON (klantbestand
//! of canonical fixture) via de huidige engine. Print het
//! gebouw-aansluitvermogen + breakdown van alle Φ-componenten op stdout.
//!
//! Ondersteunt zowel de outer-wrapper export-vorm (`{ version, schema,
//! project: {...} }`) als de bare `Project` JSON. Voor klant-JSONs uit
//! de UI (export knop) is de wrapper-vorm standaard.
//!
//! Gebruik:
//!     cargo run --release --example eval_external -p isso51-core -- <path>
//!
//! Historie: ontwikkeld 2026-05-12 voor Weesp Muiderweg 565 sanity-check
//! (klantproject 3076).

use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("argv[1] = path to JSON required")?;
    let input = fs::read_to_string(&path)?;

    // The exported JSON has an outer wrapper { version, schema, project: {...} }.
    // The engine expects the bare Project struct. Strip the wrapper if needed.
    let parsed: serde_json::Value = serde_json::from_str(&input)?;
    let project_json = if let Some(p) = parsed.get("project") {
        serde_json::to_string(p)?
    } else {
        input.clone()
    };

    // Echo which infiltration_method is in effect (after deserialization defaults).
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&project_json) {
        let m = v
            .get("building")
            .and_then(|b| b.get("infiltration_method"))
            .map(|x| x.to_string())
            .unwrap_or_else(|| "<absent → default>".to_string());
        let dc = v
            .get("building")
            .and_then(|b| b.get("dwelling_class"))
            .map(|x| x.to_string())
            .unwrap_or_else(|| "<none>".to_string());
        let cv = v
            .get("building")
            .and_then(|b| b.get("construction_variant"))
            .map(|x| x.to_string())
            .unwrap_or_else(|| "<none>".to_string());
        let cy = v
            .get("building")
            .and_then(|b| b.get("construction_year"))
            .map(|x| x.to_string())
            .unwrap_or_else(|| "<none>".to_string());
        eprintln!(
            "[input] infiltration_method={} dwelling_class={} construction_variant={} construction_year={}",
            m, dc, cv, cy
        );
    }

    let result_json = isso51_core::calculate_from_json(&project_json)?;
    let result: serde_json::Value = serde_json::from_str(&result_json)?;

    let summary = result.get("summary").ok_or("no summary in result")?;
    let rooms = result.get("rooms").and_then(|r| r.as_array()).map(|a| a.len()).unwrap_or(0);

    println!("=== Summary ===");
    println!("rooms: {}", rooms);
    println!("connection_capacity  : {} W", summary["connection_capacity"]);
    println!("phi_basis_total      : {} W", summary["phi_basis_total"]);
    println!("phi_vent_building    : {} W", summary["phi_vent_building"]);
    println!("phi_extra_quadratic  : {} W", summary["phi_extra_quadratic"]);
    if let Some(v) = summary.get("phi_t_iabe_building") {
        println!("phi_t_iabe_building  : {} W", v);
    }
    if let Some(v) = summary.get("phi_hu_building") {
        println!("phi_hu_building      : {} W", v);
    }
    if let Some(v) = summary.get("phi_infiltration_building") {
        println!("phi_infiltration_bld : {} W", v);
    }
    if let Some(v) = summary.get("phi_system_building") {
        println!("phi_system_building  : {} W", v);
    }
    Ok(())
}
