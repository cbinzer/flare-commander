use crate::cloudflare::common::TokenError;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum UserError {
    Token(TokenError),

    Reqwest(reqwest::Error),
    Unknown(String),
}

impl From<reqwest::Error> for UserError {
    fn from(err: reqwest::Error) -> Self {
        UserError::Reqwest(err)
    }
}

impl Error for UserError {}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            UserError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            _ => write!(f, "UserError: {:?}", self),
        }
    }
}
