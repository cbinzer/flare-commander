use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Utc};
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::response::{ApiError, ApiFailure};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespace {
    pub id: String,
    pub title: String,
    pub supports_url_encoding: Option<bool>,
}

#[derive(Debug)]
pub enum KvError {
    NamespaceNotFound,
    KeyNotFound,
    Authentication(AuthenticationError),
    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for KvError {}

impl fmt::Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KvError: {:?}", self)
    }
}

impl From<reqwest::Error> for KvError {
    fn from(error: reqwest::Error) -> Self {
        KvError::Reqwest(error)
    }
}

impl From<cloudflare::framework::Error> for KvError {
    fn from(error: cloudflare::framework::Error) -> Self {
        let cloudflare::framework::Error::ReqwestError(reqwest_error) = error;
        KvError::Reqwest(reqwest_error)
    }
}

impl From<ApiFailure> for KvError {
    fn from(error: ApiFailure) -> Self {
        match error {
            ApiFailure::Error(_, api_errors) => map_api_errors(api_errors.errors),
            ApiFailure::Invalid(reqwest_error) => KvError::Reqwest(reqwest_error),
        }
    }
}

pub fn map_api_errors(errors: Vec<ApiError>) -> KvError {
    if errors.is_empty() {
        return KvError::Unknown("No errors in the response.".to_string());
    }

    let error = &errors[0];
    match error.code {
        10000 => KvError::Authentication(AuthenticationError::InvalidToken),
        10001 => KvError::Authentication(AuthenticationError::InvalidToken),
        10009 => KvError::KeyNotFound,
        10013 => KvError::NamespaceNotFound,
        _ => KvError::Unknown(error.message.clone()),
    }
}

// TODO: Remove
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagePaginationArray<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Vec<ResponseInfo>,
    pub result_info: Option<PaginationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationInfo {
    pub total_count: Option<u32>,
    pub count: Option<u32>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvItems {
    pub items: Vec<KvItem>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvItem {
    pub key: String,
    pub value: String,

    #[serde(with = "ts_milliseconds_option")]
    pub expiration: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvKeys {
    pub keys: Vec<KvKey>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvKey {
    pub name: String,

    #[serde(with = "ts_milliseconds_option")]
    pub expiration: Option<DateTime<Utc>>,
}

impl From<Key> for KvKey {
    fn from(value: Key) -> Self {
        Self {
            name: value.name,
            expiration: value
                .expiration
                .and_then(|dt| DateTime::from_timestamp_millis(dt.timestamp())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetKvItemsInput<'a> {
    pub namespace_id: &'a str,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetKvItemInput<'a> {
    pub namespace_id: &'a str,
    pub key: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct GetKeysInput<'a> {
    pub namespace_id: &'a str,
    pub cursor: Option<String>,
}

pub struct GetKeyValueInput<'a> {
    pub namespace_id: &'a str,
    pub key: &'a str,
}

pub struct SetKvItemInput<'a> {
    pub namespace_id: &'a str,
    pub key: &'a str,
    pub value: &'a str,
    pub expiration: Option<DateTime<Utc>>,
}
