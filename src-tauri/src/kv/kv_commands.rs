use crate::app_state::AppState;
use crate::common::common_models::Credentials;
use crate::kv::kv_models::{GetKvItemsInput, KvError, KvItems};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn get_namespaces(
    credentials: Credentials,
    state: State<'_, AppState>,
) -> Result<Vec<WorkersKvNamespace>, KvCommandError> {
    Ok(state.kv_service.get_namespaces(&credentials).await?)
}

#[tauri::command]
pub async fn get_kv_items<'a>(
    credentials: Credentials,
    input: GetKvItemsInput<'a>,
    state: State<'_, AppState>,
) -> Result<KvItems, KvCommandError> {
    Ok(state.kv_service.get_kv_items(&credentials, input).await?)
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
    Authentication,
    Unknown,
}

impl From<KvError> for KvCommandError {
    fn from(error: KvError) -> Self {
        match error {
            KvError::NamespaceNotFound => KvCommandError {
                kind: KvCommandErrorKind::NamespaceNotFound,
                message: "Namespace not found".to_string(),
            },
            KvError::KeyNotFound => KvCommandError {
                kind: KvCommandErrorKind::KeyNotFound,
                message: "Key not found".to_string(),
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
