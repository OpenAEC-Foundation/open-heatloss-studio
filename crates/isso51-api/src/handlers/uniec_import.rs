//! Uniec 3 (`.uniec3`) import-handler.
//!
//! Ontsluit `uniec3_import::import_uniec3` als publieke reken-route, analoog aan
//! `/beng/calculate`: een compleet `.uniec3`-archief (ZIP+JSON entity-graaf) →
//! een `ProjectV2` (met `beng_geometry` + `energy` + `q_v10;spec`) plus de
//! gecertificeerde Uniec/BengCert-uitkomsten als apart vergelijkingsobject.
//!
//! Het bestand komt als base64 in een JSON-envelope binnen (`file_base64`),
//! niet als multipart: zo blijft de client-dispatch (web `fetch` vs
//! Tauri `invoke`) symmetrisch met de rest van de BENG-familie. De route krijgt
//! in `main.rs` een eigen, ruimere `DefaultBodyLimit` (8 MB) omdat een
//! `.uniec3` groter kan zijn dan de 2 MB compute-default.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use base64::Engine;
use openaec_project_shared::ProjectV2;
use serde::{Deserialize, Serialize};
use uniec3_import::{import_uniec3, Uniec3CertifiedResults, Uniec3ImportError};

// `Deserialize` op de response is er t.b.v. de route-tests (die de body terug
// parsen); de handler zelf serialiseert alleen.

/// Request body voor POST /beng/import-uniec3.
///
/// Het `.uniec3`-archief (bytes) base64-gecodeerd. Standaard-base64
/// (`STANDARD`, met padding) — de client codeert de ruwe bestandsbytes.
#[derive(Debug, Clone, Deserialize)]
pub struct Uniec3ImportRequest {
    /// Het `.uniec3`-archief als standaard-base64 (met padding).
    pub file_base64: String,
}

/// Response body: het opgebouwde project + de certified referentie + warnings.
///
/// Spiegelt `uniec3_import::Uniec3Import`, maar als een eigen serialiseerbaar
/// type (de crate-struct is niet `Serialize`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uniec3ImportResponse {
    /// Het opgebouwde project (met `beng_geometry` + `energy` + `q_v10;spec`).
    pub project: ProjectV2,
    /// De gecertificeerde Uniec/BengCert-uitkomsten (vergelijkingsobject).
    pub certified: Uniec3CertifiedResults,
    /// Verzamelde waarschuwingen (overgeslagen/benaderde velden).
    pub warnings: Vec<String>,
}

/// POST /beng/import-uniec3 — importeer een `.uniec3`-archief.
///
/// Foutmapping:
/// - ongeldige base64 → 400 (client-fout in de envelope),
/// - `Uniec3ImportError` → 422 met de **letterlijke** `Display`-boodschap als
///   `detail` (multi-zone/utiliteit-afwijzing is directe user-feedback en moet
///   ongewijzigd doorkomen),
/// - blocking-task join failure → 500.
pub async fn import_uniec3_handler(
    Json(req): Json<Uniec3ImportRequest>,
) -> impl IntoResponse {
    let bytes = match base64::engine::general_purpose::STANDARD.decode(req.file_base64.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_base64",
                    "detail": format!("Bestand-payload is geen geldige base64: {e}"),
                })),
            )
                .into_response();
        }
    };

    let result = tokio::task::spawn_blocking(move || import_uniec3(&bytes)).await;

    match result {
        Ok(Ok(imported)) => Json(Uniec3ImportResponse {
            project: imported.project,
            certified: imported.certified,
            warnings: imported.warnings,
        })
        .into_response(),
        Ok(Err(e)) => {
            // Alle import-fouten zijn client-invoer-fouten (corrupt archief,
            // buiten-scope, incompleet) → 422 met de letterlijke boodschap.
            // De multi-zone-/utiliteit-afwijzing komt zo ongewijzigd bij de
            // gebruiker aan.
            let status = import_error_status(&e);
            (
                status,
                Json(serde_json::json!({
                    "error": "uniec3_import_error",
                    "detail": e.to_string(),
                })),
            )
                .into_response()
        }
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "detail": join_err.to_string(),
            })),
        )
            .into_response(),
    }
}

/// Alle `.uniec3`-importfouten zijn onverwerkbare client-invoer (corrupt,
/// incompleet, of buiten de V1-scope) → 422 Unprocessable Entity. Aparte functie
/// zodat een toekomstige verfijning (bv. zip-bomb → 413) één plek heeft.
fn import_error_status(_e: &Uniec3ImportError) -> StatusCode {
    StatusCode::UNPROCESSABLE_ENTITY
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    use base64::Engine;
    use serde_json::json;

    // -- Minimale synthetische `.uniec3`-bouwers (geen klantdata) --
    // Bewust in-test gedupliceerd i.p.v. de crate-test-helper te delen: die
    // leeft in `uniec3-import/tests/synthetic.rs` (een integratietest-crate,
    // niet uit de lib geëxporteerd). Hier volstaat een strak-minimale variant.

    fn ent(entity: &str, id: &str, props: &[(&str, &str)]) -> serde_json::Value {
        json!({
            "NTAEntityId": entity,
            "NTAEntityDataId": id,
            "Order": 100.0,
            "NTAPropertyDatas": props
                .iter()
                .map(|(k, v)| json!({"NTAPropertyId": k, "Value": v}))
                .collect::<Vec<_>>(),
        })
    }

    fn rel(parent: &str, child: &str) -> serde_json::Value {
        json!({"ParentId": parent, "ChildId": child})
    }

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

    /// Een geldig single-unit archief (spiegelt de crate-synthetic, ingekort).
    fn valid_uniec3() -> Vec<u8> {
        let entities = json!([
            ent("UNIT", "u1", &[("UNIT_TYPEWON", "TWON_VRIJ_K"), ("UNIT_OMSCHR", "Woning")]),
            ent("UNIT-RZ", "ur1", &[("UNIT-RZAG", "50,00"), ("UNIT-RZID", "rz1")]),
            ent("RZ", "rz1", &[("RZ_OMSCHR", "woning"), ("RZ_BOUWW_VL", "CONSTRM_FL_10"), ("RZ_BOUWW_W", "CONSTRM_W_10")]),
            ent("LIBCONSTRD", "ld1", &[("LIBCONSTRD_TYPE", "LIBVLAK_VLOER"), ("LIBCONSTRD_RC", "3,50"), ("LIBCONSTRD_OMSCHR", "Vloer")]),
            ent("LIBCONSTRD", "ld2", &[("LIBCONSTRD_TYPE", "LIBVLAK_GEVEL"), ("LIBCONSTRD_RC", "4,50"), ("LIBCONSTRD_OMSCHR", "Wand")]),
            ent("LIBCONSTRT", "lt1", &[("LIBCONSTRT_TYPE", "TRANSTYPE_RAAM"), ("LIBCONSTRT_U", "1,20"), ("LIBCONSTRT_G", "0,50"), ("LIBCONSTRT_AC", "2,00"), ("LIBCONSTRT_OMSCHR", "R1")]),
            ent("BEGR", "b1", &[("BEGR_VLAK", "VLAK_VLOER"), ("BEGR_VLOER", "VL_MV_GRSP"), ("BEGR_A", "50,00"), ("BEGR_HEL", "n.v.t."), ("BEGR_OMSCHR", "vloer")]),
            ent("CONSTRD", "cd1", &[("CONSTRD_LIB", "ld1"), ("CONSTRD_OPP", "50,00")]),
            ent("CONSTRKENMV", "kv1", &[("KENMV_OMTR_VL", "30,00")]),
            ent("BEGR", "b2", &[("BEGR_VLAK", "VLAK_GEVEL"), ("BEGR_GEVEL", "GVL_BTNL_Z"), ("BEGR_A", "20,00"), ("BEGR_HEL", "90"), ("BEGR_OMSCHR", "Wand")]),
            ent("CONSTRD", "cd2", &[("CONSTRD_LIB", "ld2"), ("CONSTRD_OPP", "18,00")]),
            ent("CONSTRT", "ct1", &[("CONSTRT_LIB", "lt1"), ("CONSTRT_AANT", "1"), ("CONSTRT_BESCH", "BELEMTYPE_MIN"), ("CONSTRT_ZONW", "ZONW_GEEN"), ("CONSTRT_ZNVENT", "ZOMERNVENT_NAANW")]),
            ent("INFILUNIT", "if1", &[("INFILUNIT_QV", "0,45")]),
            ent("INSTALLATIE", "iv", &[("INSTALL_TYPE", "INST_VERW")]),
            ent("VERW", "vw", &[]),
            ent("VERW-OPWEK", "vo", &[("VERW-OPWEK_POMP", "VERW-OPWEK_POMP_BUWA"), ("VERW-OPWEK_COP_NON", "4,00")]),
            ent("VERW-AFG", "va", &[("VERW-AFG_TYPE_AFG", "VERW-AFG_TYPE_AFG_VLV")]),
            ent("RESULT-ENERGIEFUNCTIE", "ref1", &[("RESULT-ENERGIEFUNCTIE_CAT", "RESULT_VERW"), ("RESULT-ENERGIEFUNCTIE_RES_ENER_PRIM", "1234,5")]),
            ent("RESULT-ENERGIEGEBRUIK", "reg1", &[("RESULT-OPP_GEBROPP", "50,00"), ("RESULT-HERNIEUW_ELEKTR", "2000")]),
        ]);
        let relations = json!([
            rel("u1", "ur1"), rel("u1", "if1"), rel("ur1", "b1"), rel("ur1", "b2"),
            rel("b1", "cd1"), rel("b1", "kv1"), rel("b2", "cd2"), rel("b2", "ct1"),
            rel("iv", "vw"), rel("vw", "vo"), rel("vw", "va"),
        ]);
        let summary = json!({
            "GEB_OMSCHR": "Synthetisch testgebouw",
            "GEB_TYPEGEB": "TGEB_GRWON",
            "EP_BENG1": "95,00",
            "EP_BENG2": "20,50",
            "EP_BENG3": "80,0",
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

    /// Twee UNIT-entiteiten → `MultiUnitUnsupported` (appartementen/meergezins).
    fn multi_unit_uniec3() -> Vec<u8> {
        let entities = json!([
            ent("UNIT", "u1", &[("UNIT_TYPEWON", "TWON_VRIJ_K")]),
            ent("UNIT", "u2", &[("UNIT_TYPEWON", "TWON_VRIJ_K")]),
        ]);
        let summary = json!({ "GEB_OMSCHR": "Meergezins", "GEB_TYPEGEB": "TGEB_GRWON" });
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        write_entry(&mut zip, "meta.json", &json!({"Version": 2, "App": "x"}));
        write_entry(&mut zip, "buildings.json", &json!([{"BuildingId": 1}]));
        write_entry(&mut zip, "buildings/1/entities.json", &entities);
        write_entry(&mut zip, "buildings/1/relations.json", &json!([]));
        write_entry(&mut zip, "buildings/1/summary.json", &summary);
        zip.finish().unwrap().into_inner()
    }

    fn b64(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[tokio::test]
    async fn valid_archive_returns_200_with_project_and_certified() {
        let req = Uniec3ImportRequest {
            file_base64: b64(&valid_uniec3()),
        };
        let resp = import_uniec3_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: Uniec3ImportResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed.project.shared.name, "Synthetisch testgebouw");
        assert!(parsed.project.beng_geometry.is_some());
        assert!(parsed.project.energy.is_some());
        assert_eq!(parsed.certified.beng1_kwh_m2_jr, Some(95.0));
        assert_eq!(parsed.certified.energy_label.as_deref(), Some("A++"));
    }

    #[tokio::test]
    async fn corrupt_zip_returns_422() {
        let req = Uniec3ImportRequest {
            file_base64: b64(b"not a zip file at all"),
        };
        let resp = import_uniec3_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn multi_zone_returns_422_with_literal_message() {
        let req = Uniec3ImportRequest {
            file_base64: b64(&multi_unit_uniec3()),
        };
        let resp = import_uniec3_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let detail = v["detail"].as_str().unwrap();
        // De multi-unit-afwijzing komt letterlijk door (user-feedback).
        assert!(
            detail.contains("units") || detail.contains("meergezins"),
            "onverwachte detail-boodschap: {detail}"
        );
    }

    #[tokio::test]
    async fn invalid_base64_returns_400() {
        let req = Uniec3ImportRequest {
            file_base64: "!!! not base64 !!!".to_string(),
        };
        let resp = import_uniec3_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// E2E met het échte Aalten-golden `.uniec3` (gitignored, alleen lokaal).
    /// Skipt netjes als het bestand ontbreekt (bv. in CI) — patroon van de
    /// round-trip-tests. Bewijst dat de volledige route-keten (base64 → decode →
    /// `import_uniec3` → JSON-response) een afgemeld gebouw importeert:
    /// project + certified + de bekende twee waarschuwingen.
    #[tokio::test]
    async fn aalten_golden_imports_end_to_end_if_present() {
        let path = "../../tests/verification/beng_uniec_crosscheck/aalten-2522/2522_woning-aalten_2024-11-22.uniec3";
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("SKIPPED: Aalten-golden ontbreekt (gitignored, alleen lokaal): {path}");
            return;
        };

        let req = Uniec3ImportRequest {
            file_base64: b64(&bytes),
        };
        let resp = import_uniec3_handler(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK, "Aalten-import moet slagen");

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: Uniec3ImportResponse = serde_json::from_slice(&body).unwrap();

        // Project opgebouwd met gevel-geometrie + installaties.
        assert!(parsed.project.beng_geometry.is_some());
        assert!(parsed.project.energy.is_some());
        // Certified BENG-indicatoren aanwezig (vergelijkingsobject gevuld).
        assert!(parsed.certified.beng1_kwh_m2_jr.is_some());
        assert!(parsed.certified.beng2_kwh_m2_jr.is_some());
        // De Aalten-export levert twee bekende waarschuwingen.
        assert_eq!(
            parsed.warnings.len(),
            2,
            "verwacht 2 waarschuwingen, kreeg: {:?}",
            parsed.warnings
        );
    }
}
