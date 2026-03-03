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

/// Map `isso51_core::error::Isso51Error` to an HTTP response.
pub fn into_response(err: isso51_core::error::Isso51Error) -> Response {
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
