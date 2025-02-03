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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountWithToken {
    pub id: String,
    pub name: String,
    pub token: Token,
}
