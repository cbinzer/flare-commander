use crate::app_state::AppState;
use crate::common::common_models::Credentials;
use crate::kv::kv_models::KvError;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct KvCommandError {
    kind: KvCommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KvCommandErrorKind {
    Authentication,
    Unknown,
}

impl From<KvError> for KvCommandError {
    fn from(error: KvError) -> Self {
        match error {
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
