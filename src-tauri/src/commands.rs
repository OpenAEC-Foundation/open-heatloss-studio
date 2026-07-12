//! Tauri IPC commands wrapping isso51-core.

use isso51_core::model::Project;
use isso51_core::result::ProjectResult;
use nta8800_cooling::{
    calculate_simplified_cooling, SimplifiedAreaInput, SimplifiedCoolingResult,
    SimplifiedLoadInput,
};
use nta8800_tables::climate::de_bilt_climate_data;
use openaec_project_shared::{
    compute_tojuli_full, view, BengResult, ProjectV2, TojuliFullInputs, TojuliResult,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// Run the heat loss calculation.
///
/// Called from the frontend via `invoke("calculate", { project })`.
#[tauri::command]
pub fn calculate(project: Project) -> Result<ProjectResult, String> {
    isso51_core::calculate(&project).map_err(|e| e.to_string())
}

/// Run the heat loss calculation for ProjectV2 with dual-pipeline routing.
///
/// Routes to ISSO 51 or ISSO 53 based on `project.calcs.active_norm()`.
#[tauri::command]
pub fn calculate_v2(project: ProjectV2) -> Result<serde_json::Value, String> {
    use openaec_project_shared::calcs::ActiveNorm;

    match project.calcs.active_norm() {
        ActiveNorm::Isso51 => {
            let isso51_project = view::to_isso51_project(&project)
                .map_err(|e| format!("Failed to convert to ISSO 51 project: {e}"))?;
            let result = isso51_core::calculate(&isso51_project)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(&result)
                .map_err(|e| format!("Failed to serialize ISSO 51 result: {e}"))
        }
        ActiveNorm::Isso53 => {
            let isso53_project = view::to_isso53_project(&project)
                .map_err(|e| format!("Failed to convert to ISSO 53 project: {e}"))?;
            let result = isso53_core::calculate(&isso53_project)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(&result)
                .map_err(|e| format!("Failed to serialize ISSO 53 result: {e}"))
        }
        // BENG is geen warmteverlies-calc; gebruik het `compute_beng`-command.
        ActiveNorm::Beng => {
            Err("BENG wordt niet via calculate_v2 berekend — gebruik compute_beng".to_string())
        }
    }
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

/// Request shape for `simplified_cooling`. Mirrors the API endpoint body.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimplifiedCoolingRequest {
    pub living_area_m2: f64,
    pub other_area_m2: f64,
    pub dwelling_count: u32,
    pub persons_per_dwelling: f64,
    pub infiltration_m3_per_h: f64,
    pub natural_ventilation_m3_per_h: f64,
    pub mechanical_supply_m3_per_h: f64,
    pub peak_hour: u8,
    pub construction_year: u32,
    pub opaque_area_m2: f64,
    pub solar_load_w: f64,
    pub glazing_transmission_w: f64,
}

/// TO-juli (NTA 8800 bijlage AA) vereenvoudigde koelbehoefte-berekening.
///
/// V1 lokale Tauri-aanroep van `nta8800_cooling::calculate_simplified_cooling`.
/// Rekenzone/EFR/Climate/Window-parameters zijn V2-werk — voor nu placeholders.
#[tauri::command]
pub fn simplified_cooling(
    req: SimplifiedCoolingRequest,
) -> Result<SimplifiedCoolingResult, String> {
    let area = SimplifiedAreaInput {
        living_area_m2: req.living_area_m2,
        other_area_m2: req.other_area_m2,
        dwelling_count: req.dwelling_count,
        persons_per_dwelling: req.persons_per_dwelling,
    };
    let load = SimplifiedLoadInput {
        infiltration_m3_per_h: req.infiltration_m3_per_h,
        natural_ventilation_m3_per_h: req.natural_ventilation_m3_per_h,
        mechanical_supply_m3_per_h: req.mechanical_supply_m3_per_h,
        peak_hour: req.peak_hour,
        construction_year: req.construction_year,
        opaque_area_m2: req.opaque_area_m2,
        solar_load_w: req.solar_load_w,
        glazing_transmission_w: req.glazing_transmission_w,
    };
    let climate = de_bilt_climate_data();

    calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load)
        .map_err(|e| e.to_string())
}

/// Request shape for `tojuli_calculate` — volledige NTA 8800 H.10 keten.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TojuliCalculateRequest {
    pub project: ProjectV2,
    pub inputs: TojuliFullInputs,
}

/// TO-juli volledig — NTA 8800 H.10 keten op een ProjectV2.
///
/// Roept `openaec_project_shared::compute_tojuli_full` aan en geeft de
/// maandelijkse Q_C;use + jaarsom + intermediates terug.
#[tauri::command]
pub fn tojuli_calculate(req: TojuliCalculateRequest) -> Result<TojuliResult, String> {
    compute_tojuli_full(&req.project, &req.inputs).map_err(|e| e.to_string())
}

/// Request shape for `compute_beng` — volledige BENG-keten op een ProjectV2.
///
/// Anders dan [`TojuliCalculateRequest`] géén los `inputs`-veld: de
/// installatie-invoer leeft in `project.energy`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BengCalculateRequest {
    pub project: ProjectV2,
}

/// BENG — NTA 8800 energieprestatie op een ProjectV2.
///
/// Roept `openaec_project_shared::compute_beng` aan (fully-qualified om de
/// naamsbotsing met dit command te vermijden) en geeft BENG 1/2/3 + TOjuli +
/// label + service-breakdown terug.
#[tauri::command]
pub fn compute_beng(req: BengCalculateRequest) -> Result<BengResult, String> {
    openaec_project_shared::compute_beng(&req.project).map_err(|e| e.to_string())
}
