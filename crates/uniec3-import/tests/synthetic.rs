//! Synthetische end-to-end-test — draait **wél** in CI (geen klantdata).
//!
//! Bouwt een minimale, volledig verzonnen `.uniec3`-ZIP in-memory (UTF-8 met
//! BOM, Nederlandse-komma-getallen) en importeert die. Zo dekt CI de parsing +
//! geometrie- + installatie-mapping ook zonder de gitignored echte exports. Alle
//! waarden zijn fictief; geen adressen of echte projectdata.

use std::io::{Cursor, Write};

use openaec_project_shared::beng_geometry::{KozijnType, RcOrU, VlakType};
use openaec_project_shared::energy::{HeatGeneratorType, VentilationSystemType};
use openaec_project_shared::shared::{BuildingTypeShared, ResidentialType};
use openaec_project_shared::Orientation;
use serde_json::json;
use uniec3_import::import_uniec3;

/// Bouw een entity-JSON-object.
fn ent(entity: &str, id: &str, order: f64, props: &[(&str, &str)]) -> serde_json::Value {
    json!({
        "NTAEntityId": entity,
        "NTAEntityDataId": id,
        "Order": order,
        "NTAPropertyDatas": props
            .iter()
            .map(|(k, v)| json!({"NTAPropertyId": k, "Value": v}))
            .collect::<Vec<_>>(),
    })
}

fn rel(parent: &str, child: &str) -> serde_json::Value {
    json!({"ParentId": parent, "ChildId": child})
}

/// Schrijf één archief-entry met UTF-8-BOM voor `value`.
fn write_entry<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    name: &str,
    value: &serde_json::Value,
) {
    zip.start_file(name, zip::write::FileOptions::default())
        .unwrap();
    zip.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
    zip.write_all(serde_json::to_string(value).unwrap().as_bytes())
        .unwrap();
}

/// Assembleer een minimale maar volledige synthetische `.uniec3` (bytes).
fn synthetic_uniec3() -> Vec<u8> {
    let entities = json!([
        ent("UNIT", "u1", 100.0, &[("UNIT_TYPEWON", "TWON_VRIJ_K"), ("UNIT_OMSCHR", "Woning")]),
        ent("UNIT-RZ", "ur1", 100.0, &[("UNIT-RZAG", "50,00"), ("UNIT-RZID", "rz1")]),
        ent("RZ", "rz1", 100.0, &[
            ("RZ_OMSCHR", "woning"),
            ("RZ_BOUWW_VL", "CONSTRM_FL_10"),
            ("RZ_BOUWW_W", "CONSTRM_W_10"),
        ]),
        // Bibliotheken.
        ent("LIBCONSTRD", "ld1", 100.0, &[("LIBCONSTRD_TYPE", "LIBVLAK_VLOER"), ("LIBCONSTRD_RC", "3,50"), ("LIBCONSTRD_OMSCHR", "Vloer")]),
        ent("LIBCONSTRD", "ld2", 200.0, &[("LIBCONSTRD_TYPE", "LIBVLAK_GEVEL"), ("LIBCONSTRD_RC", "4,50"), ("LIBCONSTRD_OMSCHR", "Wand")]),
        ent("LIBCONSTRT", "lt1", 100.0, &[("LIBCONSTRT_TYPE", "TRANSTYPE_RAAM"), ("LIBCONSTRT_U", "1,20"), ("LIBCONSTRT_G", "0,50"), ("LIBCONSTRT_AC", "2,00"), ("LIBCONSTRT_OMSCHR", "R1")]),
        // Vloer-begrenzing (op grond → omtrek verplicht).
        ent("BEGR", "b1", 100.0, &[("BEGR_VLAK", "VLAK_VLOER"), ("BEGR_VLOER", "VL_MV_GRSP"), ("BEGR_A", "50,00"), ("BEGR_HEL", "n.v.t."), ("BEGR_OMSCHR", "vloer")]),
        ent("CONSTRD", "cd1", 100.0, &[("CONSTRD_LIB", "ld1"), ("CONSTRD_OPP", "50,00")]),
        ent("CONSTRKENMV", "kv1", 100.0, &[("KENMV_OMTR_VL", "30,00")]),
        // Gevel-begrenzing (zuid) met één raam.
        ent("BEGR", "b2", 200.0, &[("BEGR_VLAK", "VLAK_GEVEL"), ("BEGR_GEVEL", "GVL_BTNL_Z"), ("BEGR_A", "20,00"), ("BEGR_HEL", "90"), ("BEGR_OMSCHR", "Wand")]),
        ent("CONSTRD", "cd2", 100.0, &[("CONSTRD_LIB", "ld2"), ("CONSTRD_OPP", "18,00")]),
        ent("CONSTRT", "ct1", 100.0, &[("CONSTRT_LIB", "lt1"), ("CONSTRT_AANT", "1"), ("CONSTRT_BESCH", "BELEMTYPE_MIN"), ("CONSTRT_ZONW", "ZONW_GEEN"), ("CONSTRT_ZNVENT", "ZOMERNVENT_NAANW")]),
        // Infiltratie.
        ent("INFILUNIT", "if1", 100.0, &[("INFILUNIT_QV", "0,45")]),
        // Installaties.
        ent("INSTALLATIE", "iv", 100.0, &[("INSTALL_TYPE", "INST_VERW")]),
        ent("VERW", "vw", 100.0, &[]),
        ent("VERW-OPWEK", "vo", 100.0, &[("VERW-OPWEK_POMP", "VERW-OPWEK_POMP_BUWA"), ("VERW-OPWEK_COP_NON", "4,00")]),
        ent("VERW-AFG", "va", 100.0, &[("VERW-AFG_TYPE_AFG", "VERW-AFG_TYPE_AFG_VLV")]),
        ent("INSTALLATIE", "ivt", 300.0, &[("INSTALL_TYPE", "INST_VENT")]),
        ent("VENT", "vt", 100.0, &[("VENT_VARIANT", "VARIANT_D2")]),
        ent("WARMTETERUG", "wt", 100.0, &[("WARMTETERUG_WTW", "WARMTETERUG_WTW_WEL"), ("WARMTETERUG_REND", "0,90")]),
        // Resultaten (certified).
        ent("RESULT-ENERGIEFUNCTIE", "ref1", 100.0, &[("RESULT-ENERGIEFUNCTIE_CAT", "RESULT_VERW"), ("RESULT-ENERGIEFUNCTIE_RES_ENER_PRIM", "1234,5")]),
        ent("RESULT-ENERGIEGEBRUIK", "reg1", 100.0, &[("RESULT-OPP_GEBROPP", "50,00"), ("RESULT-HERNIEUW_ELEKTR", "2000")]),
    ]);

    let relations = json!([
        rel("u1", "ur1"),
        rel("u1", "if1"),
        rel("ur1", "b1"),
        rel("ur1", "b2"),
        rel("b1", "cd1"),
        rel("b1", "kv1"),
        rel("b2", "cd2"),
        rel("b2", "ct1"),
        rel("iv", "vw"),
        rel("vw", "vo"),
        rel("vw", "va"),
        rel("ivt", "vt"),
        rel("vt", "wt"),
    ]);

    let summary = json!({
        "GEB_OMSCHR": "Synthetisch testgebouw",
        "GEB_TYPEGEB": "TGEB_GRWON",
        "EP_BENG1": "95,00",
        "EP_BENG2": "20,50",
        "EP_BENG3": "80,0",
        "EP_BENG1_EIS": "120,00",
        "EP_ENERGIELABEL": "A++",
    });

    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    write_entry(&mut zip, "meta.json", &json!({"Version": 2, "App": "NTA8800, Version=9.9.9.9, Culture=neutral"}));
    write_entry(&mut zip, "buildings.json", &json!([{"BuildingId": 1}]));
    write_entry(&mut zip, "buildings/1/entities.json", &entities);
    write_entry(&mut zip, "buildings/1/relations.json", &relations);
    write_entry(&mut zip, "buildings/1/summary.json", &summary);
    zip.finish().unwrap().into_inner()
}

#[test]
fn synthetic_archive_imports_end_to_end() {
    let bytes = synthetic_uniec3();
    let result = import_uniec3(&bytes).expect("synthetische import moet slagen");

    // Geen onbekende codes → geen waarschuwingen.
    assert!(
        result.warnings.is_empty(),
        "onverwachte waarschuwingen: {:?}",
        result.warnings
    );

    // Shared.
    let shared = &result.project.shared;
    assert_eq!(shared.name, "Synthetisch testgebouw");
    assert_eq!(
        shared.building_type,
        BuildingTypeShared::Woning {
            subtype: ResidentialType::Detached
        }
    );
    assert_eq!(shared.gross_floor_area_m2, Some(50.0));
    assert_eq!(shared.q_v10_spec_dm3_s_m2, Some(0.45));

    // Geometrie.
    let geo = result.project.beng_geometry.as_ref().unwrap();
    geo.validate().expect("opgebouwde geometrie moet valideren");
    assert_eq!(geo.opaque_defs.len(), 2);
    assert_eq!(geo.window_defs.len(), 1);
    assert_eq!(geo.zones.len(), 1);

    let zone = &geo.zones[0];
    assert_eq!(zone.a_g_m2, 50.0);
    assert_eq!(zone.woningtype.as_deref(), Some("TWON_VRIJ_K"));
    assert_eq!(zone.gevels.len(), 2);

    let vloer = zone.gevels.iter().find(|g| g.vlak_type == VlakType::Vloer).unwrap();
    assert_eq!(vloer.omtrek_p_m, Some(30.0));
    assert_eq!(geo.opaque_def(&vloer.constructie_ref).unwrap().thermal, RcOrU::Rc(3.50));

    let gevel = zone.gevels.iter().find(|g| g.vlak_type == VlakType::Gevel).unwrap();
    assert_eq!(gevel.bruto_buiten_opp_m2, 20.0);
    assert_eq!(gevel.helling_deg, Some(90.0));
    assert_eq!(gevel.grenst_aan.orientatie(), Some(Orientation::Zuid));
    assert_eq!(gevel.ramen.len(), 1);
    let wdef = geo.window_def(&gevel.ramen[0].kozijn_ref).unwrap();
    assert_eq!(wdef.kind, KozijnType::Raam);
    assert_eq!(wdef.area_m2, 2.0);

    // Installaties.
    let energy = result.project.energy.as_ref().unwrap();
    assert_eq!(
        energy.heating.as_ref().unwrap().generator,
        HeatGeneratorType::HeatPumpAir
    );
    assert_eq!(energy.heating.as_ref().unwrap().cop, Some(4.0));
    let vent = energy.ventilation.as_ref().unwrap();
    assert_eq!(vent.system, VentilationSystemType::D);
    assert_eq!(vent.wtw_efficiency, Some(0.90));

    // Certified.
    let c = &result.certified;
    assert_eq!(c.beng1_kwh_m2_jr, Some(95.0));
    assert_eq!(c.beng2_kwh_m2_jr, Some(20.5));
    assert_eq!(c.energy_label.as_deref(), Some("A++"));
    assert_eq!(c.heating_primary_kwh, Some(1234.5));
    assert_eq!(c.pv_production_kwh, Some(2000.0));
    assert_eq!(c.app_version.as_deref(), Some("9.9.9.9"));
}

#[test]
fn utility_building_is_rejected() {
    // Zelfde archief maar met een utiliteit-gebouwtype → nette, specifieke fout.
    let bytes = {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        write_entry(&mut zip, "meta.json", &json!({"Version": 2, "App": "x"}));
        write_entry(&mut zip, "buildings.json", &json!([{"BuildingId": 1}]));
        write_entry(&mut zip, "buildings/1/entities.json", &json!([]));
        write_entry(&mut zip, "buildings/1/relations.json", &json!([]));
        write_entry(&mut zip, "buildings/1/summary.json", &json!({"GEB_TYPEGEB": "TGEB_KANTOOR"}));
        zip.finish().unwrap().into_inner()
    };
    let err = import_uniec3(&bytes).unwrap_err();
    assert!(
        matches!(err, uniec3_import::Uniec3ImportError::UtilityUnsupported(_)),
        "verwacht UtilityUnsupported, kreeg {err:?}"
    );
}

#[test]
fn corrupt_archive_is_zip_error() {
    let err = import_uniec3(b"not a zip file at all").unwrap_err();
    assert!(matches!(err, uniec3_import::Uniec3ImportError::Zip(_)));
}

#[test]
fn too_many_entries_is_rejected() {
    // >1024 entries → zip-bomb-guard slaat toe vóór er iets gelezen wordt.
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..1100 {
        zip.start_file(format!("f{i}.json"), zip::write::FileOptions::default())
            .unwrap();
        zip.write_all(b"{}").unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let err = import_uniec3(&bytes).unwrap_err();
    assert!(
        matches!(err, uniec3_import::Uniec3ImportError::TooManyEntries { count, limit } if count == 1100 && limit == 1024),
        "verwacht TooManyEntries, kreeg {err:?}"
    );
}

#[test]
fn oversized_entry_is_rejected() {
    // `meta.json` declareert >64 MB uitgepakt. Met Deflate-compressie van nullen
    // blijft het archief zelf minuscuul, maar `ZipFile::size()` rapporteert de
    // echte grootte → de pre-check weert 'm zonder 64 MB te bufferen.
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "meta.json",
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    let chunk = vec![0u8; 1024 * 1024]; // 1 MB nullen
    for _ in 0..65 {
        zip.write_all(&chunk).unwrap(); // 65 MB uitgepakt > 64 MB-limiet
    }
    let bytes = zip.finish().unwrap().into_inner();
    let err = import_uniec3(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            uniec3_import::Uniec3ImportError::EntryTooLarge { ref file, .. } if file == "meta.json"
        ),
        "verwacht EntryTooLarge op meta.json, kreeg {err:?}"
    );
}
