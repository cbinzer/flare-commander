use crate::cloudflare::common::{Credentials, API_URL};
use crate::cloudflare::kv::Kv;
use std::sync::Arc;

pub mod account_details;
pub(crate) mod kv;

pub(crate) mod common;

pub struct Cloudflare {
    pub kv: Kv,
}

impl Cloudflare {
    pub fn new(credentials: Credentials, api_url: Option<String>) -> Self {
        let api_url = api_url.unwrap_or(API_URL.to_string());
        let credentials = Arc::new(credentials);
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            kv: Kv::new(credentials, Some(api_url), Some(http_client)),
        }
    }
}
