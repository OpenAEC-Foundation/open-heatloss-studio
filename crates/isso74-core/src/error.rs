//! Error types for ISSO 74 thermal comfort assessments.

use thiserror::Error;

/// Errors that can occur during an ISSO 74 thermal comfort assessment.
#[derive(Debug, Error)]
pub enum Isso74Error {
    /// Invalid input data.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// CSV parsing failed.
    #[error("CSV parse error: {0}")]
    CsvParse(String),

    /// A required parameter or column is missing.
    #[error("missing parameter: {0}")]
    MissingParameter(String),

    /// A referenced room/column was not found.
    #[error("room column not found: {0}")]
    RoomNotFound(String),

    /// A calculated/parsed value is out of the expected range.
    #[error("value out of range: {field} = {value} (expected {expected})")]
    OutOfRange {
        field: String,
        value: f64,
        expected: String,
    },
}

/// Result type alias for ISSO 74 assessments.
pub type Result<T> = std::result::Result<T, Isso74Error>;
