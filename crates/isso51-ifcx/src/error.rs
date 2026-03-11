//! Error types for IFCX operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IfcxError {
    #[error("Missing required IFC entry: {0}")]
    MissingEntry(&'static str),

    #[error("Missing required attribute '{1}' on {0}")]
    MissingAttribute(&'static str, &'static str),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Calculation error: {0}")]
    Calc(#[from] isso51_core::error::Isso51Error),
}

pub type Result<T> = std::result::Result<T, IfcxError>;
