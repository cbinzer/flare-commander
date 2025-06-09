use crate::cloudflare::common::{
    ApiError, ApiErrorResponse, ApiResponse, Credentials, Token, TokenError, API_URL,
};
use crate::cloudflare::user::UserError;
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use std::sync::Arc;

pub struct UserClient {
    api_url: Arc<String>,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,
}

impl UserClient {
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

    pub async fn verify_token(&self) -> Result<Token, UserError> {
        let url = format!("{}/user/tokens/verify", self.api_url);

        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<Token>, Token>(response)
            .await
    }

    async fn handle_api_response<T: for<'a> Deserialize<'a>, R: From<T>>(
        &self,
        response: Response,
    ) -> Result<R, UserError> {
        match response.status() {
            StatusCode::OK => {
                let api_result: T = response.json().await?;
                Ok(api_result.into())
            }
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    async fn handle_api_error_response(&self, response: Response) -> UserError {
        let api_response_result = response.json::<ApiErrorResponse>().await;
        match api_response_result {
            Ok(api_response) => self.map_api_errors(api_response.errors),
            Err(error) => error.into(),
        }
    }

    fn map_api_errors(&self, errors: Vec<ApiError>) -> UserError {
        if errors.is_empty() {
            return UserError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            1000 => UserError::Token(TokenError::Invalid),
            1001 => UserError::Token(TokenError::Invalid),
            6003 => UserError::Token(TokenError::Invalid),
            _ => UserError::Unknown(error.message.clone()),
        }
    }
}
