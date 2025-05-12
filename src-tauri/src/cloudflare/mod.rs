use std::sync::Arc;

pub mod account_details;
mod kv;
pub mod read_key_value;

pub struct Cloudflare {
    api_url: String,
    credentials: Credentials,
    http_client: Arc<reqwest::Client>,

    pub kv: kv::Kv,
}

impl Cloudflare {
    pub fn new(credentials: Credentials, api_url: Option<String>) -> Self {
        let api_url = api_url.unwrap_or("https://api.cloudflare.com/client/v4".to_string());
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            api_url: api_url.clone(),
            credentials: credentials.clone(),
            http_client: http_client.clone(),
            kv: kv::Kv::new(credentials.clone(), Some(api_url), Some(http_client)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Credentials {
    UserAuthKey {
        account_id: String,
        email: String,
        key: String,
    },
    UserAuthToken {
        account_id: String,
        token: String,
    },
    Service {
        account_id: String,
        key: String,
    },
}

impl Credentials {
    pub fn headers(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::UserAuthKey {
                account_id: _account_id,
                email,
                key,
            } => {
                vec![("X-Auth-Email", email.clone()), ("X-Auth-Key", key.clone())]
            }
            Self::UserAuthToken {
                account_id: _account_id,
                token,
            } => {
                vec![("Authorization", format!("Bearer {}", token.clone()))]
            }
            Self::Service {
                account_id: _account_id,
                key,
            } => vec![("X-Auth-User-Service-Key", key.clone())],
        }
    }
}
