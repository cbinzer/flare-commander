use crate::cloudflare::common::{
    ApiPaginatedResponse, ApiResponse, OrderDirection, PageInfo, TokenError,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
pub enum KvError {
    NamespaceAlreadyExists(String),
    NamespaceNotFound,
    NamespaceTitleMissing(String),

    KeyNotFound,
    KeyAlreadyExists(String),

    Token(TokenError),

    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for KvError {}

impl Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KvError: {:?}", self)
    }
}

impl From<reqwest::Error> for KvError {
    fn from(error: reqwest::Error) -> Self {
        KvError::Reqwest(error)
    }
}
