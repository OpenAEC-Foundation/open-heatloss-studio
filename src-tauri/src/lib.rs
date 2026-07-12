//! ISSO 51 Tauri v2 desktop application.

pub mod commands;
pub mod reports;

use tauri::Emitter;
use tauri_plugin_fs::FsExt;

/// Tauri command: returns the file path passed as argv[1] when the app was
/// launched via a file association (double-click .ifcenergy in Explorer).
/// Returns None when launched without a file or when the arg doesn't look
/// like a supported project path.
///
/// Het resultaat verruimt in `run()` de fs-scope van de webview
/// (`allow_file`), dus de validatie is bewust strikt:
///   - alleen extensies die de open-file-flow (`AppShell` →
///     `openProjectFile`) daadwerkelijk afhandelt: `.ifcenergy` en
///     `.json`-varianten (`.isso51.json`, legacy `.heatloss.json`),
///     case-insensitive;
///   - alleen een bestaand, regulier bestand — geen directory of
///     niet-bestaand pad — mag de scope verruimen.
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
        let supported = lower.ends_with(".ifcenergy") || lower.ends_with(".json");
        if !supported {
            continue;
        }
        match std::fs::metadata(arg) {
            Ok(meta) if meta.is_file() => return Some(arg.clone()),
            _ => continue,
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
            commands::calculate_v2,
            commands::get_schema,
            commands::import_ifc,
            commands::import_vabi,
            commands::simplified_cooling,
            commands::tojuli_calculate,
            commands::compute_beng,
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
                // Het argv-pad kan buiten de statische fs-scope liggen
                // (capabilities/default.json beperkt die tot Documents/
                // Desktop/Downloads — zie src-tauri/README.md). Runtime
                // allowlisten zodat de frontend `readTextFile` op dit pad
                // mag doen — zelfde mechaniek als de dialog-plugin gebruikt
                // voor user-gekozen paden.
                let _ = app.fs_scope().allow_file(&path);
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
