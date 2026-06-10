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
    //
    // Audit-fix 2026-06-10: het optimistic-locking guard zit nu óók in de
    // WHERE-clause (`?5 IS NULL OR updated_at = ?5`). De pre-SELECT hierboven
    // blijft de snelle 409-route, maar was alleen een pre-check — een
    // concurrent write tussen check en UPDATE kon een lost update opleveren.
    // Met de guard in de UPDATE zelf is de vergelijking atomair met de write.
    let result = sqlx::query(
        "UPDATE projects
            SET name         = COALESCE(?1, name),
                project_data = COALESCE(?2, project_data),
                updated_at   = datetime('now')
          WHERE id = ?3
            AND user_id = ?4
            AND is_archived = 0
            AND (?5 IS NULL OR updated_at = ?5)",
    )
    .bind(body.name.as_deref())
    .bind(project_data_json.as_deref())
    .bind(&project_id)
    .bind(&claims.sub)
    .bind(body.expected_updated_at.as_deref())
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        // Onderscheid stale optimistic-lock van niet-gevonden: bestaat de
        // rij (voor déze eigenaar) nog wel, dan is de guard de reden →
        // 409 met dezelfde response-shape als de pre-check hierboven.
        if body.expected_updated_at.is_some() {
            let current = sqlx::query_scalar::<_, String>(
                "SELECT updated_at FROM projects
                 WHERE id = ?1 AND user_id = ?2 AND is_archived = 0",
            )
            .bind(&project_id)
            .bind(&claims.sub)
            .fetch_optional(&state.db)
            .await?;

            if let Some(updated_at) = current {
                return Ok((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "detail": "Project is elders gewijzigd",
                        "server_updated_at": updated_at
                    })),
                ));
            }
        }

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

    // Run calculation on blocking thread. `project_data` kan sinds de
    // envelope-pariteit fix een opslag-envelope zijn — pak dan het kale
    // Project eruit, want de rekenkern verwacht het kale formaat.
    let project_json = extract_calculation_input(&row.project_data);
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

/// Envelope-marker zoals de frontend die schrijft (`lib/importExport.ts`).
const ENVELOPE_SCHEMA_ID: &str = "isso51-project-v1";

/// `project_data` is een opake tekst-blob voor deze API, met één
/// uitzondering: de server-side rekenroute moet het kale Project-JSON aan
/// `isso51_core` voeren. Sinds de envelope-pariteit fix schrijft de frontend
/// een opslag-envelope (`{ schema: "isso51-project-v1", project: {...}, ... }`)
/// als `project_data`; legacy rijen bevatten nog het kale Project-object.
///
/// Deze helper pakt het `project`-veld uit wanneer de envelope-marker
/// aanwezig is en geeft anders de input ongewijzigd terug (legacy rijen en
/// onparsebare data vallen door naar de bestaande foutafhandeling in de
/// rekenkern).
fn extract_calculation_input(project_data: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(project_data) {
        if value.get("schema").and_then(|s| s.as_str()) == Some(ENVELOPE_SCHEMA_ID) {
            if let Some(project) = value.get("project") {
                return project.to_string();
            }
        }
    }
    project_data.to_string()
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod optimistic_locking_tests {
    use super::*;
    use crate::auth::OidcClaims;
    use axum::response::IntoResponse;

    /// Vast beginpunt voor `updated_at` — bewust ver van `datetime('now')`
    /// zodat een geslaagde update de timestamp aantoonbaar verandert
    /// (`datetime('now')` heeft seconde-granulariteit).
    const T0: &str = "2026-01-01 10:00:00";

    async fn test_state() -> AppState {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite");
        crate::run_migrations(&db).await;
        AppState::new(
            db,
            None,
            None,
            None,
            None,
            openaec_cloud::TenantsRegistry::default(),
            None,
        )
    }

    fn user_claims(sub: &str) -> OidcClaims {
        OidcClaims {
            sub: sub.to_string(),
            ..Default::default()
        }
    }

    async fn seed_project(state: &AppState, user: &str, updated_at: &str) -> String {
        sqlx::query(
            "INSERT OR IGNORE INTO users (id, email, oidc_issuer)
             VALUES (?1, '', 'authentik')",
        )
        .bind(user)
        .execute(&state.db)
        .await
        .expect("seed user");

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO projects (id, user_id, name, project_data, updated_at)
             VALUES (?1, ?2, 'Test', '{}', ?3)",
        )
        .bind(&id)
        .bind(user)
        .bind(updated_at)
        .execute(&state.db)
        .await
        .expect("seed project");
        id
    }

    fn update_body(expected: Option<&str>) -> UpdateProjectRequest {
        UpdateProjectRequest {
            name: None,
            project_data: Some(serde_json::json!({ "info": { "name": "nieuw" } })),
            expected_updated_at: expected.map(str::to_string),
        }
    }

    async fn call_update(
        state: &AppState,
        user: &str,
        project_id: &str,
        body: UpdateProjectRequest,
    ) -> (StatusCode, serde_json::Value) {
        let result = update_project(
            State(state.clone()),
            crate::auth::AuthClaims(user_claims(user)),
            Path(project_id.to_string()),
            Json(body),
        )
        .await;

        let response = match result {
            Ok(ok) => ok.into_response(),
            Err(err) => err.into_response(),
        };
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn current_updated_at(state: &AppState, project_id: &str) -> String {
        sqlx::query_scalar::<_, String>("SELECT updated_at FROM projects WHERE id = ?1")
            .bind(project_id)
            .fetch_one(&state.db)
            .await
            .expect("updated_at")
    }

    /// Happy flow (auto-save zonder expected): blijft 200 met `ok` +
    /// `updated_at` — gedragsgelijk met de bestaande frontend-flow.
    #[tokio::test]
    async fn update_without_expected_succeeds() {
        let state = test_state().await;
        let id = seed_project(&state, "user-1", T0).await;

        let (status, body) = call_update(&state, "user-1", &id, update_body(None)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        assert!(body["updated_at"].is_string());
    }

    /// Happy flow mét expected (de normale optimistic-locking route).
    #[tokio::test]
    async fn update_with_matching_expected_succeeds() {
        let state = test_state().await;
        let id = seed_project(&state, "user-1", T0).await;

        let (status, body) = call_update(&state, "user-1", &id, update_body(Some(T0))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        // De write is daadwerkelijk doorgekomen.
        assert_ne!(current_updated_at(&state, &id).await, T0);
    }

    /// Lost-update-race door de handler heen: twee schrijvers met dezelfde
    /// `expected`. De eerste wint; de tweede (inmiddels stale) krijgt 409
    /// in dezelfde response-shape als de bestaande conflict-route en de
    /// data van de eerste schrijver blijft staan.
    #[tokio::test]
    async fn second_writer_with_stale_expected_gets_409() {
        let state = test_state().await;
        let id = seed_project(&state, "user-1", T0).await;

        // Schrijver 1: expected = T0 → slaagt, updated_at verandert.
        let (status, _) = call_update(&state, "user-1", &id, update_body(Some(T0))).await;
        assert_eq!(status, StatusCode::OK);
        let after_first = current_updated_at(&state, &id).await;

        // Schrijver 2: nog steeds expected = T0 (stale) → 409.
        let mut stale = update_body(Some(T0));
        stale.project_data = Some(serde_json::json!({ "info": { "name": "verliezer" } }));
        let (status, body) = call_update(&state, "user-1", &id, stale).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["detail"], "Project is elders gewijzigd");
        assert_eq!(body["server_updated_at"], after_first);

        // De data van schrijver 1 is niet overschreven.
        let data = sqlx::query_scalar::<_, String>(
            "SELECT project_data FROM projects WHERE id = ?1",
        )
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .expect("project_data");
        assert!(data.contains("nieuw"));
        assert!(!data.contains("verliezer"));
    }

    /// De race-window zelf: de pre-SELECT is in een unit-test niet te
    /// interleaven met een concurrent write, dus we bewijzen de atomaire
    /// guard op statement-niveau — exact het UPDATE-statement uit
    /// [`update_project`] met een stale `?5` raakt 0 rijen, met een
    /// kloppende `?5` raakt het 1 rij.
    #[tokio::test]
    async fn update_statement_guard_rejects_stale_expected_atomically() {
        let state = test_state().await;
        let id = seed_project(&state, "user-1", T0).await;

        let run = |expected: &'static str| {
            let db = state.db.clone();
            let id = id.clone();
            async move {
                sqlx::query(
                    "UPDATE projects
                        SET name         = COALESCE(?1, name),
                            project_data = COALESCE(?2, project_data),
                            updated_at   = datetime('now')
                      WHERE id = ?3
                        AND user_id = ?4
                        AND is_archived = 0
                        AND (?5 IS NULL OR updated_at = ?5)",
                )
                .bind(None::<&str>)
                .bind(Some("{\"x\":1}"))
                .bind(&id)
                .bind("user-1")
                .bind(Some(expected))
                .execute(&db)
                .await
                .expect("update")
                .rows_affected()
            }
        };

        // Stale expected (concurrent write heeft updated_at al veranderd).
        assert_eq!(run("1999-01-01 00:00:00").await, 0);
        // Kloppende expected → de write komt door.
        assert_eq!(run(T0).await, 1);
    }

    /// Niet-bestaand project met expected → 404 (niet 409): de
    /// disambiguatie na rows_affected == 0 mag geen conflict melden voor
    /// een rij die er (voor deze eigenaar) niet is.
    #[tokio::test]
    async fn missing_project_with_expected_returns_404_not_409() {
        let state = test_state().await;
        seed_project(&state, "user-1", T0).await;

        let (status, body) =
            call_update(&state, "user-1", "bestaat-niet", update_body(Some(T0))).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"], "not_found");
    }
}

#[cfg(test)]
mod tests {
    use super::extract_calculation_input;

    #[test]
    fn envelope_project_data_unwraps_to_bare_project() {
        let envelope = r#"{
            "version": "1.0.0",
            "schema": "isso51-project-v1",
            "exported_at": "2026-06-10T00:00:00Z",
            "project": { "info": { "name": "Test" }, "rooms": [] },
            "result": null,
            "modeller": { "rooms": [], "windows": [], "doors": [] }
        }"#;
        let extracted = extract_calculation_input(envelope);
        let value: serde_json::Value = serde_json::from_str(&extracted).unwrap();
        assert_eq!(value["info"]["name"], "Test");
        assert!(value.get("schema").is_none());
    }

    #[test]
    fn legacy_bare_project_data_passes_through_unchanged() {
        let bare = r#"{ "info": { "name": "Legacy" }, "rooms": [] }"#;
        assert_eq!(extract_calculation_input(bare), bare);
    }

    #[test]
    fn foreign_schema_passes_through_unchanged() {
        // Andere schema-waarde → geen envelope van ons; niet uitpakken.
        let other = r#"{ "schema": "iets-anders", "project": {} }"#;
        assert_eq!(extract_calculation_input(other), other);
    }

    #[test]
    fn invalid_json_passes_through_unchanged() {
        let broken = "geen json";
        assert_eq!(extract_calculation_input(broken), broken);
    }
}

