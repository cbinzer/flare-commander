use crate::authentication::authentication_commands::verify_account_and_credentials;
use crate::kv::kv_commands::{
    create_kv_pair, create_namespace, delete_kv_pairs, delete_namespace, get_kv_pair, get_kv_pairs,
    get_namespace, list_kv_keys, list_namespaces, update_namespace, write_kv_pair, write_kv_pairs,
};

mod authentication;
mod cloudflare;
mod kv;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            verify_account_and_credentials,
            list_namespaces,
            get_namespace,
            create_namespace,
            update_namespace,
            delete_namespace,
            get_kv_pair,
            get_kv_pairs,
            list_kv_keys,
            create_kv_pair,
            write_kv_pair,
            write_kv_pairs,
            delete_kv_pairs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
