//! Server-side IFC import handler.
//!
//! Accepts a multipart file upload, writes it to a temporary directory,
//! runs the `ifc-tool import` CLI, and returns the JSON result.

use std::time::Duration;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::auth::AuthClaims;
use crate::state::AppState;

/// Maximum upload size: 100 MB.
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Subprocess timeout.
const IFC_TOOL_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    detail: String,
}

fn error_response(status: StatusCode, error: &str, detail: String) -> Response {
    let body = ErrorBody {
        error: error.to_string(),
        detail,
    };
    (status, Json(body)).into_response()
}

/// `POST /ifc/import` — Upload an IFC file and run server-side import.
///
/// Requires an authenticated caller (OIDC Bearer token) — the 100 MB body
/// limit + subprocess exec is too much attack surface to expose publicly.
pub async fn import_ifc(
    State(state): State<AppState>,
    AuthClaims(_claims): AuthClaims,
    mut multipart: Multipart,
) -> Response {
    // Extract the "file" field from multipart form data.
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let filename = field
            .file_name()
            .unwrap_or("upload.ifc")
            .to_string();

        match field.bytes().await {
            Ok(bytes) => {
                if bytes.len() > MAX_FILE_SIZE {
                    return error_response(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "file_too_large",
                        format!(
                            "Bestand is te groot ({:.1} MB, max {} MB)",
                            bytes.len() as f64 / 1_048_576.0,
                            MAX_FILE_SIZE / 1_048_576
                        ),
                    );
                }
                file_data = Some((filename, bytes.to_vec()));
            }
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "upload_error",
                    format!("Bestand lezen mislukt: {e}"),
                );
            }
        }
        break;
    }

    let Some((filename, data)) = file_data else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "missing_file",
            "Geen 'file' veld in multipart upload".to_string(),
        );
    };

    // Write to a temporary directory (auto-cleaned on drop).
    let tmp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("tempdir creation failed: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Kan tijdelijke map niet aanmaken".to_string(),
            );
        }
    };

    let ifc_path = tmp_dir.path().join(&filename);
    let ifc_path_str = ifc_path.to_string_lossy().to_string();

    if let Err(e) = write_file(&ifc_path, &data).await {
        tracing::error!("Failed to write temp IFC file: {e}");
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Kan tijdelijk bestand niet schrijven".to_string(),
        );
    }

    tracing::info!(
        "IFC import: {} ({:.1} MB) → {}",
        filename,
        data.len() as f64 / 1_048_576.0,
        ifc_path_str
    );

    // Run ifc-tool subprocess.
    let result = tokio::time::timeout(
        IFC_TOOL_TIMEOUT,
        run_ifc_tool(&state.ifc_tool_path, &ifc_path_str),
    )
    .await;

    match result {
        Ok(Ok(json_value)) => Json(json_value).into_response(),
        Ok(Err(msg)) => {
            tracing::error!("ifc-tool failed: {msg}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ifc_tool_error",
                msg,
            )
        }
        Err(_) => {
            tracing::error!("ifc-tool timed out after {IFC_TOOL_TIMEOUT:?}");
            error_response(
                StatusCode::GATEWAY_TIMEOUT,
                "timeout",
                format!(
                    "IFC import duurde te lang (max {} seconden)",
                    IFC_TOOL_TIMEOUT.as_secs()
                ),
            )
        }
    }
}

/// Write bytes to a file asynchronously.
async fn write_file(
    path: &std::path::Path,
    data: &[u8],
) -> Result<(), std::io::Error> {
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(data).await?;
    file.flush().await?;
    Ok(())
}

/// Run `ifc-tool import --input <path>` and parse stdout as JSON.
async fn run_ifc_tool(
    tool_path: &str,
    ifc_path: &str,
) -> Result<serde_json::Value, String> {
    let output = tokio::process::Command::new(tool_path)
        .args(["import", "--input", ifc_path, "--no-close-gaps"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("ifc-tool starten mislukt: {e}"))?
        .wait_with_output()
        .await
        .map_err(|e| format!("ifc-tool uitvoeren mislukt: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(format!(
            "ifc-tool exit code {code}: {}",
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("ifc-tool output is geen geldige UTF-8: {e}"))?;

    serde_json::from_str(&stdout)
        .map_err(|e| format!("ifc-tool output is geen geldige JSON: {e}"))
}
