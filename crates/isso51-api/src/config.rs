//! Server configuration constants.

/// Default port for the API server.
pub const PORT: u16 = 3001;

/// API route prefix.
pub const API_PREFIX: &str = "/api/v1";

/// Allowed CORS origins for development.
pub const CORS_ORIGINS: &[&str] = &[
    "http://localhost:5173", // Vite dev server
    "http://localhost:1420", // Tauri dev server
];
