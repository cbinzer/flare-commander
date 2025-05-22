use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApiPaginatedResponse<T> {
    pub result: T,
    pub result_info: PageInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApiCursorPaginatedResponse<T> {
    pub result: T,
    pub result_info: CursorPageInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApiResponse<T> {
    pub result: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApiErrorResponse {
    pub errors: Vec<ApiError>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PageInfo {
    pub count: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CursorPageInfo {
    pub count: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,

    #[serde(rename = "desc")]
    Desc,
}

impl Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OrderDirection::Asc => "asc".to_string(),
            OrderDirection::Desc => "desc".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Credentials {
    UserAuthKey { email: String, key: String },
    UserAuthToken { token: String },
    Service { key: String },
}

impl Credentials {
    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        match self {
            Self::UserAuthKey { email, key } => {
                headers.insert(
                    "X-Auth-Email",
                    HeaderValue::from_str(email).expect("Invalid email"),
                );
                headers.insert(
                    "X-Auth-Key",
                    HeaderValue::from_str(key).expect("Invalid auth key"),
                );
            }
            Self::UserAuthToken { token } => {
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {}", token)).expect("Invalid token"),
                );
            }
            Self::Service { key } => {
                headers.insert(
                    "X-Auth-User-Service-Key",
                    HeaderValue::from_str(key).expect("Invalid service key"),
                );
            }
        }

        headers
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenError {
    Expired,
    Disabled,
    Invalid,
    Unknown(String),
}

impl Error for TokenError {}

impl Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TokenError: {:?}", self)
    }
}
