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
    /// Optional: if provided, the server checks this matches the current `updated_at`.
    /// Returns 409 Conflict if they differ (optimistic concurrency control).
    pub expected_updated_at: Option<String>,
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
    // Ensure user record exists (FK constraint on projects.user_id).
    ensure_user(&state, &claims).await?;

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
///
/// Combines ownership verification and the mutation into a single atomic
/// `UPDATE ... WHERE id = ? AND user_id = ?`, eliminating the TOCTOU window
/// between a separate SELECT ownership check and the UPDATE. Both `name` and
/// `project_data` are written in one statement so a partial write can never
/// be observed.
///
/// Behaviour:
/// - Row not found (or soft-deleted, or owned by another user) → 404. We do
///   not distinguish 403 from 404 on purpose to avoid leaking project
///   existence to non-owners.
/// - `expected_updated_at` mismatch → 409 Conflict (optimistic concurrency).
/// - Nothing to update (no `name`, no `project_data`) → 400.
pub async fn update_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Optimistic concurrency check still needs a read, but it is *only* a
    // pre-check — the authoritative ownership+existence check is the
    // rows_affected result of the UPDATE below. The read here does not
    // gate the write, so a concurrent owner change cannot slip an update
    // through: the WHERE-clause enforces ownership at commit time.
    if let Some(expected) = &body.expected_updated_at {
        let current = sqlx::query_scalar::<_, String>(
            "SELECT updated_at FROM projects
             WHERE id = ?1 AND user_id = ?2 AND is_archived = 0",
        )
        .bind(&project_id)
        .bind(&claims.sub)
        .fetch_optional(&state.db)
        .await?;

        match current {
            Some(updated_at) if updated_at != *expected => {
                return Ok((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "detail": "Project is elders gewijzigd",
                        "server_updated_at": updated_at
                    })),
                ));
            }
            Some(_) => { /* matches — fall through to UPDATE */ }
            None => {
                return Err(ApiError::NotFound("Project niet gevonden".to_string()));
            }
        }
    }

    // Reject no-op updates explicitly — otherwise a bogus call would
    // silently return 200 without touching the row.
    if body.name.is_none() && body.project_data.is_none() {
        return Err(ApiError::Internal(
            "Update vereist ten minste 'name' of 'project_data'".to_string(),
        ));
    }

    // Serialize project_data up front so any JSON error surfaces before
    // the SQL call (and the Option<String> lives through the bind).
    let project_data_json = match &body.project_data {
        Some(value) => Some(
            serde_json::to_string(value)
                .map_err(|e| ApiError::Internal(e.to_string()))?,
        ),
        None => None,
    };

    // Atomic single-statement UPDATE. COALESCE leaves the column untouched
    // when the caller did not provide a new value (NULL bind). Ownership
    // and soft-delete are enforced in the WHERE clause, so rows_affected
    // tells us authoritatively whether the user had access.
    let result = sqlx::query(
        "UPDATE projects
            SET name         = COALESCE(?1, name),
                project_data = COALESCE(?2, project_data),
                updated_at   = datetime('now')
          WHERE id = ?3
            AND user_id = ?4
            AND is_archived = 0",
    )
    .bind(body.name.as_deref())
    .bind(project_data_json.as_deref())
    .bind(&project_id)
    .bind(&claims.sub)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        // Row either does not exist, is archived, or is owned by someone
        // else. We collapse all three into 404 to avoid leaking ownership.
        return Err(ApiError::NotFound("Project niet gevonden".to_string()));
    }

    // Fetch the new updated_at to return to the client.
    let new_updated_at = sqlx::query_scalar::<_, String>(
        "SELECT updated_at FROM projects WHERE id = ?1",
    )
    .bind(&project_id)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "updated_at": new_updated_at })),
    ))
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
// Helpers
// ---------------------------------------------------------------------------

/// Ensure the authenticated user exists in the `users` table (upsert).
///
/// This prevents FOREIGN KEY violations when creating projects before
/// the frontend has called `GET /me`.
async fn ensure_user(
    state: &AppState,
    claims: &crate::auth::OidcClaims,
) -> Result<(), ApiError> {
    let email = claims.email.as_deref().unwrap_or("");
    let name = claims.name.as_deref().unwrap_or("");
    let preferred_username = claims.preferred_username.as_deref().unwrap_or("");
    let issuer = claims.iss.as_deref().unwrap_or("");

    sqlx::query(
        "INSERT INTO users (id, email, name, preferred_username, oidc_issuer)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
           email = excluded.email,
           name = excluded.name,
           preferred_username = excluded.preferred_username,
           last_login_at = datetime('now')",
    )
    .bind(&claims.sub)
    .bind(email)
    .bind(name)
    .bind(preferred_username)
    .bind(issuer)
    .execute(&state.db)
    .await?;

    Ok(())
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

