use super::kv_models::KvKeyPairUpsertInput;
use crate::app_state::AppState;
use crate::cloudflare::common::Credentials as CloudflareCredentials;
use crate::cloudflare::kv::{
    KvError as CloudflareKvError, KvKeys, KvKeysListInput, KvNamespace, KvNamespaceCreateInput,
    KvNamespaceGetInput, KvNamespaces, KvNamespacesListInput, KvPair, KvPairGetInput,
    KvPairWriteInput, KvPairsDeleteInput, KvPairsDeleteResult,
};
use crate::cloudflare::kv::{KvNamespaceDeleteInput, KvNamespaceUpdateInput};
use crate::cloudflare::Cloudflare;
use crate::common::common_models::Credentials;
use crate::kv::kv_models::{KvError, KvItem, KvKeyPairCreateInput};
use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn list_namespaces(
    credentials: CloudflareCredentials,
    input: KvNamespacesListInput,
) -> Result<KvNamespaces, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.list_namespaces(input).await?)
}

#[tauri::command]
pub async fn get_namespace(
    credentials: CloudflareCredentials,
    input: KvNamespaceGetInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.get_namespace(input).await?)
}

#[tauri::command]
pub async fn create_namespace(
    credentials: CloudflareCredentials,
    input: KvNamespaceCreateInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.create_namespace(input).await?)
}

#[tauri::command]
pub async fn update_namespace(
    credentials: CloudflareCredentials,
    input: KvNamespaceUpdateInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.update_namespace(input).await?)
}

#[tauri::command]
pub async fn delete_namespace(
    credentials: CloudflareCredentials,
    input: KvNamespaceDeleteInput,
) -> Result<(), KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.delete_namespace(input).await?)
}

#[tauri::command]
pub async fn get_kv_pair(
    credentials: CloudflareCredentials,
    input: KvPairGetInput,
) -> Result<KvPair, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.get_kv_pair(input).await?)
}

#[tauri::command]
pub async fn write_kv_pair(
    credentials: CloudflareCredentials,
    input: KvPairWriteInput,
) -> Result<KvPair, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.write_kv_pair(input).await?)
}

#[tauri::command]
pub async fn create_kv_item(
    credentials: Credentials,
    input: KvKeyPairCreateInput,
    state: State<'_, AppState>,
) -> Result<KvItem, KvCommandError> {
    Ok(state
        .kv_service
        .create_kv_item(&credentials, &input)
        .await?)
}

#[tauri::command]
pub async fn delete_kv_pairs(
    credentials: CloudflareCredentials,
    input: KvPairsDeleteInput,
) -> Result<KvPairsDeleteResult, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.delete_kv_pairs(input).await?)
}

#[tauri::command]
pub async fn list_kv_keys(
    credentials: CloudflareCredentials,
    input: KvKeysListInput,
) -> Result<KvKeys, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.list_keys(input).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvCommandError {
    kind: KvCommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KvCommandErrorKind {
    NamespaceAlreadyExists,
    NamespaceNotFound,
    NamespaceTitleMissing,

    KeyNotFound,
    KeyAlreadyExists,

    Authentication,
    Unknown,
}

impl From<KvError> for KvCommandError {
    fn from(error: KvError) -> Self {
        match error {
            KvError::NamespaceAlreadyExists(message) => KvCommandError {
                kind: KvCommandErrorKind::NamespaceAlreadyExists,
                message,
            },
            KvError::NamespaceTitleMissing(message) => KvCommandError {
                kind: KvCommandErrorKind::NamespaceTitleMissing,
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
                    message: "A network error occurred".to_string(),
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

impl From<CloudflareKvError> for KvCommandError {
    fn from(error: CloudflareKvError) -> Self {
        match error {
            CloudflareKvError::NamespaceAlreadyExists(message) => KvCommandError {
                kind: KvCommandErrorKind::NamespaceAlreadyExists,
                message,
            },
            CloudflareKvError::NamespaceTitleMissing(message) => KvCommandError {
                kind: KvCommandErrorKind::NamespaceTitleMissing,
                message,
            },
            CloudflareKvError::NamespaceNotFound => KvCommandError {
                kind: KvCommandErrorKind::NamespaceNotFound,
                message: "Namespace not found".to_string(),
            },
            CloudflareKvError::KeyNotFound => KvCommandError {
                kind: KvCommandErrorKind::KeyNotFound,
                message: "Key not found".to_string(),
            },
            CloudflareKvError::KeyAlreadyExists(key) => KvCommandError {
                kind: KvCommandErrorKind::KeyAlreadyExists,
                message: format!("An item with the key {} already exists", key),
            },
            CloudflareKvError::Token(token_err) => {
                error!(
                    "A token error occurred on interacting with kv: {}",
                    token_err
                );
                KvCommandError {
                    kind: KvCommandErrorKind::Authentication,
                    message: "Authentication error".to_string(),
                }
            }
            CloudflareKvError::Reqwest(reqwest_err) => {
                error!(
                    "A reqwest error occurred on interacting with kv: {}",
                    reqwest_err
                );
                KvCommandError {
                    kind: KvCommandErrorKind::Unknown,
                    message: "A network error occurred".to_string(),
                }
            }
            CloudflareKvError::Unknown(unknown_err) => {
                error!("An unknown kv error occurred: {}", unknown_err);
                KvCommandError {
                    kind: KvCommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}
