use crate::app_state::AppState;
use crate::authentication::authentication_commands::verify_credentials;
use crate::kv::kv_commands::{
    create_kv_pair, create_namespace, delete_kv_pairs, delete_namespace, get_namespace,
    list_kv_keys, list_namespaces, update_namespace,
};
use kv::kv_commands::{get_kv_pair, write_kv_pair};
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
            list_namespaces,
            get_namespace,
            create_namespace,
            update_namespace,
            delete_namespace,
            get_kv_pair,
            list_kv_keys,
            create_kv_pair,
            write_kv_pair,
            delete_kv_pairs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
