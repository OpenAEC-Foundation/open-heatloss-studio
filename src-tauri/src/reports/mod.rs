//! PDF report generation for warmteverlies reports.
//!
//! Implementation pattern mirrors Open Calc Studio's `src-tauri/src/reports/`.
//! See docs/superpowers/specs/2026-05-09-rust-report-integration-design.md.
pub mod blocks;
pub mod brand;
pub mod fonts;
pub mod generator;
pub mod schema;
pub mod special_pages;

use serde_json::Value;

/// Generate a PDF report from JSON data and write it to disk.
///
/// `report` is the JSON shape produced by `frontend/src/lib/reportBuilder.ts`
/// and `rcReportBuilder.ts`. `output_path` is an absolute filesystem path.
#[tauri::command]
pub fn generate_report_pdf(report: Value, output_path: String) -> Result<(), String> {
    let data: schema::ReportData =
        serde_json::from_value(report).map_err(|e| format!("invalid report data: {e}"))?;
    let bytes = generator::generate_pdf(&data)?;
    std::fs::write(&output_path, &bytes).map_err(|e| format!("write to {output_path}: {e}"))?;
    Ok(())
}

/// Generate a PDF report from JSON data and return the bytes to the frontend.
///
/// The frontend then wraps the bytes in a Blob (PDF mime) and triggers
/// download or in-app preview.
#[tauri::command]
pub fn generate_report_pdf_bytes(report: Value) -> Result<Vec<u8>, String> {
    let data: schema::ReportData =
        serde_json::from_value(report).map_err(|e| format!("invalid report data: {e}"))?;
    generator::generate_pdf(&data)
}
