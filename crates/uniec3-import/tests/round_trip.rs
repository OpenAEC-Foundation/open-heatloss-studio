//! Round-trip-golden (F8 fase 4h) — de kernvalidatie.
//!
//! Importeert de twee lokale `.uniec3`-bestanden (Aalten + Gouda) en vergelijkt
//! het opgebouwde [`BengGeometry`] **veld-voor-veld** met de bestaande
//! hand-fixtures (`beng_geometry.input.json`). De formaat-analyse bewees al
//! 28/28 (Aalten) + 29/29 (Gouda) op data-niveau; deze test automatiseert dat
//! als regressie-vangrail.
//!
//! De vergelijking is op **waarde**, niet op id-string: de importer genereert
//! Uniec-GUID's als id's terwijl de hand-fixture leesbare id's (`merk-a`,
//! `gevel-n`) gebruikt. Constructie-/kozijn-referenties worden daarom aan beide
//! kanten geresolved naar hun Rc/U/ggl/opp-waarden.
//!
//! De bron-`.uniec3`-bestanden zijn gitignored (klantdata, publieke repo). De
//! test **skipt netjes** als ze ontbreken (bv. in CI) — het patroon van
//! `vabi-importer/tests/v2_import.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use openaec_project_shared::beng_geometry::BengGeometry;
use uniec3_import::import_uniec3;

/// Lees een lokaal `.uniec3`-bestand; `None` (met skip-melding) als het ontbreekt.
fn read_uniec3(path: &str) -> Option<Vec<u8>> {
    let p = Path::new(path);
    if p.exists() {
        Some(std::fs::read(p).expect("kon .uniec3 lezen"))
    } else {
        eprintln!("SKIPPED: .uniec3 ontbreekt (gitignored, alleen lokaal): {path}");
        None
    }
}

fn load_fixture(path: &str) -> BengGeometry {
    let raw = std::fs::read_to_string(path).expect("kon fixture lezen");
    serde_json::from_str(&raw).expect("fixture parst naar BengGeometry")
}

// ---------------------------------------------------------------------------
// Canonieke, id-onafhankelijke platmaak van een BengGeometry
// ---------------------------------------------------------------------------

fn q100(v: f64) -> i64 {
    (v * 100.0).round() as i64
}
fn q1000(v: f64) -> i64 {
    (v * 1000.0).round() as i64
}

/// Serde-string van een klein enum/adjacency (id-onafhankelijk, waarde-stabiel).
fn tag<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

/// Canonieke sleutel van een gevel: (vlak-type, grenst-aan, bruto-opp) — genoeg
/// om dezelfde fysieke gevel aan beide kanten te herkennen zonder id.
fn gevel_key(g: &openaec_project_shared::beng_geometry::BengBoundary) -> String {
    format!(
        "{}|{}|{}",
        tag(&g.vlak_type),
        tag(&g.grenst_aan),
        q100(g.bruto_buiten_opp_m2)
    )
}

/// Vlak een BengGeometry naar een map `veldnaam → canonieke waarde`. Beide zijden
/// (import + fixture) produceren dezelfde sleutels voor dezelfde fysieke velden.
fn flatten(g: &BengGeometry) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();

    // Bibliotheken als geordende multiset (waarde, niet id).
    let mut opaque: Vec<String> = g
        .opaque_defs
        .iter()
        .map(|d| format!("{}:{}", tag(&d.kind), tag(&d.thermal)))
        .collect();
    opaque.sort();
    m.insert("opaque_defs".to_string(), opaque.join(","));

    let mut windows: Vec<String> = g
        .window_defs
        .iter()
        .map(|d| {
            format!(
                "{}:u{}:g{}:a{}",
                tag(&d.kind),
                q1000(d.u_w_per_m2k),
                d.ggl.map_or(-1, q1000),
                q100(d.area_m2)
            )
        })
        .collect();
    windows.sort();
    m.insert("window_defs".to_string(), windows.join(","));

    // Aanname: single-zone (V1-scope). Meer zones zou de test moeten uitbreiden.
    let zone = &g.zones[0];
    m.insert("zone.a_g".to_string(), q100(zone.a_g_m2).to_string());
    m.insert(
        "zone.bouwwijze_vloer".to_string(),
        zone.bouwwijze_vloer.clone().unwrap_or_default(),
    );
    m.insert(
        "zone.bouwwijze_wand".to_string(),
        zone.bouwwijze_wand.clone().unwrap_or_default(),
    );
    m.insert(
        "zone.woningtype".to_string(),
        zone.woningtype.clone().unwrap_or_default(),
    );
    m.insert("zone.n_gevels".to_string(), zone.gevels.len().to_string());

    for gevel in &zone.gevels {
        let k = gevel_key(gevel);
        m.insert(
            format!("{k}.helling"),
            gevel.helling_deg.map_or(-1, q1000).to_string(),
        );
        m.insert(
            format!("{k}.omtrek"),
            gevel.omtrek_p_m.map_or(-1, q100).to_string(),
        );
        // Constructie-referentie geresolved naar de thermische waarde.
        let thermal = g
            .opaque_def(&gevel.constructie_ref)
            .map_or_else(|| "MISSING".to_string(), |d| tag(&d.thermal));
        m.insert(format!("{k}.thermal"), thermal);

        // Ramen als multiset van geresolveerde (kind,u,ggl,area,aantal,
        // belemmering,zomernacht).
        let mut ramen: Vec<String> = gevel
            .ramen
            .iter()
            .map(|r| {
                let def = g.window_def(&r.kozijn_ref);
                let (kind, u, ggl, area) = def.map_or_else(
                    || ("MISSING".to_string(), -1, -1, -1),
                    |d| (tag(&d.kind), q1000(d.u_w_per_m2k), d.ggl.map_or(-1, q1000), q100(d.area_m2)),
                );
                format!(
                    "{kind}:u{u}:g{ggl}:a{area}:n{}:b{}:z{}",
                    r.aantal,
                    tag(&r.belemmering),
                    r.zomernachtventilatie
                )
            })
            .collect();
        ramen.sort();
        m.insert(format!("{k}.ramen"), ramen.join(","));
    }

    m
}

/// Vergelijk twee platgemaakte geometrieën; retourneert (aantal velden,
/// exact-matches, lijst van afwijkingen `veld: import ≠ fixture`).
fn compare(imported: &BengGeometry, fixture: &BengGeometry) -> (usize, usize, Vec<String>) {
    let a = flatten(imported);
    let b = flatten(fixture);

    let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();

    let mut exact = 0;
    let mut diffs = Vec::new();
    for k in &keys {
        let va = a.get(*k);
        let vb = b.get(*k);
        if va == vb {
            exact += 1;
        } else {
            diffs.push(format!(
                "{k}: import={:?} ≠ fixture={:?}",
                va.map(String::as_str),
                vb.map(String::as_str)
            ));
        }
    }
    (keys.len(), exact, diffs)
}

fn run_case(name: &str, uniec3_path: &str, fixture_path: &str) {
    let Some(bytes) = read_uniec3(uniec3_path) else {
        return;
    };
    let result = import_uniec3(&bytes).unwrap_or_else(|e| panic!("{name}: import faalde: {e}"));
    let imported = result
        .project
        .beng_geometry
        .as_ref()
        .expect("import moet een beng_geometry produceren");
    let fixture = load_fixture(fixture_path);

    let (total, exact, diffs) = compare(imported, &fixture);
    println!(
        "\n=== {name}: {exact}/{total} velden exact; {} waarschuwing(en) ===",
        result.warnings.len()
    );
    for w in &result.warnings {
        println!("  warn: {w}");
    }
    for d in &diffs {
        println!("  DIFF {d}");
    }
    assert!(
        diffs.is_empty(),
        "{name}: {} veld(en) wijken af van de hand-fixture",
        diffs.len()
    );
}

#[test]
fn aalten_round_trips_to_hand_fixture() {
    run_case(
        "aalten-2522",
        "../../tests/verification/beng_uniec_crosscheck/aalten-2522/2522_woning-aalten_2024-11-22.uniec3",
        "../../tests/verification/beng_uniec_crosscheck/aalten-2522/beng_geometry.input.json",
    );
}

#[test]
fn gouda_round_trips_to_hand_fixture() {
    run_case(
        "gouda-2467",
        "../../tests/verification/beng_uniec_crosscheck/gouda-2467/2467_goejanverwelledijk-85-gouda_2024-09-17.uniec3",
        "../../tests/verification/beng_uniec_crosscheck/gouda-2467/beng_geometry.input.json",
    );
}

/// Certified-extractie tegen de bestaande `expected.json`-provenance: de uit het
/// `.uniec3` gehaalde BENG-kernindicatoren moeten de gepubliceerde certified
/// waarden reproduceren (dezelfde bron als `beng_golden.rs`).
#[test]
fn certified_matches_expected_json() {
    let cases = [
        (
            "aalten-2522",
            "../../tests/verification/beng_uniec_crosscheck/aalten-2522/2522_woning-aalten_2024-11-22.uniec3",
            "../../tests/verification/beng_uniec_crosscheck/aalten-2522/expected.json",
        ),
        (
            "gouda-2467",
            "../../tests/verification/beng_uniec_crosscheck/gouda-2467/2467_goejanverwelledijk-85-gouda_2024-09-17.uniec3",
            "../../tests/verification/beng_uniec_crosscheck/gouda-2467/expected.json",
        ),
    ];
    for (name, uniec3_path, expected_path) in cases {
        let Some(bytes) = read_uniec3(uniec3_path) else {
            continue;
        };
        let result = import_uniec3(&bytes).unwrap_or_else(|e| panic!("{name}: import faalde: {e}"));
        let c = &result.certified;
        let exp: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(expected_path).unwrap()).unwrap();
        let e = &exp["expected"];

        let approx = |got: Option<f64>, want: f64, tol: f64, field: &str| {
            let g = got.unwrap_or_else(|| panic!("{name}: {field} ontbreekt in certified"));
            assert!(
                (g - want).abs() <= tol,
                "{name}: {field} import={g} ≠ expected={want}"
            );
        };
        // BENG-indicatoren + eisen komen 1-op-1 uit summary.json.
        approx(c.beng1_kwh_m2_jr, e["beng1_kwh_m2_jr"].as_f64().unwrap(), 0.01, "beng1");
        approx(c.beng2_kwh_m2_jr, e["beng2_kwh_m2_jr"].as_f64().unwrap(), 0.01, "beng2");
        approx(c.beng3_pct, e["beng3_pct"].as_f64().unwrap(), 0.01, "beng3");
        approx(
            c.beng1_limit_kwh_m2_jr,
            e["beng1_limit_kwh_m2_jr"].as_f64().unwrap(),
            0.01,
            "beng1_limit",
        );
        assert_eq!(
            c.energy_label.as_deref(),
            e["energy_label"].as_str(),
            "{name}: energy_label"
        );
        // Per-functie primair + PV + koudebehoefte (afronding op hele kWh).
        approx(c.heating_primary_kwh, e["heating_primary_kwh"].as_f64().unwrap(), 1.0, "heating_primary");
        approx(c.hot_water_primary_kwh, e["hot_water_primary_kwh"].as_f64().unwrap(), 1.0, "hot_water_primary");
        approx(c.cooling_primary_kwh, e["cooling_primary_kwh"].as_f64().unwrap(), 1.0, "cooling_primary");
        approx(c.fans_primary_kwh, e["fans_primary_kwh"].as_f64().unwrap(), 1.0, "fans_primary");
        approx(c.pv_production_kwh, e["pv_production_kwh"].as_f64().unwrap(), 1.0, "pv_production");
        approx(c.cooling_demand_kwh, e["cooling_demand_kwh"].as_f64().unwrap(), 1.0, "cooling_demand");
    }
}
