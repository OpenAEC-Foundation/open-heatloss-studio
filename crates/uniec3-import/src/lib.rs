//! # uniec3-import — native `.uniec3` → [`ProjectV2`] (F8)
//!
//! Parseert een Uniec 3 native export (`.uniec3`, drie-puntjes-menu →
//! exporteren) naar het gevel-georiënteerde [`BengGeometry`]- en
//! [`EnergyInput`]-invoermodel plus de **gecertificeerde** BENG-uitkomsten als
//! apart vergelijkingsobject ([`Uniec3CertifiedResults`]).
//!
//! Het `.uniec3`-bestand is een volledige, exacte bron voor alles wat de
//! BENG-keten nodig heeft (kruisvalidatie: 28/28 + 29/29 velden op de Aalten- en
//! Gouda-golden, nul mismatches). Zie `docs/2026-07-13-f8-uniec3-formaat-
//! analyse.md` voor het formaat-schema en de mappingtabel.
//!
//! ## Gebruik
//!
//! ```no_run
//! # fn demo(bytes: &[u8]) -> Result<(), uniec3_import::Uniec3ImportError> {
//! let result = uniec3_import::import_uniec3(bytes)?;
//! println!("project: {}", result.project.shared.name);
//! println!("certified BENG 2: {:?}", result.certified.beng2_kwh_m2_jr);
//! for w in &result.warnings {
//!     eprintln!("waarschuwing: {w}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Scope V1
//!
//! Woningbouw, **single-unit** (één UNIT, één rekenzone). Meerdere units of
//! rekenzones (appartementen/multi-zone) en utiliteitsbouw geven een nette,
//! specifieke fout in plaats van een stille aanname — zie [`Uniec3ImportError`]
//! en de V2-tickets in `TODO.md`.
//!
//! ## Tolerantie
//!
//! Onbekende veldcodes/entiteit-typen worden overgeslagen en verzameld in
//! [`Uniec3Import::warnings`] (niet falen), zodat nieuwere app-versies met extra
//! velden blijven importeren. Alleen een corrupt archief, ontbrekende
//! kernbestanden of buiten-scope-invoer falen hard.

#![deny(missing_docs)]

mod error;
mod geometry;
mod installations;
mod parse;
mod results;

pub use error::{Result, Uniec3ImportError};
pub use results::Uniec3CertifiedResults;

use openaec_project_shared::shared::{BuildingTypeShared, ResidentialType};
use openaec_project_shared::ProjectV2;

/// Het resultaat van een `.uniec3`-import.
#[derive(Debug, Clone)]
pub struct Uniec3Import {
    /// Het opgebouwde project (met `beng_geometry` + `energy` + `q_v10;spec`).
    pub project: ProjectV2,
    /// De gecertificeerde Uniec/BengCert-uitkomsten (vergelijkingsobject).
    pub certified: Uniec3CertifiedResults,
    /// Verzamelde waarschuwingen (overgeslagen/benaderde velden). Leeg =
    /// volledig herkend.
    pub warnings: Vec<String>,
}

/// Importeer een `.uniec3`-archief (bytes) naar een [`Uniec3Import`].
///
/// # Errors
///
/// - [`Uniec3ImportError::Zip`]/[`Io`](Uniec3ImportError::Io)/[`Json`](Uniec3ImportError::Json)
///   — corrupt of onleesbaar archief.
/// - [`MissingFile`](Uniec3ImportError::MissingFile)/[`NoBuilding`](Uniec3ImportError::NoBuilding)
///   — incompleet archief.
/// - [`MultiUnitUnsupported`](Uniec3ImportError::MultiUnitUnsupported)/[`UtilityUnsupported`](Uniec3ImportError::UtilityUnsupported)
///   — buiten V1-scope.
/// - [`MissingGeometry`](Uniec3ImportError::MissingGeometry)/[`GeometryValidation`](Uniec3ImportError::GeometryValidation)
///   — geen/ongeldige rekenzone-geometrie.
pub fn import_uniec3(bytes: &[u8]) -> Result<Uniec3Import> {
    let mut warnings = Vec::new();

    let raw = parse::read_archive(bytes)?;
    if raw.meta.version != 2 {
        warnings.push(format!(
            "onbekende containerversie {} (verwacht 2); parsen voortgezet",
            raw.meta.version
        ));
    }

    // Utiliteit-guard: het entity-model is generiek, maar utiliteit-import is
    // onbeproefd (geen sample-export). Woningtypen dragen `WON`/`WOON` in
    // GEB_TYPEGEB — `TGEB_GRWON` (grondgebonden) én `TGEB_WOONBB` (woonark/
    // drijvende woning) zijn woningbouw; alleen echte utiliteitscodes weren we.
    if let Some(tgeb) = raw.summary.get("GEB_TYPEGEB").and_then(|v| v.as_str()) {
        let u = tgeb.to_ascii_uppercase();
        if !(u.contains("WON") || u.contains("WOON")) {
            return Err(Uniec3ImportError::UtilityUnsupported(tgeb.to_string()));
        }
    }

    let project_name = raw
        .summary
        .get("GEB_OMSCHR")
        .and_then(|v| v.as_str())
        .unwrap_or("Uniec-import")
        .to_string();

    let idx = parse::EntityIndex::new(raw.entities, raw.relations);

    let beng_geometry = geometry::map_geometry(&idx, &mut warnings)?;
    beng_geometry
        .validate()
        .map_err(|e| Uniec3ImportError::GeometryValidation(e.to_string()))?;

    let (energy, q_v10) = installations::map_installations(&idx, &mut warnings);
    let certified = results::extract_results(&raw.summary, &idx, &raw.meta);

    let gross_floor_area: f64 = beng_geometry.zones.iter().map(|z| z.a_g_m2).sum();
    let subtype = residential_subtype(
        beng_geometry
            .zones
            .first()
            .and_then(|z| z.woningtype.as_deref()),
        &mut warnings,
    );

    let mut project = ProjectV2::new(project_name);
    project.shared.building_type = BuildingTypeShared::Woning { subtype };
    project.shared.gross_floor_area_m2 = Some(gross_floor_area);
    project.shared.q_v10_spec_dm3_s_m2 = q_v10;
    project.beng_geometry = Some(beng_geometry);
    project.energy = Some(energy);

    Ok(Uniec3Import {
        project,
        certified,
        warnings,
    })
}

/// Map de Uniec `UNIT_TYPEWON`-code op een [`ResidentialType`] (stuurt het
/// infiltratie-leakage-forfait). Onbekend → vrijstaand + waarschuwing.
fn residential_subtype(code: Option<&str>, warnings: &mut Vec<String>) -> ResidentialType {
    let Some(code) = code else {
        warnings.push("geen UNIT_TYPEWON → vrijstaand aangenomen".to_string());
        return ResidentialType::Detached;
    };
    let c = code.to_ascii_uppercase();
    if c.contains("VRIJ") {
        ResidentialType::Detached
    } else if c.contains("2O1K") || c.contains("TWEE") || c.contains("2KAP") {
        ResidentialType::SemiDetached
    } else if c.contains("TUS") {
        ResidentialType::Terraced
    } else if c.contains("HOEK") {
        ResidentialType::EndOfTerrace
    } else if c.contains("PORT") {
        ResidentialType::Porch
    } else if c.contains("GAL") {
        ResidentialType::Gallery
    } else if c.contains("APP") || c.contains("MAIS") || c.contains("FLAT") {
        ResidentialType::Stacked
    } else {
        warnings.push(format!(
            "onbekend woningtype {code} → vrijstaand aangenomen"
        ));
        ResidentialType::Detached
    }
}
