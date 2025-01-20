use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Envelope<T> {
    pub success: bool,
    pub result: Option<T>,
    pub messages: Vec<ResponseInfo>,
    pub errors: Vec<ResponseInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub status: TokenStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenStatus {
    Active,
    Disabled,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub code: u32,
    pub message: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
    InvalidAccountId(String),
    InvalidToken,
    Reqwest(reqwest::Error),
    Unknown(String),
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
