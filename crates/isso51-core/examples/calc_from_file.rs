//! Calculate from a project JSON on disk and print the result.
//!
//! Usage:
//!   cargo run --example calc_from_file -- <project.json>
//!
//! Prints both the full result JSON and a summary block with per-room and
//! building totals so manual comparison with a reference PDF is easy.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: cargo run --example calc_from_file -- <project.json>")?;

    let raw = std::fs::read_to_string(&path)?;

    // The .isso51.json envelope has { schema, version, exported_at, project, result, ... }.
    // Detect envelope vs raw Project and unwrap if needed.
    let json_value: serde_json::Value = serde_json::from_str(&raw)?;
    let project_json = if json_value.get("schema").is_some() && json_value.get("project").is_some() {
        json_value["project"].to_string()
    } else {
        raw.clone()
    };

    // Run calc.
    let result_json = isso51_core::calculate_from_json(&project_json)?;

    // Pretty-print the full result.
    let parsed: serde_json::Value = serde_json::from_str(&result_json)?;
    let pretty = serde_json::to_string_pretty(&parsed)?;
    println!("=== Full result JSON ===");
    println!("{pretty}");

    // Summary: building total + per-room phi_HL.
    println!("\n=== Summary ===");
    if let Some(summary) = parsed.get("summary") {
        println!("Building summary:");
        println!("{}", serde_json::to_string_pretty(summary)?);
    }

    if let Some(rooms) = parsed.get("rooms").and_then(|r| r.as_array()) {
        println!("\nPer-room (alle waarden in W):");
        println!(
            "{:<32} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "name", "phi_HL", "phi_T", "phi_V", "phi_inf", "phi_RH"
        );
        let mut sum_hl = 0.0;
        for room in rooms {
            let name = room
                .get("room_name")
                .and_then(|n| n.as_str())
                .unwrap_or("?");
            let phi_hl = room.get("total_heat_loss").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let phi_t = room
                .get("transmission")
                .and_then(|t| t.get("phi_t"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let phi_v = room
                .get("ventilation")
                .and_then(|t| t.get("phi_v"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let phi_i = room
                .get("infiltration")
                .and_then(|t| t.get("phi_i"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let phi_rh = room
                .get("heating_up")
                .and_then(|t| t.get("phi_hu"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            println!(
                "{name:<32} {phi_hl:>10.1} {phi_t:>10.1} {phi_v:>10.1} {phi_i:>10.1} {phi_rh:>10.1}"
            );
            sum_hl += phi_hl;
        }
        println!(
            "{:<32} {:>10.1}",
            "── som totaal heat loss (W) ──", sum_hl
        );
    }

    Ok(())
}
