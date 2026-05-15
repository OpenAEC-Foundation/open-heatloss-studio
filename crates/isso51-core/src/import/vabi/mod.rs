//! Vabi Elements `.vp` project file importer.
//!
//! This module provides functionality to import Vabi Elements project files (.vp format)
//! into ISSO 51 Project structures. A `.vp` file is a ZIP archive containing an SQLite
//! database with building, climate, and ventilation data.
//!
//! ## Usage
//!
//! ```rust,no_run
//! # #[cfg(feature = "vabi-import")]
//! use isso51_core::import::vabi::import_vabi_project;
//! use std::path::Path;
//!
//! # #[cfg(feature = "vabi-import")]
//! let project = import_vabi_project(Path::new("project.vp")).unwrap();
//! println!("Imported project: {}", project.info.name);
//! ```
//!
//! ## Supported Data
//!
//! Currently imports:
//! - Basic project information (name, number, description)
//! - Building design conditions (qv10, infiltration method, building type)
//! - Climate data (design outdoor temperature)
//! - Ventilation system configuration
//! - Room metadata (name, design temperature)
//!
//! ## Limitations
//!
//! This is a Phase 1 implementation focused on the happy path:
//! - No construction data (BuildingPart, U-values, materials) — that's Phase 2
//! - Aspect/Template/Variant scenarios use first row or CurrentProjectVersionID
//! - Complex ventilation mappings fall back to MechanicalExhaust with TODO comment

#[cfg(feature = "vabi-import")]
pub mod mapper;

#[cfg(feature = "vabi-import")]
pub mod unzip;

#[cfg(feature = "vabi-import")]
pub use mapper::import_vabi_project;

// Re-export for public API when feature is enabled
#[cfg(feature = "vabi-import")]
pub use unzip::extract_elements_database;