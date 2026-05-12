//! Error types for ISSO 51 calculations.

use thiserror::Error;

/// Errors that can occur during heat loss calculations.
#[derive(Debug, Error)]
pub enum Isso51Error {
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

    /// De gekozen [`crate::model::enums::InfiltrationMethod`] vereist een veld
    /// op `Building` dat niet is ingevuld (b.v. `dwelling_class` bij
    /// `VabiCompat`/`Nta8800Strict`). Voorkomt stille fallback met verzonnen
    /// defaults — caller moet of het veld zetten, of expliciet een andere
    /// methode kiezen.
    #[error("infiltration method requires building field: {0}")]
    InfiltrationConfig(String),
}

/// Result type alias for ISSO 51 calculations.
pub type Result<T> = std::result::Result<T, Isso51Error>;
