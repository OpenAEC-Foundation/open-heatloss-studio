//! Thermal import handler.
//!
//! Accepts a Revit thermal export JSON, maps it to an ISSO 51 Project,
//! and returns the result with warnings and construction layer info.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use isso51_core::import::{map_thermal_import, ThermalImport};

/// `POST /import/thermal` — Import a Revit thermal export.
///
/// Accepts a `ThermalImport` JSON body, maps it to an ISSO 51 Project model,
/// and returns a `ThermalImportResult` with the mapped project, warnings,
/// construction layers (for Rc-calculator review), and room polygons (for 3D viewer).
pub async fn thermal_import_handler(body: String) -> impl IntoResponse {
    // Parse on a blocking thread (may be CPU-bound for large models).
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let import: ThermalImport =
            serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {e}"))?;
        if !matches!(import.version.as_str(), "1.0" | "1.1") {
            return Err(format!(
                "Unsupported version: {}. Expected 1.0 or 1.1",
                import.version
            ));
        }
        let mapped = map_thermal_import(import);
        serde_json::to_string(&mapped).map_err(|e| format!("Serialization error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(json)) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            json,
        )
            .into_response(),
        Ok(Err(err)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "import_error",
                "detail": err
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
