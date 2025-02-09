use cloudflare::framework::auth::Credentials as CloudflareCredentials;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
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

impl From<Credentials> for CloudflareCredentials {
    fn from(credentials: Credentials) -> Self {
        match credentials {
            Credentials::UserAuthKey {
                account_id: _,
                email,
                key,
            } => CloudflareCredentials::UserAuthKey { email, key },
            Credentials::UserAuthToken {
                account_id: _,
                token,
            } => CloudflareCredentials::UserAuthToken { token },
            Credentials::Service { account_id: _, key } => CloudflareCredentials::Service { key },
        }
    }
}

impl From<&Credentials> for CloudflareCredentials {
    fn from(credentials: &Credentials) -> Self {
        match credentials {
            Credentials::UserAuthKey {
                account_id: _,
                email,
                key,
            } => CloudflareCredentials::UserAuthKey {
                email: email.to_string(),
                key: key.to_string(),
            },
            Credentials::UserAuthToken {
                account_id: _,
                token,
            } => CloudflareCredentials::UserAuthToken {
                token: token.to_string(),
            },
            Credentials::Service { account_id: _, key } => CloudflareCredentials::Service {
                key: key.to_string(),
            },
        }
    }
}

impl Credentials {
    pub fn account_id(&self) -> &str {
        match self {
            Credentials::UserAuthKey { account_id, .. } => account_id,
            Credentials::UserAuthToken { account_id, .. } => account_id,
            Credentials::Service { account_id, .. } => account_id,
        }
    }
}
