//! Tauri IPC commands wrapping isso51-core.

use isso51_core::model::Project;
use isso51_core::result::ProjectResult;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

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

/// Import an IFC file via the Python sidecar.
///
/// If `file_path` is provided, imports that file directly.
/// If `file_path` is empty, opens a native file dialog first.
///
/// Spawns `ifc-tool import --input <file_path>` and returns the
/// parsed JSON result directly to the frontend.
#[tauri::command]
pub async fn import_ifc(
    app: AppHandle,
    file_path: String,
) -> Result<serde_json::Value, String> {
    let path = if file_path.is_empty() {
        // Open native file dialog
        use tauri_plugin_dialog::DialogExt;
        let dialog_result = app
            .dialog()
            .file()
            .add_filter("IFC", &["ifc"])
            .blocking_pick_file();
        match dialog_result {
            Some(file) => {
                let path_buf = file
                    .into_path()
                    .map_err(|e| format!("Invalid file path: {e}"))?;
                path_buf.to_string_lossy().to_string()
            }
            None => return Err("Geen bestand geselecteerd".to_string()),
        }
    } else {
        file_path
    };

    let shell = app.shell();

    let output = shell
        .sidecar("ifc-tool")
        .map_err(|e| format!("Failed to create sidecar: {e}"))?
        .args(["import", "--input", &path])
        .output()
        .await
        .map_err(|e| format!("Failed to run ifc-tool: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Try to parse stdout as error JSON
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(msg) = err_json.get("error").and_then(|v| v.as_str()) {
                return Err(msg.to_string());
            }
        }
        return Err(format!("ifc-tool failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Invalid JSON from ifc-tool: {e}"))
}

/// Import a Vabi Elements `.vp` project file.
///
/// If `file_path` is empty, opens a native file dialog filtered on `.vp` first.
/// Calls `isso51_core::import::import_vabi_project` and returns a Project
/// ready to load into the frontend store.
#[tauri::command]
pub fn import_vabi(app: AppHandle, file_path: String) -> Result<Project, String> {
    let path = if file_path.is_empty() {
        use tauri_plugin_dialog::DialogExt;
        let dialog_result = app
            .dialog()
            .file()
            .add_filter("Vabi project", &["vp"])
            .blocking_pick_file();
        match dialog_result {
            Some(file) => file
                .into_path()
                .map_err(|e| format!("Invalid file path: {e}"))?,
            None => return Err("Geen bestand geselecteerd".to_string()),
        }
    } else {
        std::path::PathBuf::from(file_path)
    };

    isso51_core::import::import_vabi_project(&path).map_err(|e| e.to_string())
}
