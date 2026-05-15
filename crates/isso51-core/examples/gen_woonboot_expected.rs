//! Genereer woonboot Project + expected resultaat uit thermal_import scanner-output.
//!
//! Leest `tests/fixtures/thermal_import_woonboot.json` (revit-raycast export
//! uit pyrevit-gis2bim, project 3056) en produceert twee files:
//!
//! - `tests/fixtures/woonboot.json`         — canonical Project struct
//! - `tests/fixtures/woonboot_result.json`  — verwacht ProjectResult (baseline)
//!
//! Gebruik als regressie-bescherming voor Water-boundary engine logica.
//! Geen Vabi-rapport beschikbaar voor deze use case — de huidige
//! norm-conforme engine output dient als baseline.
//!
//! Gebruik: `cargo run --release --example gen_woonboot_expected -p isso51-core`

use isso51_core::calculate;
use isso51_core::import::{map_thermal_import, ThermalImport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "tests/fixtures/thermal_import_woonboot.json";
    let project_path = "tests/fixtures/woonboot.json";
    let result_path = "tests/fixtures/woonboot_result.json";

    let raw = std::fs::read_to_string(input_path)?;
    let thermal: ThermalImport = serde_json::from_str(&raw)?;
    println!("Loaded thermal import: {} rooms, {} constructions",
        thermal.rooms.len(), thermal.constructions.len());

    let mapped = map_thermal_import(thermal);
    println!("Mapped to Project: {} rooms, {} catalog entries, {} warnings",
        mapped.project.rooms.len(), mapped.construction_catalog.len(), mapped.warnings.len());
    for w in &mapped.warnings {
        println!("  warning: {}", w);
    }

    let project_json = serde_json::to_string_pretty(&mapped.project)?;
    std::fs::write(project_path, &project_json)?;
    println!("Written {} ({} bytes)", project_path, project_json.len());

    let result = calculate(&mapped.project)?;
    let result_json = serde_json::to_string_pretty(&result)?;
    std::fs::write(result_path, &result_json)?;
    println!("Written {} ({} bytes)", result_path, result_json.len());

    println!();
    println!("=== Summary ===");
    println!("rooms              : {}", result.rooms.len());
    println!("connection_capacity: {:.1} W", result.summary.connection_capacity);
    println!("phi_basis_total    : {:.1} W", result.summary.phi_basis_total);
    println!("phi_vent_building  : {:.1} W", result.summary.phi_vent_building);
    println!("phi_extra_quadratic: {:.1} W", result.summary.phi_extra_quadratic);

    Ok(())
}
