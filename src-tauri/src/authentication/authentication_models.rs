use crate::common::common_models::Credentials;
use cloudflare::framework::response::{ApiError, ApiFailure};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub success: bool,
    pub result: Option<T>,
    pub messages: Vec<ResponseInfo>,
    pub errors: Vec<ResponseInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Token {
    pub id: String,
    pub value: Option<String>,
    pub status: TokenStatus,
    pub policies: Option<Vec<TokenPolicy>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TokenStatus {
    Active,
    Disabled,
    Expired,
}

impl TryFrom<String> for TokenStatus {
    type Error = AuthenticationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "active" => Ok(TokenStatus::Active),
            "expired" => Ok(TokenStatus::Expired),
            "disabled" => Ok(TokenStatus::Disabled),
            _ => Err(AuthenticationError::Unknown(format!(
                "Unknown token status: {}",
                value
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TokenPolicy {
    pub id: String,
    pub effect: TokenPolicyEffect,
    pub permission_groups: Vec<PermissionGroup>,
    pub resources: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PermissionGroup {
    pub id: String,
    pub meta: Option<PermissionGroupMeta>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PermissionGroupMeta {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TokenPolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseInfo {
    pub code: u32,
    pub message: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
    InvalidAccountId(String),
    ExpiredToken,
    DisabledToken,
    InvalidToken,
    Reqwest(reqwest::Error),
    Unknown(String),
}

impl Error for AuthenticationError {}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AuthenticationError: {:?}", self)
    }
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(error: reqwest::Error) -> Self {
        AuthenticationError::Reqwest(error)
    }
}

impl From<cloudflare::framework::Error> for AuthenticationError {
    fn from(error: cloudflare::framework::Error) -> Self {
        let reqwest_error = match error {
            cloudflare::framework::Error::ReqwestError(reqwest_error) => reqwest_error,
        };
        AuthenticationError::Reqwest(reqwest_error)
    }
}

impl From<ApiFailure> for AuthenticationError {
    fn from(error: ApiFailure) -> Self {
        match error {
            ApiFailure::Error(_, api_errors) => map_api_errors(api_errors.errors),
            ApiFailure::Invalid(reqwest_error) => AuthenticationError::Reqwest(reqwest_error),
        }
    }
}

pub fn map_api_errors(errors: Vec<ApiError>) -> AuthenticationError {
    if errors.is_empty() {
        return AuthenticationError::Unknown("No errors in the response.".to_string());
    }

    let error = &errors[0];
    match error.code {
        1000 => AuthenticationError::InvalidToken,
        6003 => AuthenticationError::InvalidToken,
        7003 => AuthenticationError::InvalidAccountId(error.message.clone()),
        9109 => AuthenticationError::InvalidAccountId(error.message.clone()),
        _ => AuthenticationError::Unknown(error.message.clone()),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountWithCredentials {
    pub id: String,
    pub name: String,
    pub credentials: AccountCredentials,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AccountCredentials {
    UserAuthKey { email: String, key: String },
    UserAuthToken { token: String },
    Service { key: String },
}

impl From<Credentials> for AccountCredentials {
    fn from(value: Credentials) -> Self {
        match value {
            Credentials::UserAuthKey { email, key, .. } => {
                AccountCredentials::UserAuthKey { email, key }
            }
            Credentials::UserAuthToken { token, .. } => AccountCredentials::UserAuthToken { token },
            Credentials::Service { key, .. } => AccountCredentials::Service { key },
        }
    }
}
