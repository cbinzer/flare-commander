use crate::app_state::AppState;
use crate::common::common_models::Credentials;
use crate::kv::kv_models::KvError;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn get_namespaces(
    account_id: &str,
    token: &str,
    state: State<'_, AppState>,
) -> Result<Vec<WorkersKvNamespace>, KvCommandError> {
    let credentials = Credentials::UserAuthToken {
        account_id: account_id.to_string(),
        token: token.to_string(),
    };
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
            KvError::Authentication(_) => KvCommandError {
                kind: KvCommandErrorKind::Authentication,
                message: "Authentication error".to_string(),
            },
            KvError::Reqwest(_) => KvCommandError {
                kind: KvCommandErrorKind::Unknown,
                message: "An network error occurred".to_string(),
            },
            KvError::Unknown(_) => KvCommandError {
                kind: KvCommandErrorKind::Unknown,
                message: "An unknown error occurred".to_string(),
            },
        }
    }
}
