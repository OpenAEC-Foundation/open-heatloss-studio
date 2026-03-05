//! Error types and HTTP error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// JSON error response body.
#[derive(Serialize)]
struct ErrorBody {
    error: String,
    detail: String,
}

/// API-level errors (database, auth, not found, etc.).
#[derive(Debug)]
pub enum ApiError {
    /// Calculation engine error.
    Calculation(isso51_core::error::Isso51Error),
    /// Database error.
    Database(String),
    /// Resource not found.
    NotFound(String),
    /// Forbidden (ownership check failed).
    Forbidden(String),
    /// Internal server error.
    Internal(String),
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {err}");
        ApiError::Database("Database error".to_string())
    }
}

impl From<isso51_core::error::Isso51Error> for ApiError {
    fn from(err: isso51_core::error::Isso51Error) -> Self {
        ApiError::Calculation(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, detail) = match self {
            ApiError::Calculation(err) => {
                return into_calc_response(err);
            }
            ApiError::Database(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            ApiError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
        };

        let body = ErrorBody {
            error: error_type.to_string(),
            detail,
        };

        (status, axum::Json(body)).into_response()
    }
}

/// Map `isso51_core::error::Isso51Error` to an HTTP response.
pub fn into_calc_response(err: isso51_core::error::Isso51Error) -> Response {
    use isso51_core::error::Isso51Error;

    let (status, error_type) = match &err {
        Isso51Error::InvalidInput(_) => (StatusCode::BAD_REQUEST, "invalid_input"),
        Isso51Error::Json(_) => (StatusCode::BAD_REQUEST, "json_error"),
        Isso51Error::MissingParameter(_) => (StatusCode::BAD_REQUEST, "missing_parameter"),
        Isso51Error::RoomNotFound(_) => (StatusCode::NOT_FOUND, "room_not_found"),
        Isso51Error::OutOfRange { .. } => {
            (StatusCode::UNPROCESSABLE_ENTITY, "out_of_range")
        }
    };

    let body = ErrorBody {
        error: error_type.to_string(),
        detail: err.to_string(),
    };

    (status, axum::Json(body)).into_response()
}
