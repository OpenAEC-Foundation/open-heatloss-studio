//! Error types for ISSO 53 calculations.

use thiserror::Error;

/// Errors that can occur during heat loss calculations.
#[derive(Debug, Error)]
pub enum Isso53Error {
    /// Invalid input data.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A referenced room was not found.
    #[error("room not found: {0}")]
    RoomNotFound(String),

    /// A required parameter is missing.
    #[error("missing parameter: {0}")]
    MissingParameter(String),

    /// A calculated value is out of the expected range.
    #[error("value out of range: {field} = {value} (expected {expected})")]
    OutOfRange {
        field: String,
        value: f64,
        expected: String,
    },

    /// Room height exceeds ISSO 53 limit (4m). Use ISSO 57 for taller buildings.
    #[error("room height {height}m exceeds ISSO 53 limit of 4.0m. Use ISSO 57 for buildings with ceiling height > 4m")]
    HeightExceedsLimit { height: f64 },

    /// Infiltration method configuration error.
    #[error("infiltration method requires building field: {0}")]
    InfiltrationConfig(String),

    /// Feature not supported in this ISSO 53 implementation.
    #[error("not supported: {0}")]
    NotSupported(String),

    /// Heating-up (opwarmtoeslag) parameters fall outside the ISSO 53
    /// table domain (§4.8, tabel 4.13/4.14), so φ_hu,i is undefined.
    #[error("invalid heating-up parameters: {0}")]
    InvalidHeatingUpParameters(String),
}

/// Result type alias for ISSO 53 calculations.
pub type Result<T> = std::result::Result<T, Isso53Error>;