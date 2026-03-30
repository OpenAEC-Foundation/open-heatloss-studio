//! Cloud storage handlers — browse projects and save calculations to Nextcloud.
//!
//! All routes require authentication. The tenant is resolved from the
//! `DEFAULT_TENANT` env var (per-token tenant claim support can be added later).

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthClaims;
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
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<CloudStatusResponse>, ApiError> {
    let client = state.cloud_client(None);

    match client {
        Some(c) => {
            let available = c.is_available().await;
            Ok(Json(CloudStatusResponse {
                available,
                tenant: state.default_tenant.clone(),
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
    AuthClaims(_claims): AuthClaims,
) -> Result<Json<Vec<CloudProjectResponse>>, ApiError> {
    let client = state
        .cloud_client(None)
        .ok_or_else(|| ApiError::ServiceUnavailable("Cloud storage niet geconfigureerd".into()))?;

    if !client.is_available().await {
        return Err(ApiError::ServiceUnavailable(
            "Cloud storage niet beschikbaar".into(),
        ));
    }

    let projects = client.list_projects();
    let mut result = Vec::with_capacity(projects.len());

    for p in projects {
        let has_manifest = client.volume.read_manifest(&p.name).is_some();
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
    AuthClaims(_claims): AuthClaims,
    Path(project): Path<String>,
) -> Result<Json<Vec<CloudFileResponse>>, ApiError> {
    let client = state
        .cloud_client(None)
        .ok_or_else(|| ApiError::ServiceUnavailable("Cloud storage niet geconfigureerd".into()))?;

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
    AuthClaims(_claims): AuthClaims,
    Path(project): Path<String>,
) -> Result<Json<Vec<CloudFileResponse>>, ApiError> {
    let client = state
        .cloud_client(None)
        .ok_or_else(|| ApiError::ServiceUnavailable("Cloud storage niet geconfigureerd".into()))?;

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
    AuthClaims(_claims): AuthClaims,
    Path(project): Path<String>,
    Json(body): Json<SaveCalculationRequest>,
) -> Result<Json<SaveCalculationResponse>, ApiError> {
    let client = state
        .cloud_client(None)
        .ok_or_else(|| ApiError::ServiceUnavailable("Cloud storage niet geconfigureerd".into()))?;

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
        .upsert_manifest_object(&project, manifest_object)
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
}
