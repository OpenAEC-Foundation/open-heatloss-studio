//! Vabi-referentie verificatietests voor peak Koellast (EN 12831 / NEN 5060 TO2).
//!
//! ## Status: ENGINE NOG NIET GEÏMPLEMENTEERD
//!
//! Deze fixture (`tests/verification/koellast_vabi3.12.0.127_dr-engineering-woningbouw/`)
//! bevat peak cooling load data (W per ruimte + gebouw, mei-sep tijdvak 8-20) uit een
//! Vabi Elements Koellast rapport.
//!
//! De huidige `nta8800-cooling` crate rekent **annual cooling demand (NTA 8800 H.10)** —
//! Q_C in MJ, H_T/H_V in W/K, τ in uren. Dat is een **andere physical quantity** dan
//! peak koellast en is niet 1-op-1 te vergelijken met de Vabi PDF.
//!
//! Daarom staan alle tests in dit bestand op `#[ignore]` tot een aparte
//! `peak-cooling` engine (EN 12831 / NEN 5060) ontworpen en geïmplementeerd is.
//!
//! ## Fixture inhoud
//!
//! - `expected.json`: peak W per ruimte (Woonkamer 2074 W, Keuken 1869 W, etc.) +
//!   gebouw-totaal 6420 W in augustus tijdvak 14
//! - `input.json`: **bestaat nog niet** — vereist nieuwe project-reconstructie passend
//!   bij de echte 191,7 m² woning uit het Vabi rapport (oude 120 m² synthetisch model
//!   is verwijderd want misleidend)
//!
//! ## Wanneer de engine wel bestaat
//!
//! 1. Bouw `input.json` op vanuit Vabi-rapport (191,7 m² Ag, 6 gekoelde ruimtes)
//! 2. Implementeer `peak-cooling` engine (apart van `nta8800-cooling`)
//! 3. Verwijder `#[ignore]` attributen hieronder
//! 4. Run `cargo test -p nta8800-cooling vabi_koellast_woning`

// Bewust geen imports van de NTA 8800 cooling engine — die past niet bij peak koellast.
// Wanneer de peak-cooling engine ontstaat, importeer die hier.

/// Smoke test: dummy compile-time check zodat dit bestand niet stuk gaat in cargo build.
///
/// Deze test is NIET ignored maar doet niets functioneels. Hij garandeert dat
/// `cargo test --workspace` blijft compileren ook al heeft deze fixture nog geen
/// engine en geen input.json.
#[test]
fn vabi_koellast_woning_compiles_and_runs() {
    // Geen assertions — fixture wacht op peak-cooling engine implementatie.
    // Zie module-doc hierboven voor de open punten.
    let _placeholder = "peak-cooling engine TBD";
}

/// Toekomstige test: KPI's binnen plausibele ranges (zonder Vabi-vergelijking).
///
/// IGNORED tot peak-cooling engine bestaat.
#[test]
#[ignore = "Peak koellast engine TBD; current nta8800-cooling crate doet alleen NTA 8800 H.10 annual TO-juli"]
fn vabi_koellast_woning_kpis_in_plausible_range() {
    // Toekomst:
    // 1. Laad input.json + expected.json
    // 2. Run peak-cooling engine
    // 3. Verifieer dat alle room.peak_w binnen [50, 5000] W liggen
    // 4. Verifieer dat building.peak_w binnen [500, 30000] W ligt
    unimplemented!("peak-cooling engine ontbreekt nog");
}

/// Toekomstige test: exacte Vabi-match per ruimte binnen 10% tolerantie.
///
/// IGNORED tot peak-cooling engine bestaat.
#[test]
#[ignore = "Peak koellast engine TBD; current nta8800-cooling crate doet alleen NTA 8800 H.10 annual TO-juli"]
fn vabi_koellast_woning_matches() {
    // Toekomst:
    // 1. Laad expected.json:
    //    let expected_json = include_str!(
    //        "../../../tests/verification/koellast_vabi3.12.0.127_dr-engineering-woningbouw/expected.json"
    //    );
    // 2. Voor elke room in expected.rooms[]:
    //    - bereken peak_w via peak-cooling engine
    //    - assert binnen 10% van expected.rooms[i].peak_w
    // 3. Building totaal binnen 5% van expected.kpis.peak_cooling_load_total_w
    unimplemented!("peak-cooling engine ontbreekt nog");
}
