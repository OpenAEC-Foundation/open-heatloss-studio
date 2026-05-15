//! Vabi project import example.
//!
//! Demonstrates importing a Vabi Elements `.vp` project file and printing
//! a summary of the extracted data.
//!
//! Usage:
//! ```bash
//! cargo run --features vabi-import --example vabi_import -- "path/to/project.vp"
//! ```

use isso51_core::import::import_vabi_project;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_vp_file>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --features vabi-import --example vabi_import -- \"tests/references/Voorweg 210a - nieuw.vp\"");
        std::process::exit(1);
    }

    let vp_path = &args[1];

    println!("Importing Vabi project from: {}", vp_path);
    println!("{}", "=".repeat(80));

    match import_vabi_project(std::path::Path::new(vp_path)) {
        Ok(project) => {
            println!("✅ Import successful!");
            println!();

            // Project info
            println!("📋 Project Information:");
            println!("  Name: {}", project.info.name);
            if let Some(ref num) = project.info.project_number {
                println!("  Number: {}", num);
            }
            if let Some(ref notes) = project.info.notes {
                println!("  Notes: {}", notes);
            }
            println!();

            // Building data
            println!("🏠 Building:");
            println!("  Type: {:?}", project.building.building_type);
            println!("  qv10: {:.2} (method: {:?})", project.building.qv10, project.building.infiltration_method);
            println!("  Security class: {:?}", project.building.security_class);
            println!("  Floor area: {:.1} m²", project.building.total_floor_area);
            println!("  Floors: {}", project.building.num_floors);
            println!("  Night setback: {}", if project.building.has_night_setback { "Yes" } else { "No" });
            println!();

            // Climate
            println!("🌡️  Climate:");
            println!("  Design temperature: {:.1}°C", project.climate.theta_e);
            println!();

            // Ventilation
            println!("💨 Ventilation:");
            println!("  System type: {:?}", project.ventilation.system_type);
            println!("  Heat recovery: {}", if project.ventilation.has_heat_recovery { "Yes" } else { "No" });
            println!();

            // Rooms summary
            println!("🏠 Rooms ({} total):", project.rooms.len());
            println!("  {:<12} {:<25} {:<8} {:<8} {:<8} {:<12}", "ID", "Name", "Area", "Height", "θ_i", "Constructions");
            println!("  {}", "-".repeat(77));

            for room in &project.rooms {
                let theta_i = room.internal_air_temperature.unwrap_or(20.0);
                println!(
                    "  {:<12} {:<25} {:>6.1} m² {:>6.1} m {:>6.1}°C {:>10}",
                    room.id,
                    truncate_string(&room.name, 25),
                    room.floor_area,
                    room.height,
                    theta_i,
                    room.constructions.len()
                );
            }
            println!();

            // Construction summary (Phase 2)
            let total_constructions: usize = project.rooms.iter().map(|r| r.constructions.len()).sum();
            let rooms_with_constructions = project.rooms.iter().filter(|r| !r.constructions.is_empty()).count();
            let total_opaque_area: f64 = project.rooms.iter()
                .flat_map(|r| &r.constructions)
                .map(|c| c.area)
                .sum();

            println!("🏗️  Construction Summary (Phase 2):");
            println!("  Total constructions: {}", total_constructions);
            println!("  Rooms with constructions: {}/{}", rooms_with_constructions, project.rooms.len());
            println!("  Total opaque area: {:.1} m²", total_opaque_area);

            if total_constructions > 0 {
                let u_values: Vec<f64> = project.rooms.iter()
                    .flat_map(|r| &r.constructions)
                    .map(|c| c.u_value)
                    .collect();

                let u_min = u_values.iter().copied().fold(f64::INFINITY, f64::min);
                let u_max = u_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                println!("  U-value range: {:.3} - {:.3} W/(m²·K)", u_min, u_max);

                // Show construction details for first 3 rooms with constructions
                println!("\n🔧 Construction Details (first 3 rooms):");
                for (room_idx, room) in project.rooms.iter().filter(|r| !r.constructions.is_empty()).take(3).enumerate() {
                    println!("  {}. {} ({}):", room_idx + 1, room.name, room.id);
                    println!("     {:<20} {:<10} {:<12} {:<15}", "Description", "Area (m²)", "U-value", "Boundary");
                    println!("     {}", "-".repeat(57));

                    for construction in &room.constructions {
                        println!(
                            "     {:<20} {:>8.1} {:>10.3} {:<15}",
                            truncate_string(&construction.description, 20),
                            construction.area,
                            construction.u_value,
                            format!("{:?}", construction.boundary_type)
                        );
                    }
                    println!();
                }
            }
            println!();

            println!("📊 Summary:");
            println!("  Total floor area: {:.1} m² (from rooms: {:.1} m²)",
                project.building.total_floor_area,
                project.rooms.iter().map(|r| r.floor_area).sum::<f64>()
            );

            let avg_height = project.rooms.iter().map(|r| r.height).sum::<f64>() / project.rooms.len() as f64;
            println!("  Average room height: {:.2} m", avg_height);

            let temp_range = get_temperature_range(&project.rooms);
            println!("  Temperature range: {:.1}°C - {:.1}°C", temp_range.0, temp_range.1);

            println!();
            println!("✅ Import complete. Project ready for ISSO 51 calculation.");
            if total_constructions > 0 {
                println!("✅ Phase 2: Construction data imported successfully.");
            } else {
                println!("⚠️  Note: No construction data found in this project.");
            }
        }
        Err(e) => {
            eprintln!("❌ Import failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Truncate a string to max length, adding "..." if truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Get the min and max design temperatures across all rooms.
fn get_temperature_range(rooms: &[isso51_core::model::Room]) -> (f64, f64) {
    let temps: Vec<f64> = rooms
        .iter()
        .filter_map(|r| r.internal_air_temperature)
        .collect();

    if temps.is_empty() {
        (20.0, 20.0)
    } else {
        let min = temps.iter().copied().fold(f64::INFINITY, f64::min);
        let max = temps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }
}

