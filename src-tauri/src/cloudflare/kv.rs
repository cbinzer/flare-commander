use crate::cloudflare::Credentials;
use std::sync::Arc;

pub struct Kv {
    api_url: String,
    credentials: Credentials,
    http_client: Arc<reqwest::Client>,
}

impl Kv {
    pub fn new(
        credentials: Credentials,
        api_url: Option<String>,
        http_client: Option<Arc<reqwest::Client>>,
    ) -> Self {
        Self {
            api_url: api_url.unwrap_or("https://api.cloudflare.com/client/v4".to_string()),
            credentials,
            http_client: http_client.unwrap_or_default(),
        }
    }
}
