//! Import module for converting external thermal export data into ISSO 51 Project models.
//!
//! Supports importing from Revit thermal exports (via PyRevit ThermalExport).

pub mod sfb;
pub mod thermal;

#[cfg(feature = "vabi-import")]
pub mod vabi;

pub use thermal::{map_thermal_import, ThermalImport, ThermalImportResult};

#[cfg(feature = "vabi-import")]
pub use vabi::import_vabi_project;
