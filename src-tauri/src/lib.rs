use crate::app_state::AppState;
use crate::authentication::authentication_commands::verify_credentials;
use crate::kv::kv_commands::{get_kv_items, get_namespaces};
use tauri::Manager;

mod app_state;
mod authentication;
mod cloudflare;
mod common;
mod kv;

#[cfg(test)]
mod test;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            verify_credentials,
            get_namespaces,
            get_kv_items
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
