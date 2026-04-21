//! Tenant-aware CORS layer builder.
//!
//! Reads `<tenants_root>/<slug>/tenant.yaml` bij startup, bouwt een
//! origin-set, en construeert een `tower-http` `CorsLayer` die per request
//! de `Origin`-header matcht tegen deze set.
//!
//! Patroon analoog aan `openaec-reports` `core/tenant_cors.py` +
//! `core/cors_middleware.py` (commit 66fb4f7). Schema-referentie:
//! `C:/GitHub/openaec-tenants/tenants/_schema.md`.
//!
//! ## Semantiek
//!
//! - **Backward-compat:** als `tenants_root` ontbreekt of geen origins
//!   oplevert, valt de layer terug op de statische `cors_origins` lijst
//!   uit de bestaande env-var `CORS_ORIGINS`.
//! - **Credentials-safe:** `allow_credentials(true)` in combinatie met
//!   `AllowOrigin::predicate` echo't exact het Origin-header i.p.v. `*`.
//!   Een wildcard zou sowieso incompatibel zijn met credentials.
//! - **Malformed input:** ongeldige YAML of ongeldige origins worden
//!   per-tenant / per-entry geskipped met een `tracing::warn!`. Backend
//!   start altijd door.
//! - **Reload:** alleen bij backend-startup. Wijzigingen in `tenant.yaml`
//!   vergen een container-restart (conform `_schema.md` §Reload-semantiek).

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, HeaderName, HeaderValue, Method};
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

// ---------------------------------------------------------------------------
// YAML schema types (subset — we reading alleen wat we voor CORS nodig hebben)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TenantYaml {
    #[allow(dead_code)]
    slug: Option<String>,
    /// Default `true` als het veld ontbreekt (zie `_schema.md`).
    active: Option<bool>,
    cors: Option<CorsSection>,
}

#[derive(Debug, Deserialize)]
struct CorsSection {
    allowed_origins: Option<Vec<String>>,
    allowed_origins_dev: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Origin validation
// ---------------------------------------------------------------------------

/// Valideer een enkele origin-string volgens `_schema.md` §Validatie-regels.
///
/// Regels:
/// - Protocol `http://` of `https://`.
/// - Geen trailing slash.
/// - Geen wildcards.
/// - Lowercase.
///
/// Returns `true` als de origin geldig is. Bij `false` wordt een warning
/// gelogd door de caller met tenant-context.
fn is_valid_origin(origin: &str) -> bool {
    if origin.is_empty() {
        return false;
    }
    if !(origin.starts_with("http://") || origin.starts_with("https://")) {
        return false;
    }
    if origin.ends_with('/') {
        return false;
    }
    if origin.contains('*') {
        return false;
    }
    if origin != origin.to_lowercase() {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Scan `tenants_root` voor subdirectories en bouw de union-set van toegestane
/// CORS-origins.
///
/// Gedrag:
/// - Directory moet bestaan; anders → lege set + warning.
/// - Subdirs met naam beginnend op `.` of `_` (bv. `_schema.md`) worden
///   geskipped — conventie uit Reports-referentie.
/// - Ontbrekend `tenant.yaml` → per-tenant warning + skip.
/// - Malformed YAML of `tenant.yaml` is geen mapping → behandel als
///   `active: false` (warning + skip).
/// - `active: false` → skip zonder warning (expliciet gedrag).
/// - Ongeldige origin (protocol, slash, wildcard, casing) → per-entry
///   warning + skip. De rest van de tenant wordt wel geladen.
/// - Bij `include_dev = true` worden `allowed_origins_dev` mee-ge-unioned.
///
/// Faalt nooit — gebruiker kan altijd opstarten.
pub fn load_tenant_origins(tenants_root: &Path, include_dev: bool) -> HashSet<String> {
    let mut origins: HashSet<String> = HashSet::new();

    let entries = match std::fs::read_dir(tenants_root) {
        Ok(rd) => rd,
        Err(err) => {
            tracing::warn!(
                tenants_root = %tenants_root.display(),
                error = %err,
                "kan tenants-directory niet lezen — geen tenant-origins geladen"
            );
            return origins;
        }
    };

    // Deterministische volgorde voor reproduceerbare logs.
    let mut dirs: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    for dir in dirs {
        let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Skip verborgen en `_` prefixed dirs.
        if dir_name.starts_with('.') || dir_name.starts_with('_') {
            continue;
        }

        let yaml_path = dir.join("tenant.yaml");
        if !yaml_path.exists() {
            tracing::warn!(
                tenant = %dir_name,
                "geen tenant.yaml gevonden — CORS-config voor deze tenant geskipped"
            );
            continue;
        }

        let raw = match std::fs::read_to_string(&yaml_path) {
            Ok(s) => s,
            Err(err) => {
                tracing::warn!(
                    tenant = %dir_name,
                    error = %err,
                    "tenant.yaml niet leesbaar — behandel als inactive"
                );
                continue;
            }
        };

        let parsed: TenantYaml = match serde_yml::from_str(&raw) {
            Ok(t) => t,
            Err(err) => {
                tracing::warn!(
                    tenant = %dir_name,
                    error = %err,
                    "tenant.yaml is malformed — behandel als inactive"
                );
                continue;
            }
        };

        // active default = true (zie _schema.md §Fallback-gedrag).
        if parsed.active == Some(false) {
            tracing::debug!(tenant = %dir_name, "tenant active=false — skip");
            continue;
        }

        let Some(cors) = parsed.cors else {
            tracing::debug!(
                tenant = %dir_name,
                "tenant.yaml heeft geen cors block — geen origins geladen"
            );
            continue;
        };

        let mut lists: Vec<Vec<String>> = Vec::with_capacity(2);
        if let Some(prod) = cors.allowed_origins {
            lists.push(prod);
        }
        if include_dev {
            if let Some(dev) = cors.allowed_origins_dev {
                lists.push(dev);
            }
        }

        let mut accepted = 0usize;
        for list in lists {
            for origin in list {
                if is_valid_origin(&origin) {
                    origins.insert(origin);
                    accepted += 1;
                } else {
                    tracing::warn!(
                        tenant = %dir_name,
                        origin = %origin,
                        "origin faalt validatie (protocol/slash/wildcard/casing) — skip"
                    );
                }
            }
        }

        tracing::info!(
            tenant = %dir_name,
            count = accepted,
            "tenant CORS-origins geladen"
        );
    }

    origins
}

// ---------------------------------------------------------------------------
// CorsLayer builder
// ---------------------------------------------------------------------------

/// Custom forward_auth + tenant headers die door de frontend gestuurd mogen
/// worden. Blijft in sync met de lijst in `auth.rs` en met de Caddy-config.
fn custom_auth_headers() -> Vec<HeaderName> {
    vec![
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        HeaderName::from_static("x-authentik-username"),
        HeaderName::from_static("x-authentik-email"),
        HeaderName::from_static("x-authentik-name"),
        HeaderName::from_static("x-authentik-uid"),
        HeaderName::from_static("x-authentik-groups"),
        HeaderName::from_static("x-authentik-meta-tenant"),
        HeaderName::from_static("x-authentik-meta-company"),
        HeaderName::from_static("x-authentik-meta-jobtitle"),
        HeaderName::from_static("x-authentik-meta-phone"),
        HeaderName::from_static("x-authentik-meta-regnumber"),
        HeaderName::from_static("x-original-tenant"),
    ]
}

/// Bouw een `CorsLayer` rondom de gegeven origin-set.
///
/// Semantiek:
/// - `origins` niet-leeg → `AllowOrigin::predicate` met exact-match tegen
///   de set. `allow_credentials(true)` echo't het Origin-header bij match.
/// - `origins` leeg, `fallback` niet-leeg → gebruik `fallback` als
///   predicate-set (backward-compat met bestaande `CORS_ORIGINS` env).
/// - Beide leeg → warning + `AllowOrigin::any()` (mirror_request fout,
///   combineerbaar met credentials? nee; daarom kiezen we `any()` en
///   zetten `allow_credentials(false)` in dat pad om niet te crashen).
///
/// De predicate sluit een `Arc<HashSet<String>>` in zodat de layer
/// `Clone + Send + Sync` blijft.
pub fn build_cors_layer(origins: HashSet<String>, fallback: Vec<String>) -> CorsLayer {
    // Bepaal welke set we gaan gebruiken + welke bron we loggen.
    let (effective, source): (HashSet<String>, &str) = if !origins.is_empty() {
        (origins, "tenants")
    } else {
        let filtered: HashSet<String> = fallback
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| is_valid_origin(s))
            .collect();
        if !filtered.is_empty() {
            (filtered, "fallback")
        } else {
            (HashSet::new(), "permissive")
        }
    };

    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
        Method::PATCH,
    ];
    let allow_headers = custom_auth_headers();

    if source == "permissive" {
        tracing::warn!(
            "CORS: geen tenant-origins en geen fallback — gebruik permissive Any (credentials uit)"
        );
        return CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods(methods)
            .allow_headers(allow_headers)
            .max_age(Duration::from_secs(600));
    }

    tracing::info!(
        source = source,
        count = effective.len(),
        "CORS layer: {} origins uit bron '{}'",
        effective.len(),
        source
    );

    let allowed = Arc::new(effective);
    let allowed_for_closure = Arc::clone(&allowed);
    let predicate = AllowOrigin::predicate(move |origin: &HeaderValue, _parts| {
        // HeaderValue → &str; invalide UTF-8 origin = geen match.
        match origin.to_str() {
            Ok(s) => allowed_for_closure.contains(s),
            Err(_) => false,
        }
    });

    CorsLayer::new()
        .allow_origin(predicate)
        .allow_methods(methods)
        .allow_headers(allow_headers)
        .allow_credentials(true)
        .max_age(Duration::from_secs(600))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Write een tenant.yaml in `root/<slug>/tenant.yaml`.
    fn write_tenant(root: &Path, slug: &str, yaml: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tenant.yaml"), yaml).unwrap();
    }

    #[test]
    fn test_load_tenant_origins_skips_inactive() {
        let tmp = TempDir::new().unwrap();
        write_tenant(
            tmp.path(),
            "active_one",
            r#"
slug: active_one
active: true
cors:
  allowed_origins:
    - https://active.example.com
  allowed_origins_dev: []
"#,
        );
        write_tenant(
            tmp.path(),
            "inactive_one",
            r#"
slug: inactive_one
active: false
cors:
  allowed_origins:
    - https://inactive.example.com
  allowed_origins_dev: []
"#,
        );

        let origins = load_tenant_origins(tmp.path(), false);
        assert!(origins.contains("https://active.example.com"));
        assert!(!origins.contains("https://inactive.example.com"));
        assert_eq!(origins.len(), 1);
    }

    #[test]
    fn test_load_tenant_origins_skips_malformed_yaml() {
        let tmp = TempDir::new().unwrap();
        // Malformed — unterminated mapping.
        write_tenant(
            tmp.path(),
            "broken",
            "slug: broken\ncors:\n  allowed_origins: [this is not: valid yaml",
        );
        write_tenant(
            tmp.path(),
            "ok",
            r#"
slug: ok
active: true
cors:
  allowed_origins:
    - https://ok.example.com
  allowed_origins_dev: []
"#,
        );

        let origins = load_tenant_origins(tmp.path(), false);
        assert_eq!(origins.len(), 1);
        assert!(origins.contains("https://ok.example.com"));
    }

    #[test]
    fn test_origin_validation_skips_invalid_per_entry() {
        let tmp = TempDir::new().unwrap();
        write_tenant(
            tmp.path(),
            "mixed",
            r#"
slug: mixed
active: true
cors:
  allowed_origins:
    - https://good.example.com
    - https://bad-trailing.example.com/
    - "https://*.wildcard.example.com"
    - no-protocol.example.com
    - HTTPS://UPPER.EXAMPLE.COM
    - https://also-good.example.com
  allowed_origins_dev: []
"#,
        );

        let origins = load_tenant_origins(tmp.path(), false);
        // Alleen de twee geldige origins blijven over.
        assert_eq!(origins.len(), 2, "origins = {origins:?}");
        assert!(origins.contains("https://good.example.com"));
        assert!(origins.contains("https://also-good.example.com"));
        assert!(!origins.contains("https://bad-trailing.example.com/"));
        assert!(!origins.contains("no-protocol.example.com"));
    }

    #[test]
    fn test_include_dev_flag() {
        let tmp = TempDir::new().unwrap();
        write_tenant(
            tmp.path(),
            "t",
            r#"
slug: t
active: true
cors:
  allowed_origins:
    - https://prod.example.com
  allowed_origins_dev:
    - http://localhost:5173
    - http://127.0.0.1:5173
"#,
        );

        let prod_only = load_tenant_origins(tmp.path(), false);
        assert_eq!(prod_only.len(), 1);
        assert!(prod_only.contains("https://prod.example.com"));

        let with_dev = load_tenant_origins(tmp.path(), true);
        assert_eq!(with_dev.len(), 3);
        assert!(with_dev.contains("https://prod.example.com"));
        assert!(with_dev.contains("http://localhost:5173"));
        assert!(with_dev.contains("http://127.0.0.1:5173"));
    }

    #[test]
    fn test_load_tenant_origins_missing_yaml_and_underscore_dirs() {
        let tmp = TempDir::new().unwrap();
        // Directory met tenant.yaml
        write_tenant(
            tmp.path(),
            "real",
            r#"
slug: real
active: true
cors:
  allowed_origins:
    - https://real.example.com
  allowed_origins_dev: []
"#,
        );
        // Directory zonder tenant.yaml
        fs::create_dir_all(tmp.path().join("empty_dir")).unwrap();
        // `_docs` — moet geskipped worden door prefix-regel.
        fs::create_dir_all(tmp.path().join("_docs")).unwrap();
        fs::write(
            tmp.path().join("_docs").join("tenant.yaml"),
            r#"
slug: _docs
active: true
cors:
  allowed_origins:
    - https://should-not-load.example.com
  allowed_origins_dev: []
"#,
        )
        .unwrap();

        let origins = load_tenant_origins(tmp.path(), false);
        assert_eq!(origins.len(), 1);
        assert!(origins.contains("https://real.example.com"));
        assert!(!origins.contains("https://should-not-load.example.com"));
    }

    #[test]
    fn test_load_tenant_origins_nonexistent_root() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("does-not-exist");
        let origins = load_tenant_origins(&missing, true);
        assert!(origins.is_empty());
    }

    #[test]
    fn test_build_cors_layer_with_empty_falls_back_to_static_list() {
        // Smoke: moet niet panicen en een valid CorsLayer opleveren.
        // We kunnen de interne state niet introspecten zonder downstream
        // te runnen; dus we controleren dat geen van de aanroepen panic'en
        // en dat de layer compileert als tower::Layer.
        let empty: HashSet<String> = HashSet::new();
        let fallback = vec![
            "http://localhost:5173".to_string(),
            "http://localhost:1420".to_string(),
            "not-valid".to_string(), // wordt gefilterd
        ];
        let _layer = build_cors_layer(empty, fallback);

        // Beide leeg → permissive pad, ook geen panic.
        let _permissive = build_cors_layer(HashSet::new(), Vec::new());

        // Tenants-set aanwezig → predicate pad.
        let tenants: HashSet<String> =
            ["https://report.open-aec.com".to_string()].into_iter().collect();
        let _tenant_layer = build_cors_layer(tenants, vec!["http://localhost:5173".to_string()]);
    }

    /// Integratie-test tegen de echte `openaec-tenants` repo.
    ///
    /// `#[ignore]` standaard — alleen draaien met
    /// `cargo test -p isso51-api -- --ignored real_tenants_repo`.
    /// Test valideert dat de 3 actieve tenants (3bm, symitech,
    /// openaec_foundation) geladen worden en dat de inactieve
    /// `test_tenant_brand` wordt overgeslagen.
    #[test]
    #[ignore]
    fn real_tenants_repo_loads_expected_origins() {
        let root = std::path::Path::new("C:/GitHub/openaec-tenants/tenants");
        if !root.exists() {
            eprintln!("tenants repo niet aanwezig — skip");
            return;
        }

        let prod_only = load_tenant_origins(root, false);
        assert!(
            prod_only.contains("https://report.open-aec.com"),
            "3BM prod origin ontbreekt: {prod_only:?}"
        );
        assert!(
            prod_only.contains("https://mockup.symitech.nl"),
            "Symitech prod origin ontbreekt: {prod_only:?}"
        );
        // Inactieve test_tenant_brand mag NIET bijdragen.
        for origin in &prod_only {
            assert!(
                !origin.contains("test_tenant"),
                "inactive tenant lekte door: {origin}"
            );
        }

        let with_dev = load_tenant_origins(root, true);
        assert!(with_dev.contains("http://localhost:5173"));
        assert!(with_dev.len() > prod_only.len());
    }

    #[test]
    fn test_is_valid_origin_rules() {
        assert!(is_valid_origin("https://report.open-aec.com"));
        assert!(is_valid_origin("http://localhost:5173"));
        assert!(is_valid_origin("http://127.0.0.1:5173"));

        assert!(!is_valid_origin(""));
        assert!(!is_valid_origin("report.open-aec.com"));
        assert!(!is_valid_origin("ftp://report.open-aec.com"));
        assert!(!is_valid_origin("https://report.open-aec.com/"));
        assert!(!is_valid_origin("https://*.open-aec.com"));
        assert!(!is_valid_origin("HTTPS://Report.Open-AEC.COM"));
    }
}
