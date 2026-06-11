//! ISSO 74 thermal comfort / overheating assessment handler.
//!
//! Toets-laag: accepts an [`isso74_core::model::Isso74Request`] (CSV content +
//! config), runs the assessment on a blocking thread, and returns the
//! per-room verdict + plot-data.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use isso74_core::model::Isso74Request;
use isso74_core::result::Isso74Result;

/// POST /isso74/calculate — RMOT + ATG + TO-uren + GTO assessment.
pub async fn isso74_calculate(Json(req): Json<Isso74Request>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || isso74_core::assess_request(&req)).await;

    match result {
        Ok(Ok(r)) => Json::<Isso74Result>(r).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "isso74_assess_error",
                "detail": e.to_string()
            })),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "detail": join_err.to_string()
            })),
        )
            .into_response(),
    }
}
