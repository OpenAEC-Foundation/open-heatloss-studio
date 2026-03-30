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
    /// Directory containing static frontend files (SPA). When set, the server
    /// serves these files as a fallback for non-API routes.
    pub static_dir: Option<String>,
    /// Base URL for the OpenAEC Reports API (e.g. `https://reports.openaec.org`).
    pub reports_api_url: Option<String>,
    /// API key for the OpenAEC Reports API.
    pub reports_api_key: Option<String>,
    /// Path to the `ifc-tool` executable for server-side IFC import.
    pub ifc_tool_path: Option<String>,
    /// Path to tenants.json for multi-tenant cloud storage.
    pub tenants_config: Option<String>,
    /// Default tenant slug (used when no tenant claim in token).
    pub default_tenant: Option<String>,
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
            static_dir: env::var("STATIC_DIR").ok().filter(|s| !s.is_empty()),
            reports_api_url: env::var("REPORTS_API_URL")
                .ok()
                .filter(|s| !s.is_empty()),
            reports_api_key: env::var("REPORTS_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
            ifc_tool_path: env::var("IFC_TOOL_PATH")
                .ok()
                .filter(|s| !s.is_empty()),
            tenants_config: env::var("TENANTS_CONFIG")
                .ok()
                .filter(|s| !s.is_empty()),
            default_tenant: env::var("DEFAULT_TENANT")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}
