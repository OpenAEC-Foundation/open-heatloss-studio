//! Calculation and schema handlers (public, no auth required).

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::error;

/// Health check response.
#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// GET /health — Returns server status and version.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// POST /calculate — Run heat loss calculation.
///
/// Accepts a Project JSON body, runs the calculation on a blocking thread
/// (isso51-core is sync CPU work), and returns the ProjectResult.
pub async fn calculate(body: String) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        isso51_core::calculate_from_json(&body)
    })
    .await;

    match result {
        Ok(Ok(json)) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            json,
        )
            .into_response(),
        Ok(Err(calc_err)) => error::into_calc_response(calc_err),
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

/// Available schema definitions.
const AVAILABLE_SCHEMAS: &[(&str, &str)] = &[
    ("project", "Project input schema"),
    ("result", "Calculation result schema"),
];

/// GET /schemas — List available schemas.
pub async fn list_schemas() -> Json<serde_json::Value> {
    let schemas: Vec<serde_json::Value> = AVAILABLE_SCHEMAS
        .iter()
        .map(|(name, description)| {
            serde_json::json!({
                "name": name,
                "description": description,
                "url": format!("/api/v1/schemas/{name}"),
            })
        })
        .collect();

    Json(serde_json::json!({ "schemas": schemas }))
}

/// GET /schemas/:name — Return a JSON schema.
///
/// Supported names: "project", "result".
pub async fn get_schema(Path(name): Path<String>) -> impl IntoResponse {
    let schema = match name.as_str() {
        "project" => Some(isso51_core::project_schema()),
        "result" => Some(isso51_core::result_schema()),
        _ => None,
    };

    match schema {
        Some(json) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            json,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "not_found",
                "detail": format!("Unknown schema: {name}")
            })),
        )
            .into_response(),
    }
}
