//! Draai alle test-fixtures en print een samenvattingstabel.
//!
//! ```bash
//! cargo run -p nta8800-core --example run_all
//! ```

use std::fs;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("fixtures dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    entries.sort();

    println!(
        "{:<32} {:>7} {:>10} {:>10} {:>8} {:>6}",
        "fixture", "label", "EP MJ/m²", "kWh/m²", "hern.%", "CO2/m²"
    );
    println!("{}", "-".repeat(80));

    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let json = fs::read_to_string(&path).expect("fixture leesbaar");
        match nta8800_core::calculate_from_json(&json) {
            Ok(out) => {
                let r: nta8800_core::Nta8800Result =
                    serde_json::from_str(&out).expect("result parse");
                println!(
                    "{:<32} {:>7} {:>10.0} {:>10.1} {:>7.0}% {:>6.1}",
                    name,
                    r.ep.label,
                    r.ep.primary_energy_mj_per_m2,
                    r.ep.primary_energy_kwh_per_m2,
                    r.ep.renewable_share * 100.0,
                    r.ep.co2_kg_per_m2,
                );
            }
            Err(e) => println!("{name:<32} FOUT: {e}"),
        }
    }
}
