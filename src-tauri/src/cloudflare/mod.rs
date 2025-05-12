use crate::cloudflare::common::{Credentials, API_URL};
use crate::cloudflare::kv::Kv;
use std::sync::Arc;

pub mod account_details;
mod kv;

mod common;
pub mod read_key_value;

pub struct Cloudflare {
    api_url: String,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,

    pub kv: Kv,
}

impl Cloudflare {
    pub fn new(credentials: Credentials, api_url: Option<String>) -> Self {
        let api_url = api_url.unwrap_or(API_URL.to_string());
        let credentials = Arc::new(credentials);
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            api_url: api_url.clone(),
            credentials: credentials.clone(),
            http_client: http_client.clone(),
            kv: Kv::new(credentials, Some(api_url), Some(http_client)),
        }
    }
}
