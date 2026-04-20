//! Application state shared across handlers.

use std::sync::Arc;

use openaec_cloud::TenantsRegistry;
use sqlx::SqlitePool;

/// Default path to the `ifc-tool` executable inside the Docker container.
const DEFAULT_IFC_TOOL_PATH: &str = "/opt/ifc-tool-venv/bin/ifc-tool";

/// Tool slug used for cloud storage directory mapping.
/// Maps to `calculations/` via `openaec_cloud::container::output_dir_for_tool`.
pub const TOOL_SLUG: &str = "warmteverlies";

/// Shared application state injected into handlers via Axum's `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub http_client: reqwest::Client,
    pub reports_api_url: Option<String>,
    /// Legacy shared-secret key (X-API-Key). Deprecated, vervangen door
    /// `reports_api_service_token`. Blijft ondersteund tot Caddy bypass weg is.
    pub reports_api_key: Option<String>,
    /// Authentik service-token voor backend-to-backend reports calls.
    pub reports_api_service_token: Option<String>,
    /// Path to the `ifc-tool` CLI for server-side IFC import.
    pub ifc_tool_path: String,
    /// Multi-tenant cloud storage registry.
    pub tenants: Arc<TenantsRegistry>,
    /// Default tenant slug (fallback when token has no tenant claim).
    pub default_tenant: Option<String>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: SqlitePool,
        reports_api_url: Option<String>,
        reports_api_key: Option<String>,
        reports_api_service_token: Option<String>,
        ifc_tool_path: Option<String>,
        tenants: TenantsRegistry,
        default_tenant: Option<String>,
    ) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
            reports_api_url,
            reports_api_key,
            reports_api_service_token,
            ifc_tool_path: ifc_tool_path.unwrap_or_else(|| DEFAULT_IFC_TOOL_PATH.to_string()),
            tenants: Arc::new(tenants),
            default_tenant,
        }
    }

    /// Get a [`CloudClient`] for the given tenant slug, or the default tenant.
    ///
    /// Returns `None` if no tenant is configured or the slug is unknown.
    pub fn cloud_client(
        &self,
        tenant_slug: Option<&str>,
    ) -> Option<openaec_cloud::CloudClient> {
        let slug = tenant_slug
            .or(self.default_tenant.as_deref())?;
        let tenant = self.tenants.get(slug)?;
        Some(openaec_cloud::CloudClient::new(tenant, TOOL_SLUG))
    }
}
