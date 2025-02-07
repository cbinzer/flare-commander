use crate::authentication::authentication_service::AuthenticationService;
use crate::kv::kv_service::KvService;
use std::sync::Arc;

pub struct AppState {
    pub auth_service: AuthenticationService,
    pub kv_service: KvService,
}

impl Default for AppState {
    fn default() -> Self {
        let api_url = "https://api.cloudflare.com";
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            auth_service: AuthenticationService::new(api_url, http_client.clone()),
            kv_service: KvService::new(None),
        }
    }
}
