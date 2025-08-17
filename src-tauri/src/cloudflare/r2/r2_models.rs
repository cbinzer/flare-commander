use crate::cloudflare::common::{
    ApiCursorPaginatedResponse, CursorPageInfo, OrderDirection, TokenError,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Buckets {
    pub items: Vec<Bucket>,
    pub page_info: Option<CursorPageInfo>,
}

impl From<ApiCursorPaginatedResponse<BucketsListResponse>> for Buckets {
    fn from(value: ApiCursorPaginatedResponse<BucketsListResponse>) -> Self {
        Self {
            items: value.result.buckets,
            page_info: value.result_info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub creation_date: DateTime<Utc>,
    pub jurisdiction: Option<BucketJurisdiction>,
    pub location: Option<BucketLocation>,
    pub storage_class: Option<BucketStorageClass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BucketJurisdiction {
    #[serde(rename = "default")]
    Default,

    #[serde(rename = "eu")]
    Eu,

    #[serde(rename = "fedramp")]
    FedRamp,
}

impl Display for BucketJurisdiction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            BucketJurisdiction::Default => "default".to_string(),
            BucketJurisdiction::Eu => "eu".to_string(),
            BucketJurisdiction::FedRamp => "fedramp".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BucketLocation {
    Apac,
    EEur,
    ENam,
    WEur,
    WNam,
    Oc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BucketStorageClass {
    Standard,
    InfrequentAccess,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketsListInput {
    pub account_id: String,
    pub cursor: Option<String>,
    pub direction: Option<OrderDirection>,
    pub name_contains: Option<String>,
    pub order: Option<BucketOrder>,
    pub per_page: Option<u32>,
    pub start_after: Option<String>,
    pub jurisdiction: Option<BucketJurisdiction>,
}

impl From<BucketsListInput> for HashMap<String, String> {
    fn from(value: BucketsListInput) -> Self {
        let mut map = HashMap::new();

        if let Some(cursor) = value.cursor {
            map.insert("cursor".to_string(), cursor);
        }

        if let Some(direction) = value.direction {
            map.insert("direction".to_string(), direction.to_string());
        }

        if let Some(name_contains) = value.name_contains {
            map.insert("name_contains".to_string(), name_contains);
        }

        if let Some(order) = value.order {
            map.insert("order".to_string(), order.to_string());
        }

        if let Some(per_page) = value.per_page {
            map.insert("per_page".to_string(), per_page.to_string());
        }

        if let Some(start_after) = value.start_after {
            map.insert("start_after".to_string(), start_after);
        }

        map
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BucketOrder {
    #[serde(rename = "name")]
    Name,
}

impl Display for BucketOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            BucketOrder::Name => "name".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketsListResponse {
    pub buckets: Vec<Bucket>,
}

#[derive(Debug)]
pub enum R2Error {
    Token(TokenError),
    Bucket(BucketError),
    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for R2Error {}

impl Display for R2Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            R2Error::Token(err) => write!(f, "Token Error: {}", err),
            R2Error::Bucket(err) => write!(f, "Bucket Error: {}", err),
            R2Error::Reqwest(err) => write!(f, "Reqwest Error: {}", err),
            R2Error::Unknown(err) => write!(f, "Unknown Error: {}", err),
        }
    }
}

impl From<reqwest::Error> for R2Error {
    fn from(error: reqwest::Error) -> Self {
        R2Error::Reqwest(error)
    }
}

#[derive(Debug)]
pub enum BucketError {
    InvalidCursor,
    Validation(String),
}

impl Error for BucketError {}

impl Display for BucketError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "BucketError: {self:?}")
    }
}
