use crate::cloudflare::account::AccountClient;
use crate::cloudflare::common::{Credentials, API_URL};
use crate::cloudflare::kv::KvClient;
use crate::cloudflare::user::UserClient;
use std::sync::Arc;

pub mod account_details;
pub(crate) mod kv;

mod account;
pub(crate) mod common;
mod user;

pub struct Cloudflare {
    pub accounts: AccountClient,
    pub kv: KvClient,
    pub user: UserClient,
}

impl Cloudflare {
    pub fn new(credentials: Credentials, api_url: Option<String>) -> Self {
        let api_url = Arc::new(api_url.unwrap_or(API_URL.to_string()));
        let credentials = Arc::new(credentials);
        let http_client = Arc::new(reqwest::Client::new());

        Self {
            accounts: AccountClient::new(
                credentials.clone(),
                Some(api_url.clone()),
                Some(http_client.clone()),
            ),
            kv: KvClient::new(
                credentials.clone(),
                Some(api_url.clone()),
                Some(http_client.clone()),
            ),
            user: UserClient::new(credentials, Some(api_url), Some(http_client)),
        }
    }
}
