//! Report generation proxy handler.
//!
//! Forwards report JSON to the OpenAEC Reports API via Authentik service-token
//! (``svc-warmteverlies``) met ``X-Original-Tenant`` header voor on-behalf-of
//! user-tenant context.
//!
//! Auth vereisten:
//! - Caller moet via forward_auth (AuthClaims) authenticated zijn
//! - Upstream-call gebruikt service-token Bearer als geconfigureerd
//! - Fallback: legacy X-API-Key als service-token niet beschikbaar is

use std::time::Duration;

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::auth::AuthClaims;
use crate::error::ApiError;
use crate::state::AppState;

/// POST /report/generate — proxy report generation to OpenAEC Reports API.
///
/// Auth chain:
/// 1. `AuthClaims` extractor (forward_auth) valideert de caller en levert user-tenant.
/// 2. Upstream call gebruikt `REPORTS_API_SERVICE_TOKEN` (Authentik ak-*) als
///    primary auth methode. De user-tenant wordt doorgegeven als
///    `X-Original-Tenant` header zodat reports de juiste tenant-templates kiest.
/// 3. Fallback voor transitie: als service-token niet geconfigureerd is maar
///    `REPORTS_API_KEY` wel, stuur die als `X-API-Key` (legacy Caddy bypass).
pub async fn generate_report(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    body: String,
) -> Result<Response, ApiError> {
    let base_url = state.reports_api_url.as_deref().ok_or_else(|| {
        ApiError::ServiceUnavailable(
            "Rapportgeneratie is niet geconfigureerd (REPORTS_API_URL ontbreekt)".to_string(),
        )
    })?;

    let url = format!("{}/api/generate/v2", base_url.trim_end_matches('/'));

    let mut req = state
        .http_client
        .post(&url)
        .header(header::CONTENT_TYPE.as_str(), "application/json")
        .timeout(Duration::from_secs(30));

    // Primair: Authentik service-token (Bearer ak-*) + X-Original-Tenant
    if let Some(token) = state.reports_api_service_token.as_deref() {
        req = req.header(
            header::AUTHORIZATION.as_str(),
            format!("Bearer {token}"),
        );
        if let Some(tenant) = claims.tenant.as_deref() {
            req = req.header("X-Original-Tenant", tenant);
        }
    } else if let Some(api_key) = state.reports_api_key.as_deref() {
        // Legacy fallback: X-API-Key (Caddy bypass) — wordt verwijderd
        // zodra service-token overal werkt.
        req = req.header("X-API-Key", api_key);
    }

    let upstream = req.body(body).send().await.map_err(|e| {
        tracing::error!("Reports API request failed: {e}");
        ApiError::ReportService(format!("Rapport service niet bereikbaar: {e}"))
    })?;

    if !upstream.status().is_success() {
        let status = upstream.status();
        let detail = upstream.text().await.unwrap_or_default();
        tracing::error!("Reports API returned {status}: {detail}");
        return Err(ApiError::ReportService(format!(
            "Rapport generatie mislukt ({status}): {detail}"
        )));
    }

    let pdf_bytes = upstream.bytes().await.map_err(|e| {
        tracing::error!("Failed to read report PDF: {e}");
        ApiError::ReportService("Fout bij ophalen van rapport PDF".to_string())
    })?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"rapport.pdf\"",
            ),
        ],
        pdf_bytes,
    )
        .into_response())
}
