use crate::app_state::AppState;
use crate::common::common_models::Credentials;
use crate::kv::kv_models::{
    CreateKvItemInput, GetKeysInput, KvError, KvItem, KvItemsDeletionInput, KvItemsDeletionResult,
    KvKeys, KvNamespace,
};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

use super::kv_models::{GetKvItemInput, WriteKvItemInput};

#[tauri::command]
pub async fn get_namespaces(
    credentials: Credentials,
    state: State<'_, AppState>,
) -> Result<Vec<WorkersKvNamespace>, KvCommandError> {
    Ok(state.kv_service.get_namespaces(&credentials).await?)
}

#[tauri::command]
pub async fn create_namespace(
    credentials: Credentials,
    title: String,
    state: State<'_, AppState>,
) -> Result<KvNamespace, KvCommandError> {
    Ok(state
        .kv_service
        .create_namespace(&credentials, title)
        .await?)
}

#[tauri::command]
pub async fn get_kv_item<'a>(
    credentials: Credentials,
    input: GetKvItemInput<'a>,
    state: State<'_, AppState>,
) -> Result<KvItem, KvCommandError> {
    Ok(state.kv_service.get_kv_item(&credentials, input).await?)
}

#[tauri::command]
pub async fn write_kv_item(
    credentials: Credentials,
    input: WriteKvItemInput,
    state: State<'_, AppState>,
) -> Result<KvItem, KvCommandError> {
    Ok(state.kv_service.write_kv_item(&credentials, input).await?)
}

#[tauri::command]
pub async fn create_kv_item(
    credentials: Credentials,
    input: CreateKvItemInput,
    state: State<'_, AppState>,
) -> Result<KvItem, KvCommandError> {
    Ok(state
        .kv_service
        .create_kv_item(&credentials, &input)
        .await?)
}

#[tauri::command]
pub async fn delete_kv_items(
    credentials: Credentials,
    input: KvItemsDeletionInput,
    state: State<'_, AppState>,
) -> Result<KvItemsDeletionResult, KvCommandError> {
    Ok(state.kv_service.delete_items(&credentials, &input).await?)
}

#[tauri::command]
pub async fn get_kv_keys<'a>(
    credentials: Credentials,
    input: GetKeysInput<'a>,
    state: State<'_, AppState>,
) -> Result<KvKeys, KvCommandError> {
    Ok(state.kv_service.get_keys(&credentials, input).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvCommandError {
    kind: KvCommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KvCommandErrorKind {
    NamespaceNotFound,
    KeyNotFound,
    KeyAlreadyExists,
    Authentication,
    Unknown,
}

impl From<KvError> for KvCommandError {
    fn from(error: KvError) -> Self {
        match error {
            KvError::NamespaceAlreadyExists(message) => KvCommandError {
                kind: KvCommandErrorKind::NamespaceNotFound,
                message,
            },
            KvError::NamespaceNotFound => KvCommandError {
                kind: KvCommandErrorKind::NamespaceNotFound,
                message: "Namespace not found".to_string(),
            },
            KvError::KeyNotFound => KvCommandError {
                kind: KvCommandErrorKind::KeyNotFound,
                message: "Key not found".to_string(),
            },
            KvError::KeyAlreadyExists(key) => KvCommandError {
                kind: KvCommandErrorKind::KeyAlreadyExists,
                message: format!("An item with the key {} already exists", key),
            },
            KvError::Authentication(auth_err) => {
                error!(
                    "An authentication error occurred on interacting with kv: {}",
                    auth_err
                );
                KvCommandError {
                    kind: KvCommandErrorKind::Authentication,
                    message: "Authentication error".to_string(),
                }
            }
            KvError::Reqwest(reqwest_err) => {
                error!(
                    "A reqwest error occurred on interacting with kv: {}",
                    reqwest_err
                );
                KvCommandError {
                    kind: KvCommandErrorKind::Unknown,
                    message: "An network error occurred".to_string(),
                }
            }
            KvError::Unknown(unknown_err) => {
                error!("An unknown kv error occurred: {}", unknown_err);
                KvCommandError {
                    kind: KvCommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}
