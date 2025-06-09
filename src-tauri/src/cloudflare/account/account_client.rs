use crate::cloudflare::account::{Account, AccountError};
use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse, Credentials, API_URL};
use reqwest::{Response, StatusCode};
use serde::Deserialize;
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

    pub async fn get_account(&self, account_id: &str) -> Result<Account, AccountError> {
        let url = format!("{}/accounts/{}", self.api_url, account_id,);

        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<Account>, Account>(response)
            .await
    }

    async fn handle_api_response<T: for<'a> Deserialize<'a>, R: From<T>>(
        &self,
        response: Response,
    ) -> Result<R, AccountError> {
        match response.status() {
            StatusCode::OK => {
                let api_result: T = response.json().await?;
                Ok(api_result.into())
            }
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    async fn handle_api_error_response(&self, response: Response) -> AccountError {
        let api_response_result = response.json::<ApiErrorResponse>().await;
        match api_response_result {
            Ok(api_response) => self.map_api_errors(api_response.errors),
            Err(error) => error.into(),
        }
    }

    fn map_api_errors(&self, errors: Vec<ApiError>) -> AccountError {
        if errors.is_empty() {
            return AccountError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            7003 => AccountError::InvalidId,
            9109 => AccountError::InvalidId,
            _ => AccountError::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cloudflare::common::Credentials;
    use std::sync::Arc;

    mod get_account {
        use crate::cloudflare::account::account_client::test::create_account_client;
        use crate::cloudflare::account::{Account, AccountError, AccountSettings};
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use chrono::Utc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        pub async fn should_get_an_account() -> Result<(), AccountError> {
            let expected_account = Account {
                id: "12345".to_string(),
                name: "Test Account".to_string(),
                created_on: Some(Utc::now()),
                settings: Some(AccountSettings {
                    abuse_contact_email: Some("test@example.com".to_string()),
                    enforce_twofactor: Some(true),
                }),
            };
            let mock_server = create_succeeding_mock_server(&expected_account).await;

            let account_client = create_account_client(&mock_server.uri());
            let account = account_client.get_account(&expected_account.id).await?;

            assert_eq!(account, expected_account);

            Ok(())
        }

        #[tokio::test]
        pub async fn should_return_an_error_for_invalid_account_id() -> Result<(), AccountError> {
            let account_id = "invalid_id";
            let errors = vec![ApiError {
                code: 7003,
                message: "Invalid account ID".to_string(),
            }];
            let mock_server = create_failing_mock_server(account_id, errors).await;

            let account_client = create_account_client(&mock_server.uri());
            let result = account_client.get_account(account_id).await;

            assert!(matches!(result, Err(AccountError::InvalidId)));

            Ok(())
        }

        pub async fn create_succeeding_mock_server(account: &Account) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template_value =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<Account> {
                    result: account.clone(),
                });

            Mock::given(method("GET"))
                .and(path(format!("/client/v4/accounts/{}", account.id,)))
                .respond_with(response_template_value)
                .mount(&mock_server)
                .await;

            mock_server
        }

        pub async fn create_failing_mock_server(
            account_id: &str,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors });

            Mock::given(method("GET"))
                .and(path(format!("/client/v4/accounts/{account_id}")))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    pub fn create_account_client(host_url: &str) -> super::AccountClient {
        super::AccountClient::new(
            Arc::new(Credentials::UserAuthToken {
                token: "12345".to_string(),
            }),
            Some(Arc::new(format!("{}/client/v4", host_url))),
            None,
        )
    }
}
