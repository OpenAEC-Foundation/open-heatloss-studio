//! Cloud storage handlers — browse projects and save calculations to Nextcloud.
//!
//! All routes require authentication. The tenant is resolved from the
//! authenticated user's tenant claim (`X-Authentik-Meta-Tenant` /
//! `attributes.tenant`); zonder claim valt dit terug op de expliciete
//! `DEFAULT_TENANT` env-var — zie [`resolve_cloud_client`].

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{AuthClaims, OidcClaims};
use crate::error::ApiError;
use crate::state::AppState;

// ── Response types ──────────────────────────────────────────────

/// Cloud status response.
#[derive(Serialize)]
pub struct CloudStatusResponse {
    pub available: bool,
    pub tenant: Option<String>,
    pub volume_mounted: bool,
}

/// A project folder from cloud storage.
#[derive(Serialize)]
pub struct CloudProjectResponse {
    pub name: String,
    pub has_manifest: bool,
}

/// A file entry from cloud storage.
#[derive(Serialize)]
pub struct CloudFileResponse {
    pub name: String,
    pub size: u64,
    pub last_modified: String,
}

/// Request body for saving a calculation.
#[derive(Deserialize)]
pub struct SaveCalculationRequest {
    /// Filename for the calculation (without path). E.g. `"berekening-001.json"`.
    pub filename: String,
    /// The calculation data to save.
    pub data: serde_json::Value,
    /// Optional project name (used in the manifest entry).
    pub project_name: Option<String>,
}

/// Response after saving a calculation.
#[derive(Serialize)]
pub struct SaveCalculationResponse {
    pub ok: bool,
    pub path: String,
    pub manifest_guid: String,
}

// ── Handlers ────────────────────────────────────────────────────

/// `GET /api/v1/cloud/status` — Check if cloud storage is available.
pub async fn cloud_status(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<CloudStatusResponse>, ApiError> {
    let tenant_slug = resolved_tenant_slug(&state, &claims);
    let client = state.cloud_client(claims.tenant.as_deref());

    match client {
        Some(c) => {
            let available = c.is_available().await;
            Ok(Json(CloudStatusResponse {
                available,
                tenant: tenant_slug.map(str::to_string),
                volume_mounted: c.volume.available(),
            }))
        }
        None => Ok(Json(CloudStatusResponse {
            available: false,
            tenant: None,
            volume_mounted: false,
        })),
    }
}

/// `GET /api/v1/cloud/projects` — List project folders.
pub async fn cloud_list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<CloudProjectResponse>>, ApiError> {
    let client = resolve_cloud_client(&state, &claims)?;

    if !client.is_available().await {
        return Err(ApiError::ServiceUnavailable(
            "Cloud storage niet beschikbaar".into(),
        ));
    }

    let projects = client.list_projects();
    let mut result = Vec::with_capacity(projects.len());

    for p in projects {
        let has_manifest = client.volume.read_manifest(&p.name, "project.wefc").is_some();
        result.push(CloudProjectResponse {
            name: p.name,
            has_manifest,
        });
    }

    Ok(Json(result))
}

/// `GET /api/v1/cloud/projects/{project}/models` — List IFC model files.
pub async fn cloud_list_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project): Path<String>,
) -> Result<Json<Vec<CloudFileResponse>>, ApiError> {
    validate_project_name(&project)?;
    let client = resolve_cloud_client(&state, &claims)?;

    if !client.project_exists(&project) {
        return Err(ApiError::NotFound(format!(
            "Project '{project}' niet gevonden in cloud storage"
        )));
    }

    let files = client.list_models(&project);
    let result = files
        .into_iter()
        .map(|f| CloudFileResponse {
            name: f.name,
            size: f.size,
            last_modified: f.last_modified,
        })
        .collect();

    Ok(Json(result))
}

/// `GET /api/v1/cloud/projects/{project}/calculations` — List calculation files.
pub async fn cloud_list_calculations(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project): Path<String>,
) -> Result<Json<Vec<CloudFileResponse>>, ApiError> {
    validate_project_name(&project)?;
    let client = resolve_cloud_client(&state, &claims)?;

    if !client.project_exists(&project) {
        return Err(ApiError::NotFound(format!(
            "Project '{project}' niet gevonden in cloud storage"
        )));
    }

    let files = client.list_files(&project);
    let result = files
        .into_iter()
        .map(|f| CloudFileResponse {
            name: f.name,
            size: f.size,
            last_modified: f.last_modified,
        })
        .collect();

    Ok(Json(result))
}

/// `POST /api/v1/cloud/projects/{project}/save` — Save a calculation to cloud.
///
/// Uploads the calculation JSON to `calculations/{filename}` and updates
/// the project manifest (`project.wefc`) with a `WefcCalculation` entry.
pub async fn cloud_save_calculation(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project): Path<String>,
    Json(body): Json<SaveCalculationRequest>,
) -> Result<Json<SaveCalculationResponse>, ApiError> {
    validate_project_name(&project)?;
    let client = resolve_cloud_client(&state, &claims)?;

    if !client.is_available().await {
        return Err(ApiError::ServiceUnavailable(
            "Cloud storage niet beschikbaar".into(),
        ));
    }

    // Sanitize filename — only allow alphanumeric, dashes, underscores, dots.
    let filename = sanitize_filename(&body.filename);
    if filename.is_empty() {
        return Err(ApiError::Internal("Ongeldige bestandsnaam".into()));
    }

    // Ensure .json extension.
    let filename = if filename.ends_with(".json") {
        filename
    } else {
        format!("{filename}.json")
    };

    // Serialize calculation data.
    let data = serde_json::to_vec_pretty(&body.data)
        .map_err(|e| ApiError::Internal(format!("JSON serialisatie mislukt: {e}")))?;

    // Upload to calculations/{filename} via WebDAV.
    client
        .upload_file(&project, &filename, data)
        .await
        .map_err(|e| ApiError::Internal(format!("Upload mislukt: {e}")))?;

    let output_dir = openaec_cloud::output_dir_for_tool(
        crate::state::TOOL_SLUG,
    );
    let file_path = format!("{output_dir}/{filename}");

    // Build manifest entry.
    let guid = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let display_name = body
        .project_name
        .as_deref()
        .unwrap_or(&project);

    let manifest_object = serde_json::json!({
        "type": "WefcCalculation",
        "guid": guid,
        "name": format!("Warmteverliesberekening - {display_name}"),
        "path": file_path,
        "calculationType": "heatLoss",
        "status": "active",
        "created": now,
        "modified": now,
    });

    // Update manifest (read → merge → write).
    client
        .upsert_default_manifest_object(&project, manifest_object)
        .await
        .map_err(|e| {
            tracing::warn!(
                project = %project,
                error = %e,
                "manifest update failed — calculation was saved but manifest not updated"
            );
            ApiError::Internal(format!("Manifest update mislukt: {e}"))
        })?;

    tracing::info!(
        project = %project,
        filename = %filename,
        guid = %guid,
        "calculation saved to cloud storage"
    );

    Ok(Json(SaveCalculationResponse {
        ok: true,
        path: file_path,
        manifest_guid: guid,
    }))
}

// ── Helpers ─────────────────────────────────────────────────────

/// Resolve de tenant-slug voor de geauthenticeerde gebruiker.
///
/// Tenant-claim uit de auth (forward_auth header of Authentik
/// `attributes.tenant`) gaat vóór; zonder claim valt dit terug op
/// `DEFAULT_TENANT`.
fn resolved_tenant_slug<'a>(state: &'a AppState, claims: &'a OidcClaims) -> Option<&'a str> {
    claims.tenant.as_deref().or(state.default_tenant.as_deref())
}

/// Resolve een [`openaec_cloud::CloudClient`] voor de geauthenticeerde
/// gebruiker.
///
/// Keuze (audit-fix 2026-06-10): de tenant komt uit de **auth-claims**, niet
/// uit een vaste `DEFAULT_TENANT`. Zonder tenant-claim vallen we terug op de
/// expliciete `DEFAULT_TENANT` env-var in plaats van 403 te geven — motivatie:
/// single-tenant deployments zetten `DEFAULT_TENANT` bewust als infra-opt-in
/// voor gebruikers zonder Authentik tenant-attribute. Is er géén claim én géén
/// `DEFAULT_TENANT` (of is de claim-slug onbekend in de registry), dan is er
/// geen tenant-storage → 503, nooit impliciet andermans storage.
fn resolve_cloud_client(
    state: &AppState,
    claims: &OidcClaims,
) -> Result<openaec_cloud::CloudClient, ApiError> {
    state
        .cloud_client(claims.tenant.as_deref())
        .ok_or_else(|| {
            tracing::warn!(
                user = %claims.sub,
                tenant_claim = claims.tenant.as_deref().unwrap_or("<geen>"),
                "cloud storage niet beschikbaar voor tenant van gebruiker"
            );
            ApiError::ServiceUnavailable("Cloud storage niet geconfigureerd".into())
        })
}

/// Valideer een `{project}` path-parameter tegen directory-traversal.
///
/// Weigert lege namen, padscheiders (`/`, `\`), `..`-segmenten, namen die
/// met een punt beginnen (verborgen mappen, `.`/`..`) en control-characters.
/// De parameter wordt verderop als directory-naam binnen de tenant-root
/// gebruikt; alles wat daarbuiten kan wijzen is een 400.
fn validate_project_name(name: &str) -> Result<(), ApiError> {
    let valid = !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && !name.contains(['/', '\\'])
        && !name.contains("..")
        && !name.chars().any(char::is_control);

    if valid {
        Ok(())
    } else {
        // Bewust zonder echo van de input — geen reflectie van attacker-data.
        Err(ApiError::BadRequest("Ongeldige projectnaam".into()))
    }
}

/// Remove any characters that could cause path traversal or OS issues.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_path_traversal() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("normal-file.json"), "normal-file.json");
        assert_eq!(
            sanitize_filename("my calc (1).json"),
            "mycalc1.json"
        );
    }

    #[test]
    fn sanitize_strips_leading_dots() {
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("...dots"), "dots");
    }

    #[test]
    fn sanitize_empty_input() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("../.."), "");
    }

    // --- validate_project_name (directory-traversal guard) ---------------

    #[test]
    fn project_name_accepts_normal_names() {
        assert!(validate_project_name("Project Aldlânstate 12").is_ok());
        assert!(validate_project_name("2026-042_woonhuis").is_ok());
        assert!(validate_project_name("a").is_ok());
    }

    #[test]
    fn project_name_rejects_traversal_segments() {
        assert!(validate_project_name("..").is_err());
        assert!(validate_project_name("../etc").is_err());
        assert!(validate_project_name("a/../b").is_err());
        assert!(validate_project_name("a..b").is_err());
        assert!(validate_project_name("..\\windows").is_err());
    }

    #[test]
    fn project_name_rejects_path_separators_and_absolute_paths() {
        assert!(validate_project_name("a/b").is_err());
        assert!(validate_project_name("/etc/passwd").is_err());
        assert!(validate_project_name("a\\b").is_err());
        assert!(validate_project_name("C:\\Windows").is_err());
    }

    #[test]
    fn project_name_rejects_empty_hidden_and_control_chars() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name(".").is_err());
        assert!(validate_project_name(".verborgen").is_err());
        assert!(validate_project_name("naam\u{0}met-nul").is_err());
        assert!(validate_project_name("naam\nmet-newline").is_err());
        assert!(validate_project_name(&"x".repeat(256)).is_err());
    }
}
