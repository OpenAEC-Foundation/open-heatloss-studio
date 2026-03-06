//! Report generation proxy handler.
//!
//! Forwards report JSON to the OpenAEC Reports API.
//! Auth: stuurt het Bearer token van de gebruiker door (OIDC).
//! Optioneel: voegt X-API-Key toe als REPORTS_API_KEY is geconfigureerd.

use std::time::Duration;

use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::state::AppState;

/// POST /report/generate — proxy report generation to OpenAEC Reports API.
///
/// Forwards the BM Reports JSON body and the caller's Authorization header
/// to the upstream API. If `REPORTS_API_KEY` is configured, adds X-API-Key
/// as additional auth method.
pub async fn generate_report(
    State(state): State<AppState>,
    headers: HeaderMap,
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

    // Forward Bearer token van de ingelogde gebruiker
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        req = req.header(header::AUTHORIZATION.as_str(), auth.to_str().unwrap_or(""));
    }

    // Optioneel: X-API-Key als server-side fallback
    if let Some(api_key) = state.reports_api_key.as_deref() {
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
