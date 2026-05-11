//! ISSO 51 Tauri v2 desktop application.

mod commands;
pub mod reports;

use tauri::{Emitter, Manager};

/// Tauri command: returns the file path passed as argv[1] when the app was
/// launched via a file association (double-click .ifcenergy in Explorer).
/// Returns None when launched without a file or when the arg doesn't look
/// like a supported project path.
#[tauri::command]
fn launched_with_file() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    // argv[0] is the exe path; argv[1] (when present) from a file-association
    // is the absolute path the OS handed us. Skip flags (start with '-' or '/').
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') || arg.starts_with('/') {
            continue;
        }
        let lower = arg.to_lowercase();
        if lower.ends_with(".ifcenergy")
            || lower.ends_with(".isso51.json")
            || lower.ends_with(".json")
        {
            return Some(arg.clone());
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::calculate,
            commands::get_schema,
            commands::import_ifc,
            launched_with_file,
            reports::generate_report_pdf,
            reports::generate_report_pdf_bytes,
        ])
        .setup(|app| {
            // Emit `open-file` event on startup if argv carries a project
            // path (Windows file-association double-click flow). Frontend
            // listens and runs the same `openProjectFile` pipeline as the
            // Bestand → Openen action.
            if let Some(path) = launched_with_file() {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    // Tiny delay so the main window is mounted + the React
                    // event listener is attached before we emit.
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    let _ = handle.emit("open-file", path);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ISSO 51 application");
}
