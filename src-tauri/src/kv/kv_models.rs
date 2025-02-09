use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
use chrono::{DateTime, Utc};
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
        let reqwest_error = match error {
            cloudflare::framework::Error::ReqwestError(reqwest_error) => reqwest_error,
        };
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

fn map_api_errors(errors: Vec<ApiError>) -> KvError {
    if errors.is_empty() {
        return KvError::Unknown("No errors in the response.".to_string());
    }

    let error = &errors[0];
    match error.code {
        10000 => KvError::Authentication(AuthenticationError::InvalidToken),
        10001 => KvError::Authentication(AuthenticationError::InvalidToken),
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
pub struct KvKeyValueList {
    pub data: Vec<KvKeyValue>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KvKeyValue {
    pub key: String,
    pub value: String,
    pub expiration: Option<DateTime<Utc>>,
}
