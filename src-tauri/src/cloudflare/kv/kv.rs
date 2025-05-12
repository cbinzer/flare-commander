use crate::cloudflare::common::{
    ApiError, ApiErrorResponse, ApiPaginatedResponse, Credentials, API_URL,
};
use crate::cloudflare::kv::{KvError, KvNamespace, KvNamespaces, KvNamespacesListInput};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Kv {
    api_url: String,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,
}

impl Kv {
    pub fn new(
        credentials: Arc<Credentials>,
        api_url: Option<String>,
        http_client: Option<Arc<reqwest::Client>>,
    ) -> Self {
        Self {
            api_url: api_url.unwrap_or(API_URL.to_string()),
            credentials,
            http_client: http_client.unwrap_or_default(),
        }
    }

    pub async fn list_namespaces(
        &self,
        input: KvNamespacesListInput,
    ) -> Result<KvNamespaces, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces",
            self.api_url, input.account_id,
        );

        let query_parameters: HashMap<String, String> = input.into();
        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .query(&query_parameters)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let api_result: ApiPaginatedResponse<Vec<KvNamespace>> = response.json().await?;
                Ok(api_result.into())
            }
            _ => {
                let api_response = response.json::<ApiErrorResponse>().await?;
                Err(self.map_api_errors(api_response.errors))
            }
        }
    }

    fn map_api_errors(&self, errors: Vec<ApiError>) -> KvError {
        if errors.is_empty() {
            return KvError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            // 10000 => KvError::Authentication(AuthenticationError::InvalidToken),
            // 10001 => KvError::Authentication(AuthenticationError::InvalidToken),
            10009 => KvError::KeyNotFound,
            10013 => KvError::NamespaceNotFound,
            10014 => KvError::NamespaceAlreadyExists(error.message.clone()),
            10019 => KvError::NamespaceTitleMissing(error.message.clone()),
            _ => KvError::Unknown(error.message.clone()),
        }
    }
}
