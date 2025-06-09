use crate::cloudflare::common::{Credentials, API_URL};
use std::sync::Arc;

pub struct AccountClient {
    api_url: Arc<String>,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,
}

impl AccountClient {
    pub fn new(
        credentials: Arc<Credentials>,
        api_url: Option<Arc<String>>,
        http_client: Option<Arc<reqwest::Client>>,
    ) -> Self {
        Self {
            api_url: api_url.unwrap_or(Arc::new(API_URL.to_string())),
            credentials,
            http_client: http_client.unwrap_or_default(),
        }
    }
}
