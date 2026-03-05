//! Project CRUD handlers (auth required).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthClaims;
use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Request body for creating a project.
#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: Option<String>,
    pub project_data: serde_json::Value,
}

/// Request body for updating a project.
#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub project_data: Option<serde_json::Value>,
}

/// Summary returned in project list.
#[derive(Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub has_result: bool,
}

/// Full project response.
#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub project_data: serde_json::Value,
    pub result_data: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /projects — List all non-archived projects for the authenticated user.
pub async fn list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<ProjectSummary>>, ApiError> {
    let rows = sqlx::query_as::<_, ProjectListRow>(
        "SELECT id, name, updated_at, result_data IS NOT NULL as has_result
         FROM projects
         WHERE user_id = ?1 AND is_archived = 0
         ORDER BY updated_at DESC",
    )
    .bind(&claims.sub)
    .fetch_all(&state.db)
    .await?;

    let summaries = rows
        .into_iter()
        .map(|r| ProjectSummary {
            id: r.id,
            name: r.name,
            updated_at: r.updated_at,
            has_result: r.has_result,
        })
        .collect();

    Ok(Json(summaries))
}

/// POST /projects — Create a new project.
pub async fn create_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(body): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let id = Uuid::new_v4().to_string();
    let name = body.name.unwrap_or_else(|| "Naamloos project".to_string());
    let project_data = serde_json::to_string(&body.project_data)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    sqlx::query(
        "INSERT INTO projects (id, user_id, name, project_data)
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&id)
    .bind(&claims.sub)
    .bind(&name)
    .bind(&project_data)
    .execute(&state.db)
    .await?;

    let response = serde_json::json!({ "id": id, "name": name });
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /projects/:id — Get a single project (ownership check).
pub async fn get_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let row = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, name, project_data, result_data, created_at, updated_at
         FROM projects
         WHERE id = ?1 AND is_archived = 0",
    )
    .bind(&project_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project niet gevonden".to_string()))?;

    if row.user_id != claims.sub {
        return Err(ApiError::Forbidden(
            "Geen toegang tot dit project".to_string(),
        ));
    }

    let project_data: serde_json::Value = serde_json::from_str(&row.project_data)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let result_data: Option<serde_json::Value> = row
        .result_data
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ProjectResponse {
        id: row.id,
        name: row.name,
        project_data,
        result_data,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

/// PUT /projects/:id — Update a project.
pub async fn update_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify ownership.
    let owner = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM projects WHERE id = ?1 AND is_archived = 0",
    )
    .bind(&project_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project niet gevonden".to_string()))?;

    if owner != claims.sub {
        return Err(ApiError::Forbidden(
            "Geen toegang tot dit project".to_string(),
        ));
    }

    if let Some(name) = &body.name {
        sqlx::query("UPDATE projects SET name = ?1, updated_at = datetime('now') WHERE id = ?2")
            .bind(name)
            .bind(&project_id)
            .execute(&state.db)
            .await?;
    }

    if let Some(project_data) = &body.project_data {
        let json_str = serde_json::to_string(project_data)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        sqlx::query(
            "UPDATE projects SET project_data = ?1, updated_at = datetime('now') WHERE id = ?2",
        )
        .bind(&json_str)
        .bind(&project_id)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// DELETE /projects/:id — Soft-delete a project (set is_archived = 1).
pub async fn delete_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let owner = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM projects WHERE id = ?1 AND is_archived = 0",
    )
    .bind(&project_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project niet gevonden".to_string()))?;

    if owner != claims.sub {
        return Err(ApiError::Forbidden(
            "Geen toegang tot dit project".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE projects SET is_archived = 1, updated_at = datetime('now') WHERE id = ?1",
    )
    .bind(&project_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// POST /projects/:id/calculate — Calculate and save the result.
pub async fn calculate_and_save(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Fetch and verify ownership.
    let row = sqlx::query_as::<_, ProjectDataRow>(
        "SELECT user_id, project_data FROM projects WHERE id = ?1 AND is_archived = 0",
    )
    .bind(&project_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Project niet gevonden".to_string()))?;

    if row.user_id != claims.sub {
        return Err(ApiError::Forbidden(
            "Geen toegang tot dit project".to_string(),
        ));
    }

    // Run calculation on blocking thread.
    let project_json = row.project_data.clone();
    let result_json = tokio::task::spawn_blocking(move || {
        isso51_core::calculate_from_json(&project_json)
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .map_err(ApiError::Calculation)?;

    // Save result.
    sqlx::query(
        "UPDATE projects SET result_data = ?1, updated_at = datetime('now') WHERE id = ?2",
    )
    .bind(&result_json)
    .bind(&project_id)
    .execute(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        [("content-type", "application/json")],
        result_json,
    ))
}

// ---------------------------------------------------------------------------
// SQLx row types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct ProjectListRow {
    id: String,
    name: String,
    updated_at: String,
    has_result: bool,
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: String,
    user_id: String,
    name: String,
    project_data: String,
    result_data: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct ProjectDataRow {
    user_id: String,
    project_data: String,
}
