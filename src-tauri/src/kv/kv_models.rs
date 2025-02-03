use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagePaginationArray<T> {
    pub success: bool,
    pub result: Option<T>,
    pub messages: Vec<ResponseInfo>,
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
