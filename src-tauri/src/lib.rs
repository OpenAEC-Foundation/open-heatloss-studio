//! ISSO 51 Tauri v2 desktop application.

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::calculate,
            commands::get_schema,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISSO 51 application");
}
