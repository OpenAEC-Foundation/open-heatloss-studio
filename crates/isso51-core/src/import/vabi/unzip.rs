//! ZIP archive extraction for Vabi `.vp` files.
//!
//! Vabi project files are ZIP archives containing an SQLite database (`Elements.sqlite3`)
//! and configuration files. This module handles extraction of the database to a temporary
//! location for processing.

use crate::error::{Isso51Error, Result};
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Extract the `Elements.sqlite3` database from a Vabi `.vp` file.
///
/// Returns the path to a temporary SQLite file that will be cleaned up
/// when the returned `NamedTempFile` is dropped.
///
/// # Arguments
/// * `vp_path` - Path to the `.vp` file
///
/// # Returns
/// Tuple of (database_path, temp_file_handle). The temp_file_handle must be
/// kept alive for the duration of database access.
///
/// # Errors
/// Returns `VabiZipError` if the ZIP cannot be read or `Elements.sqlite3` is missing.
pub fn extract_elements_database(vp_path: &Path) -> Result<(PathBuf, NamedTempFile)> {
    let file = File::open(vp_path).map_err(|e| {
        Isso51Error::VabiZipError(format!("Cannot open .vp file '{}': {}", vp_path.display(), e))
    })?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        Isso51Error::VabiZipError(format!(
            "Invalid ZIP archive '{}': {}",
            vp_path.display(),
            e
        ))
    })?;

    // Find Elements.sqlite3 in the archive
    let mut elements_file = None;
    for i in 0..archive.len() {
        let file_in_zip = archive.by_index(i).map_err(|e| {
            Isso51Error::VabiZipError(format!("Cannot read ZIP entry {}: {}", i, e))
        })?;

        if file_in_zip.name() == "Elements.sqlite3" {
            elements_file = Some(i);
            break;
        }
    }

    let elements_index = elements_file.ok_or_else(|| {
        Isso51Error::VabiZipError(
            "Elements.sqlite3 not found in .vp archive. Is this a valid Vabi project file?"
                .to_string(),
        )
    })?;

    // Extract to temporary file
    let mut elements_file = archive.by_index(elements_index).map_err(|e| {
        Isso51Error::VabiZipError(format!("Cannot extract Elements.sqlite3: {}", e))
    })?;

    let mut temp_file = NamedTempFile::new().map_err(|e| {
        Isso51Error::VabiZipError(format!("Cannot create temporary file: {}", e))
    })?;

    std::io::copy(&mut elements_file, &mut temp_file).map_err(|e| {
        Isso51Error::VabiZipError(format!("Cannot write Elements.sqlite3 to temporary file: {}", e))
    })?;

    let temp_path = temp_file.path().to_path_buf();

    Ok((temp_path, temp_file))
}