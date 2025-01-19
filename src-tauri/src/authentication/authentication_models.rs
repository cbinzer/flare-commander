use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    pub success: bool,
    pub result: Option<TokenVerification>,
    pub messages: Vec<ResponseInfo>,
    pub errors: Vec<ResponseInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenVerification {
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenVerificationResult {
    Active,
    Disabled,
    Expired,
    Invalid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub code: u32,
    pub message: String,
}

impl From<TokenStatus> for TokenVerificationResult {
    fn from(status: TokenStatus) -> TokenVerificationResult {
        match status {
            TokenStatus::Active => TokenVerificationResult::Active,
            TokenStatus::Disabled => TokenVerificationResult::Disabled,
            TokenStatus::Expired => TokenVerificationResult::Expired,
        }
    }
}

#[derive(Debug)]
pub enum AuthenticationError {
    RequestError(reqwest::Error),
    Unknown(String),
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(error: reqwest::Error) -> Self {
        AuthenticationError::RequestError(error)
    }
}
