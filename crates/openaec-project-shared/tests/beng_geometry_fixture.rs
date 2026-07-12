//! F6 fase-1 vangrail: de gevel-georiënteerde BENG-geometrie-fixture voor
//! 2522 Woning Aalten parst en valideert groen.
//!
//! De fixture (`tests/verification/beng_uniec_crosscheck/aalten-2522/
//! beng_geometry.input.json`) is gereconstrueerd uit de certified-Uniec
//! velden-capture van dezelfde case. Deze test bewijst dat het
//! [`BengGeometry`]-invoermodel die geometrie 1-op-1 kan dragen: de twee
//! bibliotheken, 6 begrenzingsvlakken en kozijn-plaatsingen deserialiseren, de
//! resolver/validator loopt zonder fouten, en de per-gevel opaak-oppervlakten
//! sluiten aan op de betrouwbaar-gecapturede Uniec-waarden (invariant-check
//! tegen tikfouten in de fixture).
//!
//! Kanttekening: de kozijn-plaatsing op Wand (O) en Wand (W) is gereconstrueerd
//! (stale capture; zie `_note` in de fixture + `docs/2026-07-12-uniec-velden-
//! inventarisatie.md` §1). De opake vlakken en buiten-oppervlakten zijn wél
//! certified; die worden hier hard geasserteerd.

use openaec_project_shared::{BengAdjacency, BengGeometry, Orientation, VlakType};

const AALTEN_BENG_GEOMETRY: &str = include_str!(
    "../../../tests/verification/beng_uniec_crosscheck/aalten-2522/beng_geometry.input.json"
);

/// Parse de fixture naar [`BengGeometry`]. Onbekende `_meta`/`_note`-sleutels
/// (fixture-commentaar) worden door serde genegeerd.
fn load() -> BengGeometry {
    serde_json::from_str(AALTEN_BENG_GEOMETRY).expect("Aalten BENG-geometrie-fixture moet parsen")
}

/// Totale kozijn-oppervlakte op een gevel = Σ (aantal · WindowDef::area_m2).
fn windows_area(geo: &BengGeometry, gevel: &openaec_project_shared::BengBoundary) -> f64 {
    gevel
        .ramen
        .iter()
        .map(|r| {
            let def = geo
                .window_def(&r.kozijn_ref)
                .unwrap_or_else(|| panic!("kozijn_ref {} moet resolven", r.kozijn_ref));
            f64::from(r.aantal) * def.area_m2
        })
        .sum()
}

#[test]
fn fixture_parses_and_validates() {
    let geo = load();
    geo.validate().expect("Aalten-geometrie moet groen valideren");
}

#[test]
fn fixture_has_expected_library_and_zone_shape() {
    let geo = load();
    assert_eq!(geo.opaque_defs.len(), 3, "3 opake constructie-definities");
    assert_eq!(geo.window_defs.len(), 13, "13 kozijnmerken (A–J + deurglas + deur + dakraam)");
    assert_eq!(geo.zones.len(), 1, "1 rekenzone");

    let zone = &geo.zones[0];
    assert!((zone.a_g_m2 - 67.00).abs() < 1e-9, "A_g = 67,00 m²");
    assert_eq!(zone.gevels.len(), 6, "6 begrenzingsvlakken (vloer + 4 gevels + dak)");
}

#[test]
fn floor_is_ground_bound_with_perimeter() {
    let geo = load();
    let vloer = geo.zones[0]
        .gevels
        .iter()
        .find(|g| g.id == "gevel-vloer")
        .expect("vloer-vlak");
    assert_eq!(vloer.vlak_type, VlakType::Vloer);
    assert_eq!(vloer.grenst_aan, BengAdjacency::VloerOpMaaiveldBovenGrond);
    assert!(vloer.grenst_aan.requires_omtrek());
    assert!(
        (vloer.omtrek_p_m.expect("omtrek P verplicht bij vloer-op-grond") - 32.92).abs() < 1e-9
    );
    assert!((vloer.bruto_buiten_opp_m2 - 67.00).abs() < 1e-9);
}

#[test]
fn boundary_orientations_match_uniec() {
    let geo = load();
    let by_id = |id: &str| {
        geo.zones[0]
            .gevels
            .iter()
            .find(|g| g.id == id)
            .unwrap_or_else(|| panic!("gevel {id}"))
            .grenst_aan
            .orientatie()
    };
    assert_eq!(by_id("gevel-n").unwrap(), Orientation::Noord);
    assert_eq!(by_id("gevel-o").unwrap(), Orientation::Oost);
    assert_eq!(by_id("gevel-z").unwrap(), Orientation::Zuid);
    assert_eq!(by_id("gevel-w").unwrap(), Orientation::West);
    // Dak is een hellend N-vlak (DAK_BTNL_N, 15°), geen HOR.
    assert_eq!(by_id("gevel-dak").unwrap(), Orientation::Noord);
    // Vloer draagt geen oriëntatie.
    assert!(by_id("gevel-vloer").is_none());
}

#[test]
fn opaque_areas_match_certified_capture() {
    // Per gevel: opaak = bruto − Σ ramen. Alle 6 gevels zijn certified: N/Z/dak uit
    // de eerste walk, O/W uit de her-capture v2 (uniec_fields_capture_retry2.json,
    // mét losse invoervelden). Voor élke gevel is de certified opake CONSTRD_OPP
    // bekend en sluit opaak + ramen exact op het bruto gevelvlak.
    let geo = load();
    let check = |id: &str, expected_opaque: f64| {
        let g = geo.zones[0].gevels.iter().find(|g| g.id == id).unwrap();
        let opaque = g.bruto_buiten_opp_m2 - windows_area(&geo, g);
        assert!(
            (opaque - expected_opaque).abs() < 1e-6,
            "gevel {id}: opaak {opaque:.2} != verwacht {expected_opaque:.2}"
        );
        assert!(opaque > 0.0, "gevel {id}: opaak-oppervlak moet positief zijn");
    };
    check("gevel-n", 16.51); // certified: opaak 16,51 + ramen 5,45 = 21,96
    check("gevel-z", 29.91); // certified: opaak 29,91 + ramen 9,95 = 39,86
    check("gevel-dak", 68.10); // certified: opaak 68,10 + dakraam 1,20 = 69,30
    check("gevel-o", 18.77); // certified: opaak 18,77 + ramen 5,04 (A+B+C) = 23,81
    check("gevel-w", 18.22); // certified: opaak 18,22 + ramen 5,59 (F+G) = 23,81
}
