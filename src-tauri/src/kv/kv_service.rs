use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
use crate::kv::kv_models::{KvError, KvNamespace, PagePaginationArray};
use reqwest::Client;
use std::sync::Arc;

pub struct KvService {
    api_url: String,
    http_client: Arc<Client>,
}

impl KvService {
    pub fn new(api_url: &str, http_client: Arc<Client>) -> Self {
        Self {
            api_url: api_url.to_string(),
            http_client,
        }
    }

    pub async fn get_namespaces(
        &self,
        account_id: &str,
        token: &str,
    ) -> Result<Vec<KvNamespace>, KvError> {
        let namespaces_result: PagePaginationArray<Vec<KvNamespace>> = self
            .http_client
            .get(format!(
                "{}/client/v4/accounts/{}/storage/kv/namespaces",
                self.api_url, account_id
            ))
            .bearer_auth(token)
            .query(&[("per_page", "100")])
            .send()
            .await?
            .json()
            .await?;

        match namespaces_result.result {
            None => Err(self.map_errors(namespaces_result.errors)),
            Some(namespaces) => Ok(namespaces),
        }
    }

    fn map_errors(&self, errors: Vec<ResponseInfo>) -> KvError {
        if errors.is_empty() {
            return KvError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            10000 => KvError::Authentication(AuthenticationError::InvalidToken),
            10001 => KvError::Authentication(AuthenticationError::InvalidToken),
            _ => KvError::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    mod get_namespaces {
        use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
        use crate::kv::kv_models::KvError::Authentication;
        use crate::kv::kv_models::{KvError, KvNamespace, PagePaginationArray, PaginationInfo};
        use crate::kv::kv_service::KvService;
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_namespaces() -> Result<(), KvError> {
            let expected_namespaces = vec![
                KvNamespace {
                    id: "namespace_id_1".to_string(),
                    title: "namespace_title_1".to_string(),
                    supports_url_encoding: None,
                },
                KvNamespace {
                    id: "namespace_id_2".to_string(),
                    title: "namespace_title_2".to_string(),
                    supports_url_encoding: Some(true),
                },
                KvNamespace {
                    id: "namespace_id_3".to_string(),
                    title: "namespace_title_3".to_string(),
                    supports_url_encoding: Some(false),
                },
            ];
            let account_id = "account_id";

            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200).set_body_json(PagePaginationArray {
                success: true,
                result: Some(expected_namespaces.clone()),
                errors: vec![],
                result_info: Some(PaginationInfo {
                    total_count: Some(3),
                    count: Some(3),
                    page: Some(1),
                    per_page: Some(20),
                }),
            });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service =
                KvService::new(mock_server.uri().as_str(), Arc::new(reqwest::Client::new()));
            let namespaces = kv_service.get_namespaces(account_id, "token").await?;

            assert_eq!(namespaces.len(), 3);
            assert_eq!(namespaces, expected_namespaces);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_an_unknown_error_occurred(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id";
            let unknown_error_message = "Unknown error.";
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(400).set_body_json(PagePaginationArray::<Vec<KvNamespace>> {
                    success: false,
                    result: None,
                    errors: vec![ResponseInfo {
                        code: 1111,
                        message: unknown_error_message.to_string(),
                    }],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service =
                KvService::new(mock_server.uri().as_str(), Arc::new(reqwest::Client::new()));
            let namespaces_result = kv_service.get_namespaces(account_id, "token").await;

            assert!(namespaces_result.is_err());

            let error = namespaces_result.unwrap_err();
            assert!(matches!(error, KvError::Unknown(_)));

            let error_message = match error {
                KvError::Unknown(message) => message,
                _ => "".to_string(),
            };
            assert_eq!(error_message, unknown_error_message);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id";
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(400).set_body_json(PagePaginationArray::<Vec<KvNamespace>> {
                    success: false,
                    result: None,
                    errors: vec![],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service =
                KvService::new(mock_server.uri().as_str(), Arc::new(reqwest::Client::new()));
            let namespaces_result = kv_service.get_namespaces(account_id, "token").await;

            assert!(namespaces_result.is_err());

            let error = namespaces_result.unwrap_err();
            assert!(matches!(error, KvError::Unknown(_)));

            let error_message = match error {
                KvError::Unknown(message) => message,
                _ => "".to_string(),
            };
            assert_eq!(error_message, "No errors in the response.");

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_authentication_error_if_the_request_could_not_be_authenticated(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id";
            let error_message = "Unable to authenticate request";
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(400).set_body_json(PagePaginationArray::<Vec<KvNamespace>> {
                    success: false,
                    result: None,
                    errors: vec![ResponseInfo {
                        code: 10001,
                        message: error_message.to_string(),
                    }],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service =
                KvService::new(mock_server.uri().as_str(), Arc::new(reqwest::Client::new()));
            let namespaces_result = kv_service.get_namespaces(account_id, "token").await;

            assert!(namespaces_result.is_err());

            let error = namespaces_result.unwrap_err();
            assert!(matches!(error, Authentication(_)));

            let authentication_error = match error {
                Authentication(error) => error,
                _ => AuthenticationError::Unknown("".to_string()),
            };
            assert!(matches!(
                authentication_error,
                AuthenticationError::InvalidToken
            ));

            Ok(())
        }
    }
}
