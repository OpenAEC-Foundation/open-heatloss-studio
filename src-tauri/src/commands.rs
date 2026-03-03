//! Tauri IPC commands wrapping isso51-core.

use isso51_core::model::Project;
use isso51_core::result::ProjectResult;

/// Run the heat loss calculation.
///
/// Called from the frontend via `invoke("calculate", { project })`.
#[tauri::command]
pub fn calculate(project: Project) -> Result<ProjectResult, String> {
    isso51_core::calculate(&project).map_err(|e| e.to_string())
}

/// Return a JSON schema by name.
///
/// Supported: "project", "result".
#[tauri::command]
pub fn get_schema(which: String) -> Result<String, String> {
    match which.as_str() {
        "project" => Ok(isso51_core::project_schema()),
        "result" => Ok(isso51_core::result_schema()),
        _ => Err(format!("Unknown schema: {which}")),
    }
}
