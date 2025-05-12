pub mod account_details;
pub mod read_key_value;

pub struct Cloudflare {
    api_url: String,
    credentials: Credentials,
    http_client: reqwest::Client,
}

impl Cloudflare {
    pub fn new(credentials: Credentials, api_url: Option<String>) -> Self {
        Self {
            api_url: api_url.unwrap_or("https://api.cloudflare.com/client/v4".to_string()),
            credentials,
            http_client: reqwest::Client::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Credentials {
    UserAuthKey { email: String, key: String },
    UserAuthToken { token: String },
    Service { key: String },
}

impl Credentials {
    pub fn headers(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::UserAuthKey { email, key } => {
                vec![("X-Auth-Email", email.clone()), ("X-Auth-Key", key.clone())]
            }
            Self::UserAuthToken { token } => {
                vec![("Authorization", format!("Bearer {}", token.clone()))]
            }
            Self::Service { key } => vec![("X-Auth-User-Service-Key", key.clone())],
        }
    }
}
