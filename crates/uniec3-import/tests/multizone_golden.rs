//! MZ-V2b multi-rekenzone-golden — projectnr. 2176 (vrijstaande woning, 3 rekenzones).
//!
//! Importeert het lokale drie-rekenzone-`.uniec3` (vrijstaande woning met kelder,
//! app 3.3.6) en pint de importer-kant vast: 3 rekenzones, A_g;tot = 435,10 m²
//! (Σ 159 + 117,1 + 159), en het gecertificeerde BENG-triplet uit de summary.
//! Daarna draait [`compute_beng`] via het **norm-exacte per-rekenzone-pad**
//! (MZ-V2b, §6.6.2/§8.2.2 formule 10.19: demand per zone, dan gesommeerd) en
//! toetst de energiebehoefte-indicator **BENG 1** binnen de reguliere
//! F8-tolerantie (±6 %). BENG 2/3 worden **gerapporteerd, niet geasserteerd**: hun
//! restgap is dezelfde **PV-saldering-normversie**-discrepantie als bij de
//! single-zone Aalten/Gouda-goldens (NTA 8800:2025+C1 §5.5.2 salderert PV-export
//! volledig tegen fP;exp;el = 1,45; certified Uniec 3.3.x crediteert ~64 %), géén
//! multi-zone-demand-fout. Zie de V2a→V2b-meting hieronder + het MZ-doc §11.
//!
//! Het bron-`.uniec3` is gitignored (klantdata, publieke repo) en wordt via een
//! glob op `*.uniec3` in de golden-map gevonden — de bestandsnaam zelf (klantdata)
//! staat dus niet in de repo. De test **skipt netjes** als het ontbreekt (bv. in
//! CI), net als `round_trip.rs`.

use std::path::Path;

use openaec_project_shared::compute_beng;
use uniec3_import::import_uniec3;

/// Golden-map (geanonimiseerd op projectnr.); het gitignored `.uniec3` erin wordt
/// via een `*.uniec3`-glob gevonden zodat de klant-bestandsnaam niet in de repo staat.
const GOLDEN_DIR: &str = "../../tests/verification/beng_uniec_crosscheck/woning-2176";

/// A_g van de drie rekenzones (uit UNIT-RZAG, bevestigd op het bestand).
const A_G_TOTAL: f64 = 435.10; // 159,00 + 117,10 + 159,00

/// Vind en lees het (enige) `.uniec3` in de golden-map; `None` (met skip-melding)
/// als de map/het bestand ontbreekt (gitignored, alleen lokaal).
fn read_golden_uniec3() -> Option<Vec<u8>> {
    let dir = Path::new(GOLDEN_DIR);
    let entry = std::fs::read_dir(dir).ok()?.filter_map(std::result::Result::ok).find(|e| {
        e.path().extension().is_some_and(|x| x == "uniec3")
    });
    match entry {
        Some(e) => Some(std::fs::read(e.path()).expect("kon .uniec3 lezen")),
        None => {
            eprintln!("SKIPPED: geen .uniec3 in {GOLDEN_DIR} (gitignored, alleen lokaal)");
            None
        }
    }
}

/// F8-tolerantie voor de energiebehoefte-indicator BENG 1 (identiek aan de
/// single-zone Aalten-golden: ±6 %). BENG 1 = (Q_H;nd + Q_C;nd)/A_g;tot en is dus
/// de zuivere maat voor de per-rekenzone-demand die MZ-V2b levert.
const BENG1_TOL_PCT: f64 = 6.0;

#[test]
fn woning_2176_imports_three_zones_with_pooled_a_g() {
    let Some(bytes) = read_golden_uniec3() else {
        return;
    };
    let result = import_uniec3(&bytes).expect("woning-2176-import moet slagen");

    let geo = result
        .project
        .beng_geometry
        .as_ref()
        .expect("import moet een beng_geometry produceren");
    geo.validate().expect("multi-zone geometrie moet valideren");

    // Structuur: drie rekenzones.
    assert_eq!(geo.zones.len(), 3, "verwacht 3 rekenzones");

    // A_g;tot = Σ zones = 435,10 (exact op 2 decimalen).
    let a_g_sum: f64 = geo.zones.iter().map(|z| z.a_g_m2).sum();
    assert!(
        (a_g_sum - A_G_TOTAL).abs() < 0.01,
        "A_g;tot uit zones = {a_g_sum} ≠ {A_G_TOTAL}"
    );
    assert_eq!(
        result.project.shared.gross_floor_area_m2,
        Some(a_g_sum),
        "gross_floor_area_m2 moet Σ zones zijn"
    );

    // Certified A_g (RESULT-OPP_GEBROPP, gebouw-niveau) reproduceert Σ zones.
    if let Some(cert_ag) = result.certified.gebruiks_opp_m2 {
        assert!(
            (cert_ag - A_G_TOTAL).abs() < 0.01,
            "certified A_g = {cert_ag} ≠ {A_G_TOTAL}"
        );
    }

    // Gecertificeerd BENG-triplet aanwezig (het vergelijkingsobject).
    let c = &result.certified;
    let beng1 = c.beng1_kwh_m2_jr.expect("certified BENG 1 aanwezig");
    let beng2 = c.beng2_kwh_m2_jr.expect("certified BENG 2 aanwezig");
    let beng3 = c.beng3_pct.expect("certified BENG 3 aanwezig");

    // Indicatief-warning uit de importer.
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("rekenzones geïmporteerd")),
        "verwacht gepoold-indicatief-warning, kreeg: {:?}",
        result.warnings
    );

    // ---- MZ-V2b: per-rekenzone-demand → BENG 1 binnen F8-tol; BENG 2/3 gerapporteerd.
    let r = compute_beng(&result.project).expect("compute_beng op multi-zone project");
    let d1 = (r.beng1.value - beng1) / beng1 * 100.0;
    println!("\n=== Woning 2176 — 3 rekenzones, MZ-V2b per-zone ===");
    println!("A_g;tot = {a_g_sum:.2} m² (certified {:?})", c.gebruiks_opp_m2);
    println!(
        "BENG 1  V2b {:7.2}  certified {beng1:7.2}  Δ {:+.2} ({d1:+.1} %)  [tol ±{BENG1_TOL_PCT} %]",
        r.beng1.value,
        r.beng1.value - beng1
    );
    println!(
        "BENG 2  V2b {:7.2}  certified {beng2:7.2}  Δ {:+.2}  (PV-saldering-normversie, niet geasserteerd)",
        r.beng2.value,
        r.beng2.value - beng2
    );
    println!(
        "BENG 3  V2b {:7.2}  certified {beng3:7.2}  Δ {:+.2}  (idem, niet geasserteerd)",
        r.beng3.value,
        r.beng3.value - beng3
    );

    // (1) Norm-exact-note: geen INDICATIEF (MZ-V2a) meer, wél de MZ-V2b-note.
    assert!(
        !r.notes.iter().any(|n| n.contains("INDICATIEF (MZ-V2a)")),
        "V2b-pad mag geen INDICATIEF (MZ-V2a)-note meer dragen: {:?}",
        r.notes
    );
    assert!(
        r.notes.iter().any(|n| n.contains("MZ-V2b (norm-exact)")),
        "multi-zone-berekening moet de MZ-V2b-norm-exact-note dragen: {:?}",
        r.notes
    );

    // (2) BENG 1 (energiebehoefte) binnen de reguliere F8-tolerantie — de zuivere
    // maat voor de per-rekenzone-demand die V2b levert (V2a-gepoold was −12,5 %,
    // buiten tol; V2b brengt het naar ~−4,8 %). BENG 2/3 blijven gated door de
    // PV-saldering-normversie (zelfde artefact als de single-zone goldens), géén
    // multi-zone-fout → bewust NIET geasserteerd (anti-fudge).
    assert!(
        d1.abs() <= BENG1_TOL_PCT,
        "BENG 1 = {:.2} vs certified {beng1:.2} = {d1:+.1} %, buiten ±{BENG1_TOL_PCT} %",
        r.beng1.value
    );
}
