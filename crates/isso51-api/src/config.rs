//! Server configuration loaded from environment variables.

use std::env;

/// API route prefix.
pub const API_PREFIX: &str = "/api/v1";

/// Server configuration.
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub oidc_issuer: Option<String>,
    pub oidc_audience: Option<String>,
    pub cors_origins: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://isso51.db?mode=rwc".to_string()),
            oidc_issuer: env::var("OIDC_ISSUER").ok(),
            oidc_audience: env::var("OIDC_AUDIENCE").ok(),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| {
                    "http://localhost:5173,http://localhost:1420".to_string()
                })
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}
