use crate::cloudflare::common::{
    ApiCursorPaginatedResponse, ApiPaginatedResponse, ApiResponse, OrderDirection, PageInfo,
    TokenError,
};

use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct KvNamespacesListInput {
    pub account_id: String,
    pub order_by: Option<KvNamespacesOrderBy>,
    pub order_direction: Option<OrderDirection>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl From<KvNamespacesListInput> for HashMap<String, String> {
    fn from(value: KvNamespacesListInput) -> Self {
        let mut map = HashMap::new();
        if let Some(order_by) = value.order_by {
            map.insert("order".to_string(), order_by.to_string());
        }

        if let Some(order_direction) = value.order_direction {
            map.insert("direction".to_string(), order_direction.to_string());
        }

        if let Some(page) = value.page {
            map.insert("page".to_string(), page.to_string());
        }

        if let Some(per_page) = value.per_page {
            map.insert("per_page".to_string(), per_page.to_string());
        }

        map
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum KvNamespacesOrderBy {
    #[serde(rename = "id")]
    Id,

    #[serde(rename = "title")]
    Title,
}

impl Display for KvNamespacesOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            KvNamespacesOrderBy::Id => "id".to_string(),
            KvNamespacesOrderBy::Title => "title".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespaces {
    pub items: Vec<KvNamespace>,
    pub page_info: PageInfo,
}

impl From<ApiPaginatedResponse<Vec<KvNamespace>>> for KvNamespaces {
    fn from(value: ApiPaginatedResponse<Vec<KvNamespace>>) -> Self {
        Self {
            items: value.result,
            page_info: value.result_info,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespace {
    pub id: String,
    pub title: String,
    pub beta: Option<bool>,
    pub supports_url_encoding: Option<bool>,
}

impl From<ApiResponse<KvNamespace>> for KvNamespace {
    fn from(value: ApiResponse<KvNamespace>) -> Self {
        value.result
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespaceGetInput {
    pub account_id: String,
    pub namespace_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespaceCreateInput {
    pub account_id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespaceUpdateInput {
    pub account_id: String,
    pub namespace_id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvNamespaceDeleteInput {
    pub account_id: String,
    pub namespace_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvKeys {
    pub keys: Vec<KvKey>,
    pub count: usize,
    pub cursor: Option<String>,
}

impl From<ApiCursorPaginatedResponse<Vec<KvKey>>> for KvKeys {
    fn from(value: ApiCursorPaginatedResponse<Vec<KvKey>>) -> Self {
        Self {
            keys: value.result,
            count: value.result_info.count,
            cursor: value.result_info.cursor,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvKey {
    pub name: String,

    #[serde(default)]
    pub metadata: Option<Value>,

    #[serde(default)]
    #[serde(with = "ts_seconds_option")]
    pub expiration: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KvKeysListInput {
    pub account_id: String,
    pub namespace_id: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPairGetInput {
    pub account_id: String,
    pub namespace_id: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPairsGetInput {
    pub account_id: String,
    pub namespace_id: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvValuesGetInput {
    pub account_id: String,
    pub namespace_id: String,
    pub keys: Vec<String>,

    #[serde(rename = "type")]
    pub value_type: Option<KvValueType>,

    #[serde(rename = "withMetadata")]
    pub with_metadata: Option<bool>,
}

impl From<KvPairsGetInput> for KvValuesGetInput {
    fn from(value: KvPairsGetInput) -> Self {
        Self {
            account_id: value.account_id,
            namespace_id: value.namespace_id,
            keys: value.keys,
            value_type: Some(KvValueType::Text),
            with_metadata: Some(true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum KvValueType {
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "binary")]
    Json,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum KvValuesResult {
    Raw(KvValuesRaw),
    WithMetadata(KvValues),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvValuesRaw {
    pub values: HashMap<String, Value>,
}

impl From<ApiResponse<KvValuesRaw>> for KvValuesRaw {
    fn from(response: ApiResponse<KvValuesRaw>) -> Self {
        response.result
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvValues {
    pub values: HashMap<String, KvValue>,
}

impl From<ApiResponse<KvValues>> for KvValues {
    fn from(response: ApiResponse<KvValues>) -> Self {
        response.result
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvValue {
    pub value: Value,
    pub metadata: KvPairMetadata,

    #[serde(with = "ts_seconds_option")]
    pub expiration: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPair {
    pub key: String,
    pub value: Vec<u8>,

    #[serde(default)]
    pub metadata: KvPairMetadata,

    #[serde(with = "ts_seconds_option")]
    pub expiration: Option<DateTime<Utc>>,
}

pub type KvPairMetadata = Option<HashMap<String, Value>>;

impl From<KvPairGetInput> for KvPairMetadataGetInput {
    fn from(value: KvPairGetInput) -> Self {
        Self {
            account_id: value.account_id,
            namespace_id: value.namespace_id,
            key: value.key,
        }
    }
}

impl From<ApiResponse<KvPairMetadata>> for KvPairMetadata {
    fn from(response: ApiResponse<KvPairMetadata>) -> Self {
        response.result
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPairMetadataGetInput {
    pub account_id: String,
    pub namespace_id: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPairCreateInput {
    pub account_id: String,
    pub namespace_id: String,
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub expiration: Option<DateTime<Utc>>,
    pub expiration_ttl: Option<u32>,
    pub metadata: KvPairMetadata,
}

impl From<&KvPairCreateInput> for KvPairGetInput {
    fn from(input: &KvPairCreateInput) -> Self {
        Self {
            account_id: input.account_id.clone(),
            namespace_id: input.namespace_id.clone(),
            key: input.key.clone(),
        }
    }
}

impl From<KvPairCreateInput> for KvPairWriteInput {
    fn from(value: KvPairCreateInput) -> Self {
        Self {
            account_id: value.account_id,
            namespace_id: value.namespace_id,
            key: value.key,
            value: value.value,
            expiration: value.expiration,
            expiration_ttl: value.expiration_ttl,
            metadata: value.metadata,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvPairWriteInput {
    pub account_id: String,
    pub namespace_id: String,
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub expiration: Option<DateTime<Utc>>,
    pub expiration_ttl: Option<u32>,
    pub metadata: KvPairMetadata,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct KvPairsDeleteInput {
    pub account_id: String,
    pub namespace_id: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct KvPairsDeleteResult {
    pub successful_key_count: u32,
    pub unsuccessful_keys: Vec<String>,
}

impl From<ApiResponse<KvPairsDeleteResult>> for KvPairsDeleteResult {
    fn from(response: ApiResponse<KvPairsDeleteResult>) -> Self {
        response.result
    }
}

#[derive(Debug)]
pub enum KvError {
    NamespaceAlreadyExists(String),
    NamespaceNotFound,
    NamespaceTitleMissing(String),

    KeyNotFound,
    KeyAlreadyExists(String),
    InvalidMetadata,
    InvalidExpiration,

    NonTextValue,

    Token(TokenError),

    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for KvError {}

impl Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            KvError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            _ => write!(f, "KvError: {:?}", self),
        }
    }
}

impl From<reqwest::Error> for KvError {
    fn from(error: reqwest::Error) -> Self {
        KvError::Reqwest(error)
    }
}
