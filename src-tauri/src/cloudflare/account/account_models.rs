use crate::cloudflare::common::ApiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub created_on: Option<DateTime<Utc>>,
    pub settings: Option<AccountSettings>,
}

impl From<ApiResponse<Account>> for Account {
    fn from(api_response: ApiResponse<Account>) -> Self {
        api_response.result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AccountSettings {
    pub abuse_contact_email: Option<String>,
    pub enforce_twofactor: Option<bool>,
}

#[derive(Debug)]
pub enum AccountError {
    InvalidId,

    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for AccountError {}

impl Display for AccountError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            AccountError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            _ => write!(f, "AccountError: {:?}", self),
        }
    }
}

impl From<reqwest::Error> for AccountError {
    fn from(err: reqwest::Error) -> Self {
        AccountError::Reqwest(err)
    }
}
