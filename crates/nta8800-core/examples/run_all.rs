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
        "{:<32} {:>7} {:>9} {:>8} {:>8} {:>8} {:>7}",
        "fixture", "label", "EP kWh/m²", "BENG1", "BENG2", "BENG3", "pass"
    );
    println!("{}", "-".repeat(88));

    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let json = fs::read_to_string(&path).expect("fixture leesbaar");
        match nta8800_core::calculate_from_json(&json) {
            Ok(out) => {
                let r: nta8800_core::Nta8800Result =
                    serde_json::from_str(&out).expect("result parse");
                let pass = [
                    r.beng.beng1_pass,
                    r.beng.beng2_pass,
                    r.beng.beng3_pass,
                ]
                .iter()
                .map(|p| if *p { '+' } else { '-' })
                .collect::<String>();
                println!(
                    "{:<32} {:>7} {:>9.1} {:>8.1} {:>8.1} {:>7.0}% {:>7}",
                    name,
                    r.ep.label,
                    r.ep.primary_energy_kwh_per_m2,
                    r.beng.beng1_kwh_per_m2,
                    r.beng.beng2_kwh_per_m2,
                    r.beng.beng3_pct,
                    pass,
                );
            }
            Err(e) => println!("{name:<32} FOUT: {e}"),
        }
    }
    println!("\npass-kolom: BENG1/2/3 t.o.v. indicatieve nieuwbouw-grenzen (+ = voldoet).");
}
