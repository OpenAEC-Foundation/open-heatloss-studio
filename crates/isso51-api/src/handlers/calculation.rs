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

/// POST /calculate_v2 — Run heat loss calculation with dual-pipeline routing.
///
/// Accepts a `ProjectV2` JSON body and routes to ISSO 51 or ISSO 53 based on
/// `project.calcs.active_norm()` (mirrors the Tauri `calculate_v2` command).
///
/// Error mapping:
/// - body deserialisation failure → 400 with the serde message,
/// - view conversion / serialisation failure → 422 with the detail,
/// - calculation error → mapped via `Isso51Error` (400/422/404) for ISSO 51;
///   ISSO 53 calc errors → 422 with the detail,
/// - blocking-task join failure → 500.
pub async fn calculate_v2(body: String) -> impl IntoResponse {
    use openaec_project_shared::calcs::ActiveNorm;
    use openaec_project_shared::{view, ProjectV2};

    // Explicit deserialisation so a malformed body yields 400, not 500.
    let project: ProjectV2 = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "deserialization_error",
                    "detail": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value, (StatusCode, String, String)> {
        match project.calcs.active_norm() {
            ActiveNorm::Isso51 => {
                let isso51_project = view::to_isso51_project(&project).map_err(|e| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "conversion_error".to_string(),
                        format!("Failed to convert to ISSO 51 project: {e}"),
                    )
                })?;
                let result = isso51_core::calculate(&isso51_project).map_err(|e| {
                    let (status, error_type) = match &e {
                        isso51_core::error::Isso51Error::OutOfRange { .. }
                        | isso51_core::error::Isso51Error::InfiltrationConfig(_) => {
                            (StatusCode::UNPROCESSABLE_ENTITY, "calc_error")
                        }
                        isso51_core::error::Isso51Error::RoomNotFound(_) => {
                            (StatusCode::NOT_FOUND, "room_not_found")
                        }
                        _ => (StatusCode::BAD_REQUEST, "calc_error"),
                    };
                    (status, error_type.to_string(), e.to_string())
                })?;
                serde_json::to_value(&result).map_err(|e| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "serialization_error".to_string(),
                        format!("Failed to serialize ISSO 51 result: {e}"),
                    )
                })
            }
            ActiveNorm::Isso53 => {
                let isso53_project = view::to_isso53_project(&project).map_err(|e| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "conversion_error".to_string(),
                        format!("Failed to convert to ISSO 53 project: {e}"),
                    )
                })?;
                let result = isso53_core::calculate(&isso53_project).map_err(|e| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "calc_error".to_string(),
                        e.to_string(),
                    )
                })?;
                serde_json::to_value(&result).map_err(|e| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "serialization_error".to_string(),
                        format!("Failed to serialize ISSO 53 result: {e}"),
                    )
                })
            }
            // BENG is geen warmteverlies-calc; `calculate_v2` route't hem niet.
            // De dedicated `/beng/calculate`-route draait `compute_beng`.
            ActiveNorm::Beng => Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "unsupported_norm".to_string(),
                "BENG wordt niet via /calculate_v2 berekend — gebruik POST /beng/calculate".to_string(),
            )),
        }
    })
    .await;

    match result {
        Ok(Ok(json)) => (StatusCode::OK, Json(json)).into_response(),
        Ok(Err((status, error_type, detail))) => (
            status,
            Json(serde_json::json!({
                "error": error_type,
                "detail": detail
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

/// Available schema definitions.
const AVAILABLE_SCHEMAS: &[(&str, &str)] = &[
    ("project", "Project input schema"),
    ("result", "Calculation result schema"),
    ("ifcx", "IFCX document schema (IFC5 + isso51:: namespace)"),
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
        "ifcx" => Some(isso51_ifcx::ifcx_schema()),
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
