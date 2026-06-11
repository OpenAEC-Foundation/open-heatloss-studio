use std::path::Path;
use vabi_importer::{extract_elements_database, import_vabi_project_v2};

/// Reference `.vp` files in `tests/references/` are gitignored (klantdata,
/// publieke repo) en bestaan alleen op dev-machines. Skip de test netjes als
/// het bestand ontbreekt (bv. in CI); faal hard op importfouten als het er wél is.
fn reference_exists(vp_path: &Path) -> bool {
    if vp_path.exists() {
        true
    } else {
        eprintln!(
            "SKIPPED: reference file ontbreekt (gitignored, alleen lokaal): {}",
            vp_path.display()
        );
        false
    }
}

#[test]
fn tr03_houtfabriek_imports_to_v2() {
    let vp_path = Path::new("../../tests/references/TR03 - Houtfabriek.vp.zip");
    if !reference_exists(vp_path) {
        return;
    }
    let (db_path, _temp) = extract_elements_database(vp_path).expect("zip extract should succeed");

    let project = import_vabi_project_v2(&db_path).expect("V2 import should succeed");

    assert_eq!(project.schema_version, 2, "schema_version moet 2 zijn");
    assert!(
        project.geometry.spaces.len() >= 30,
        "TR03 heeft ~65 rooms, kreeg {}",
        project.geometry.spaces.len()
    );
    assert!(
        project.shared.gross_floor_area_m2.unwrap_or(0.0) > 100.0,
        "totaal vloeroppervlak moet substantieel zijn, kreeg {:?}",
        project.shared.gross_floor_area_m2
    );
    assert!(
        project.geometry.spaces.iter().all(|s| s.floor_area_m2 > 0.0),
        "alle spaces moeten positieve floor_area hebben"
    );
}

#[test]
fn vabi_24221_opdc_imports_to_v2() {
    let vp_path = Path::new("../../tests/references/24221-20250618.vp");
    if !reference_exists(vp_path) {
        return;
    }
    let (db_path, _temp) = extract_elements_database(vp_path).expect("zip extract should succeed");

    let project = import_vabi_project_v2(&db_path).expect("V2 import should succeed");

    assert_eq!(project.schema_version, 2);
    assert!(
        project.geometry.spaces.len() >= 50,
        "24221 heeft 106 rooms, kreeg {}",
        project.geometry.spaces.len()
    );
}
