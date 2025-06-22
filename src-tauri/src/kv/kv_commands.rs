use crate::cloudflare::common::Credentials;
use crate::cloudflare::kv::{
    KvError, KvKeys, KvKeysListInput, KvNamespace, KvNamespaceCreateInput, KvNamespaceGetInput,
    KvNamespaces, KvNamespacesListInput, KvPair, KvPairCreateInput, KvPairGetInput,
    KvPairWriteInput, KvPairsDeleteInput, KvPairsDeleteResult,
};
use crate::cloudflare::kv::{KvNamespaceDeleteInput, KvNamespaceUpdateInput};
use crate::cloudflare::Cloudflare;

use log::error;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn list_namespaces(
    credentials: Credentials,
    input: KvNamespacesListInput,
) -> Result<KvNamespaces, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.list_namespaces(input).await?)
}

#[tauri::command]
pub async fn get_namespace(
    credentials: Credentials,
    input: KvNamespaceGetInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.get_namespace(input).await?)
}

#[tauri::command]
pub async fn create_namespace(
    credentials: Credentials,
    input: KvNamespaceCreateInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.create_namespace(input).await?)
}

#[tauri::command]
pub async fn update_namespace(
    credentials: Credentials,
    input: KvNamespaceUpdateInput,
) -> Result<KvNamespace, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.update_namespace(input).await?)
}

#[tauri::command]
pub async fn delete_namespace(
    credentials: Credentials,
    input: KvNamespaceDeleteInput,
) -> Result<(), KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.delete_namespace(input).await?)
}

#[tauri::command]
pub async fn get_kv_pair(
    credentials: Credentials,
    input: KvPairGetInput,
) -> Result<KvPair, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.get_kv_pair(input).await?)
}

#[tauri::command]
pub async fn write_kv_pair(
    credentials: Credentials,
    input: KvPairWriteInput,
) -> Result<KvPair, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.write_kv_pair(input).await?)
}

#[tauri::command]
pub async fn create_kv_pair(
    credentials: Credentials,
    input: KvPairCreateInput,
) -> Result<KvPair, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.create_kv_pair(input).await?)
}

#[tauri::command]
pub async fn delete_kv_pairs(
    credentials: Credentials,
    input: KvPairsDeleteInput,
) -> Result<KvPairsDeleteResult, KvCommandError> {
    let cloudflare_client = Cloudflare::new(credentials, None);
    let kv = cloudflare_client.kv;
    Ok(kv.delete_kv_pairs(input).await?)
}

#[tauri::command]
pub async fn list_kv_keys(
    credentials: Credentials,
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
    InvalidMetadata,
    InvalidExpiration,

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
            KvError::InvalidMetadata => KvCommandError {
                kind: KvCommandErrorKind::InvalidMetadata,
                message: "Metadata must be valid json".to_string(),
            },
            KvError::InvalidExpiration => KvCommandError {
                kind: KvCommandErrorKind::InvalidExpiration,
                message: "Invalid expiration date. Please specify integer greater than the current number of seconds since the UNIX epoch.".to_string(),
            },
            KvError::Token(token_err) => {
                error!(
                    "A token error occurred on interacting with kv: {}",
                    token_err
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
