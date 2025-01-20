use crate::authentication::authentication_service::AuthenticationService;
use std::sync::Arc;

pub struct AppState {
    pub auth_service: AuthenticationService,
}

impl Default for AppState {
    fn default() -> Self {
        let api_url = "https://api.cloudflare.com";
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            auth_service: AuthenticationService::new(api_url, http_client),
        }
    }
}
