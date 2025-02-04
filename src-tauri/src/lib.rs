use crate::app_state::AppState;
use crate::authentication::authentication_commands::login;
use crate::kv::kv_commands::get_namespaces;
use tauri::Manager;

mod app_state;
mod authentication;
mod kv;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login, get_namespaces])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
