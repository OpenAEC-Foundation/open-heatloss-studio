//! Vabi Elements .vp import to ProjectV2 format
//!
//! This crate provides functionality to import Vabi Elements project files (.vp format)
//! into ProjectV2 structures for multi-norm analysis. It builds on the V1 importer in
//! isso51-core but adds V2-specific features like building type detection and enriched
//! room data.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use vabi_importer::{import_vabi_project_v2, extract_elements_database};
//! use std::path::Path;
//!
//! let vp_path = Path::new("project.vp");
//! let (db_path, _temp) = extract_elements_database(vp_path).unwrap();
//! let project = import_vabi_project_v2(&db_path).unwrap();
//! println!("Imported V2 project: {}", project.shared.name);
//! ```

pub mod mapper;
pub mod error;

pub use mapper::import_vabi_project_v2;
pub use error::{VabiImporterError, Result};

// Re-export from isso51-core for convenience
pub use isso51_core::import::vabi::extract_elements_database;