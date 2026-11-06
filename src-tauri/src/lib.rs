mod commands;
mod cps;
mod project;
mod project_file;
mod project_state;
mod settings;

use project_state::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! This greeting came from Rust.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::restore_project,
            commands::import_cps,
            commands::open_project,
            commands::save_project,
            commands::update_setting
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
