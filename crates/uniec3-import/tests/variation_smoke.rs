//! Variatie-smoke over de lokale corpus (F8 fase 4-validatie).
//!
//! Draait de importer over álle `.uniec3`-bestanden in
//! `C:\Users\JochemK\Desktop\uniec\` en rapporteert per bestand: parse OK/fout +
//! aantal waarschuwingen. `#[ignore]` — diagnostiek, geen CI-gate (de corpus is
//! lokaal/klantdata). Draai handmatig:
//!
//! ```text
//! cargo test -p uniec3-import --test variation_smoke -- --ignored --nocapture
//! ```

use std::path::Path;

use openaec_project_shared::compute_beng;
use uniec3_import::import_uniec3;

#[test]
#[ignore = "diagnostiek — lokale corpus, draai met --ignored --nocapture"]
fn variation_smoke_over_desktop_corpus() {
    let dir = Path::new(r"C:\Users\JochemK\Desktop\uniec");
    if !dir.exists() {
        eprintln!("SKIPPED: corpus-map ontbreekt: {}", dir.display());
        return;
    }

    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "uniec3"))
        .collect();
    files.sort();

    let mut ok = 0;
    let mut fail = 0;
    println!("\n=== variatie-smoke over {} bestanden ===", files.len());
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let bytes = std::fs::read(path).unwrap();
        match import_uniec3(&bytes) {
            Ok(r) => {
                ok += 1;
                let g = r.project.beng_geometry.as_ref();
                let n_gevels: usize = g.map_or(0, |g| g.zones.iter().map(|z| z.gevels.len()).sum());
                println!(
                    "  OK   {name}  (v{}, {} gevels, {} PV-velden, {} warn)",
                    r.certified.app_version.as_deref().unwrap_or("?"),
                    n_gevels,
                    r.project.energy.as_ref().map_or(0, |e| e.pv.len()),
                    r.warnings.len()
                );
            }
            Err(e) => {
                fail += 1;
                println!("  FOUT {name}  → {e}");
            }
        }
    }
    println!("=== {ok} OK, {fail} fout ===");
}

/// MZ-V2b: draai `compute_beng` over de hele corpus en verifieer dat het
/// per-rekenzone-demand-pad op géén enkel bestand paniekt of faalt — met nadruk
/// op de multi-rekenzone-bestanden (kelder-splitsingen incl. de 4 m²-mini-kelder,
/// drijvende woningen). Rapporteert per multi-zone-bestand het aantal zones +
/// BENG 1/2/3. `#[ignore]` — lokale corpus, geen CI-gate.
#[test]
#[ignore = "diagnostiek — lokale corpus, draai met --ignored --nocapture"]
fn compute_beng_over_desktop_corpus() {
    let dir = Path::new(r"C:\Users\JochemK\Desktop\uniec");
    if !dir.exists() {
        eprintln!("SKIPPED: corpus-map ontbreekt: {}", dir.display());
        return;
    }

    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "uniec3"))
        .collect();
    files.sort();

    let mut single = 0;
    let mut multi = 0;
    let mut errors = 0;
    println!("\n=== compute_beng over {} bestanden ===", files.len());
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let bytes = std::fs::read(path).unwrap();
        let Ok(r) = import_uniec3(&bytes) else {
            continue; // import-falen valt onder de andere smoke
        };
        let n_zones = r
            .project
            .beng_geometry
            .as_ref()
            .map_or(0, |g| g.zones.len());
        match compute_beng(&r.project) {
            Ok(b) => {
                if n_zones > 1 {
                    multi += 1;
                    println!(
                        "  MZ({n_zones}) {name}  BENG1={:.2}  BENG2={:.2}  BENG3={:.1}",
                        b.beng1.value, b.beng2.value, b.beng3.value
                    );
                } else {
                    single += 1;
                }
            }
            Err(e) => {
                errors += 1;
                println!("  FOUT {name}  ({n_zones} zones) → {e}");
            }
        }
    }
    println!("=== single-zone {single}, multi-zone {multi}, compute_beng-fouten {errors} ===");
    assert_eq!(errors, 0, "compute_beng faalde op {errors} bestand(en)");
}
