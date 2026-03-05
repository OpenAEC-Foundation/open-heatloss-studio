//! User profile handler with OIDC-based upsert.

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::auth::AuthClaims;
use crate::error::ApiError;
use crate::state::AppState;

/// User profile response.
#[derive(Serialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub name: String,
    pub preferred_username: String,
    pub first_seen_at: String,
    pub last_login_at: String,
}

/// GET /me — Return the current user's profile, creating it if it doesn't exist.
///
/// Uses OIDC claims from the JWT token to upsert the user record.
pub async fn get_profile(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<UserProfile>, ApiError> {
    let sub = &claims.sub;
    let email = claims.email.as_deref().unwrap_or("");
    let name = claims.name.as_deref().unwrap_or("");
    let preferred_username = claims.preferred_username.as_deref().unwrap_or("");
    let issuer = claims.iss.as_deref().unwrap_or("");

    // Upsert: insert if new, update last_login + profile fields if existing.
    sqlx::query(
        "INSERT INTO users (id, email, name, preferred_username, oidc_issuer)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
           email = excluded.email,
           name = excluded.name,
           preferred_username = excluded.preferred_username,
           last_login_at = datetime('now')",
    )
    .bind(sub)
    .bind(email)
    .bind(name)
    .bind(preferred_username)
    .bind(issuer)
    .execute(&state.db)
    .await?;

    // Fetch the full profile back.
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, preferred_username, first_seen_at, last_login_at
         FROM users WHERE id = ?1",
    )
    .bind(sub)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserProfile {
        id: row.id,
        email: row.email,
        name: row.name,
        preferred_username: row.preferred_username,
        first_seen_at: row.first_seen_at,
        last_login_at: row.last_login_at,
    }))
}

/// Internal row type for SQLx query mapping.
#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    name: String,
    preferred_username: String,
    first_seen_at: String,
    last_login_at: String,
}
