//! Application state shared across handlers.

use sqlx::SqlitePool;

use crate::auth::JwksCache;

/// Shared application state injected into handlers via Axum's `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwks: Option<JwksCache>,
}

impl AppState {
    pub fn new(db: SqlitePool, jwks: Option<JwksCache>) -> Self {
        Self { db, jwks }
    }
}

impl AsRef<Option<JwksCache>> for AppState {
    fn as_ref(&self) -> &Option<JwksCache> {
        &self.jwks
    }
}
