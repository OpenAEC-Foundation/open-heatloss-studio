use thiserror::Error;

#[derive(Error, Debug)]
pub enum VabiImporterError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ISSO51 core error: {0}")]
    Isso51(#[from] isso51_core::import::vabi::VabiImportError),

    #[error("ISSO51 calc error: {0}")]
    Isso51Calc(#[from] isso51_core::error::Isso51Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VabiImporterError>;