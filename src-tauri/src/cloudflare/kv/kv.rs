use crate::cloudflare::common::{
    ApiError, ApiErrorResponse, ApiPaginatedResponse, ApiResponse, Credentials, TokenError, API_URL,
};
use crate::cloudflare::kv::{
    KvError, KvNamespace, KvNamespaceCreateInput, KvNamespaceDeleteInput, KvNamespaceGetInput,
    KvNamespaceUpdateInput, KvNamespaces, KvNamespacesListInput,
};
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::option::Option;
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

        self.handle_api_response::<ApiPaginatedResponse<Vec<KvNamespace>>, KvNamespaces>(response)
            .await
    }

    pub async fn get_namespace(&self, input: KvNamespaceGetInput) -> Result<KvNamespace, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}",
            self.api_url, input.account_id, input.namespace_id
        );

        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<KvNamespace>, KvNamespace>(response)
            .await
    }

    pub async fn create_namespace(
        &self,
        input: KvNamespaceCreateInput,
    ) -> Result<KvNamespace, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces",
            self.api_url, input.account_id
        );

        let response = self
            .http_client
            .post(&url)
            .headers(self.credentials.headers())
            .json(&HashMap::from([("title", input.title)]))
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<KvNamespace>, KvNamespace>(response)
            .await
    }

    pub async fn update_namespace(
        &self,
        input: KvNamespaceUpdateInput,
    ) -> Result<KvNamespace, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}",
            self.api_url, input.account_id, input.namespace_id
        );

        let response = self
            .http_client
            .put(&url)
            .headers(self.credentials.headers())
            .json(&HashMap::from([("title", input.title)]))
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<KvNamespace>, KvNamespace>(response)
            .await
    }

    pub async fn delete_namespace(&self, input: KvNamespaceDeleteInput) -> Result<(), KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}",
            self.api_url, input.account_id, input.namespace_id
        );

        let response = self
            .http_client
            .delete(&url)
            .headers(self.credentials.headers())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            _ => {
                let api_response = response.json::<ApiErrorResponse>().await?;
                Err(self.map_api_errors(api_response.errors))
            }
        }
    }

    async fn handle_api_response<T: for<'a> Deserialize<'a>, R: From<T>>(
        &self,
        response: Response,
    ) -> Result<R, KvError> {
        match response.status() {
            StatusCode::OK => {
                let api_result: T = response.json().await?;
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
            10000 => KvError::Token(TokenError::Invalid),
            10001 => KvError::Token(TokenError::Invalid),
            10009 => KvError::KeyNotFound,
            10013 => KvError::NamespaceNotFound,
            10014 => KvError::NamespaceAlreadyExists(error.message.clone()),
            10019 => KvError::NamespaceTitleMissing(error.message.clone()),
            _ => KvError::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cloudflare::common::Credentials;
    use crate::cloudflare::kv::Kv;
    use std::sync::Arc;

    mod list_namespaces {
        use crate::authentication::authentication_models::AuthenticationError;
        use crate::cloudflare::common::{OrderDirection, PageInfo, TokenError};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{
            KvError, KvNamespace, KvNamespaces, KvNamespacesListInput, KvNamespacesOrderBy,
        };
        use crate::common::common_models::{ApiError, ApiErrorResponse, ApiPaginatedResponse};
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_list_namespaces() -> Result<(), KvError> {
            let expected_namespaces = KvNamespaces {
                items: vec![
                    KvNamespace {
                        id: "namespace_id_1".to_string(),
                        title: "namespace_title_1".to_string(),
                        beta: Some(false),
                        supports_url_encoding: Some(true),
                    },
                    KvNamespace {
                        id: "namespace_id_2".to_string(),
                        title: "namespace_title_2".to_string(),
                        beta: Some(false),
                        supports_url_encoding: Some(true),
                    },
                    KvNamespace {
                        id: "namespace_id_3".to_string(),
                        title: "namespace_title_3".to_string(),
                        beta: Some(false),
                        supports_url_encoding: Some(true),
                    },
                ],
                page_info: PageInfo {
                    total_count: 3,
                    count: 3,
                    page: 1,
                    per_page: 20,
                },
            };

            let list_namespaces_input = KvNamespacesListInput {
                account_id: "account_id".to_string(),
                order_by: Some(KvNamespacesOrderBy::Title),
                order_direction: Some(OrderDirection::Asc),
                page: Some(1),
                per_page: Some(10),
            };
            let mock_server = create_succeeding_mock_server(
                list_namespaces_input.clone(),
                expected_namespaces.clone(),
            )
            .await;

            let kv = create_kv(mock_server.uri().to_string());
            let namespaces = kv.list_namespaces(list_namespaces_input).await?;
            assert_eq!(namespaces.items.len(), 3);
            assert_eq!(namespaces, expected_namespaces);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id".to_string();
            let mock_server = create_failing_mock_server(&account_id, vec![]).await;

            let kv = create_kv(mock_server.uri().to_string());
            let namespaces_result = kv
                .list_namespaces(KvNamespacesListInput {
                    account_id: account_id.clone(),
                    order_by: None,
                    order_direction: None,
                    page: None,
                    per_page: None,
                })
                .await;

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
        async fn should_respond_with_an_token_error_if_the_request_could_not_be_authenticated(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id".to_string();
            let error_message = "Unable to authenticate request";
            let mock_server = create_failing_mock_server(
                &account_id,
                vec![ApiError {
                    code: 10001,
                    message: error_message.to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let namespaces_result = kv
                .list_namespaces(KvNamespacesListInput {
                    account_id: account_id.clone(),
                    order_by: None,
                    order_direction: None,
                    page: None,
                    per_page: None,
                })
                .await;

            assert!(namespaces_result.is_err());

            let error = namespaces_result.unwrap_err();
            assert!(matches!(error, KvError::Token(_)));

            let user_error = match error {
                KvError::Token(error) => error,
                _ => TokenError::Unknown("".to_string()),
            };
            assert!(matches!(user_error, TokenError::Invalid));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: KvNamespacesListInput,
            namespaces: KvNamespaces,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template = ResponseTemplate::new(200).set_body_json(
                ApiPaginatedResponse::<Vec<KvNamespace>> {
                    result: namespaces.clone().items,
                    result_info: crate::common::common_models::PageInfo {
                        total_count: namespaces.items.len(),
                        count: namespaces.items.len(),
                        page: 1,
                        per_page: 20,
                    },
                },
            );

            let mut mock_builder = Mock::given(method("GET")).and(path(format!(
                "/client/v4/accounts/{}/storage/kv/namespaces",
                input.account_id
            )));

            if let Some(order_by) = input.order_by {
                mock_builder = mock_builder.and(query_param("order", order_by.to_string()));
            }
            if let Some(order_direction) = input.order_direction {
                mock_builder =
                    mock_builder.and(query_param("direction", order_direction.to_string()));
            }
            if let Some(page) = input.page {
                mock_builder = mock_builder.and(query_param("page", page.to_string()));
            }
            if let Some(per_page) = input.per_page {
                mock_builder = mock_builder.and(query_param("per_page", per_page.to_string()));
            }

            mock_builder
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(account_id: &str, errors: Vec<ApiError>) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces"
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod get_namespace {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvNamespace, KvNamespaceGetInput};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_namespace() -> Result<(), KvError> {
            let expected_namespace = KvNamespace {
                id: "12345".to_string(),
                title: "MyNamespace".to_string(),
                beta: Some(false),
                supports_url_encoding: Some(false),
            };

            let input = KvNamespaceGetInput {
                account_id: "my_account_id".to_string(),
                namespace_id: expected_namespace.id.clone(),
            };
            let mock_server =
                create_succeeding_mock_server(input.clone(), expected_namespace.clone()).await;

            let kv = create_kv(mock_server.uri());
            let namespace = kv.get_namespace(input).await?;

            assert_eq!(namespace, expected_namespace);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error() -> Result<(), KvError> {
            let input = KvNamespaceGetInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "12345".to_string(),
            };

            let mock_server = create_failing_mock_server(
                input.clone(),
                vec![ApiError {
                    code: 10013,
                    message: "get namespace: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let namespace_result = kv.get_namespace(input).await;

            assert!(namespace_result.is_err());

            let error = namespace_result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: KvNamespaceGetInput,
            namespace: KvNamespace,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvNamespace> {
                    result: namespace.clone(),
                });

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: KvNamespaceGetInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod create_namespace {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvNamespace, KvNamespaceCreateInput};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_create_namespace() -> Result<(), KvError> {
            let expected_namespace = KvNamespace {
                id: "12345".to_string(),
                title: "MyNamespace".to_string(),
                beta: Some(false),
                supports_url_encoding: Some(false),
            };
            let create_input = KvNamespaceCreateInput {
                account_id: "my_account_id".to_string(),
                title: expected_namespace.title.clone(),
            };
            let mock_server =
                create_succeeding_mock_server(create_input.clone(), expected_namespace.clone())
                    .await;

            let kv = create_kv(mock_server.uri());
            let created_namespace = kv.create_namespace(create_input).await?;

            assert_eq!(created_namespace, expected_namespace);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_already_exists_error() -> Result<(), KvError> {
            let expected_error_message =
                "create namespace: 'a namespace with this account ID and title already exists'";
            let create_input = KvNamespaceCreateInput {
                account_id: "my_account_id".to_string(),
                title: "MyNamespace".to_string(),
            };
            let mock_server = create_failing_mock_server(
                create_input.clone(),
                vec![ApiError {
                    code: 10014,
                    message: expected_error_message.to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let created_namespace_result = kv.create_namespace(create_input).await;
            assert!(created_namespace_result.is_err());

            let error = created_namespace_result.unwrap_err();
            assert!(
                matches!(error, KvError::NamespaceAlreadyExists(ref error_message) if error_message == expected_error_message)
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_title_missing_error() -> Result<(), KvError> {
            let expected_error_message = "request is missing a title definition";
            let create_input = KvNamespaceCreateInput {
                account_id: "my_account_id".to_string(),
                title: "".to_string(),
            };
            let mock_server = create_failing_mock_server(
                create_input.clone(),
                vec![ApiError {
                    code: 10019,
                    message: expected_error_message.to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let created_namespace_result = kv.create_namespace(create_input).await;
            assert!(created_namespace_result.is_err());

            let error = created_namespace_result.unwrap_err();
            assert!(
                matches!(error, KvError::NamespaceTitleMissing(ref error_message) if error_message == expected_error_message)
            );

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: KvNamespaceCreateInput,
            namespace: KvNamespace,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvNamespace> {
                    result: namespace.clone(),
                });

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces",
                    input.account_id
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: KvNamespaceCreateInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces",
                    input.account_id
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod update_namespace {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvNamespace, KvNamespaceUpdateInput};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_update_namespace() -> Result<(), KvError> {
            let expected_namespace = KvNamespace {
                id: "12345".to_string(),
                title: "MyNamespace".to_string(),
                beta: Some(false),
                supports_url_encoding: Some(false),
            };
            let update_input = KvNamespaceUpdateInput {
                account_id: "my_account_id".to_string(),
                namespace_id: expected_namespace.id.clone(),
                title: expected_namespace.title.clone(),
            };
            let mock_server =
                create_succeeding_mock_server(update_input.clone(), expected_namespace.clone())
                    .await;
            let kv = create_kv(mock_server.uri());
            let updated_namespace = kv.update_namespace(update_input).await?;

            assert_eq!(updated_namespace, expected_namespace);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_already_exists_error() -> Result<(), KvError> {
            let namespace_id = "12345";
            let expected_error_message =
                "rename namespace: 'a namespace with this account ID and title already exists'";
            let update_input = KvNamespaceUpdateInput {
                account_id: "my_account_id".to_string(),
                namespace_id: namespace_id.to_string(),
                title: "MyNamespace".to_string(),
            };
            let mock_server = create_failing_mock_server(
                update_input.clone(),
                vec![ApiError {
                    code: 10014,
                    message: expected_error_message.to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let updated_namespace_result = kv.update_namespace(update_input).await;
            assert!(updated_namespace_result.is_err());

            let error = updated_namespace_result.unwrap_err();
            assert!(
                matches!(error, KvError::NamespaceAlreadyExists(ref error_message) if error_message == expected_error_message)
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_title_missing_error() -> Result<(), KvError> {
            let namespace_id = "12345";
            let expected_error_message = "request is missing a title definition";
            let update_input = KvNamespaceUpdateInput {
                account_id: "my_account_id".to_string(),
                namespace_id: namespace_id.to_string(),
                title: "".to_string(),
            };
            let mock_server = create_failing_mock_server(
                update_input.clone(),
                vec![ApiError {
                    code: 10019,
                    message: expected_error_message.to_string(),
                }],
            )
            .await;

            let kv_service = create_kv(mock_server.uri());
            let updated_namespace_result = kv_service.update_namespace(update_input).await;
            assert!(updated_namespace_result.is_err());

            let error = updated_namespace_result.unwrap_err();
            assert!(
                matches!(error, KvError::NamespaceTitleMissing(ref error_message) if error_message == expected_error_message)
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error() -> Result<(), KvError> {
            let namespace_id = "12345";
            let update_input = KvNamespaceUpdateInput {
                account_id: "my_account_id".to_string(),
                namespace_id: namespace_id.to_string(),
                title: "".to_string(),
            };
            let mock_server = create_failing_mock_server(
                update_input.clone(),
                vec![ApiError {
                    code: 10013,
                    message: "rename namespace: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let updated_namespace_result = kv.update_namespace(update_input).await;
            assert!(updated_namespace_result.is_err());

            let error = updated_namespace_result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: KvNamespaceUpdateInput,
            namespace: KvNamespace,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvNamespace> {
                    result: namespace.clone(),
                });

            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: KvNamespaceUpdateInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod delete_namespace {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvNamespaceDeleteInput};

        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_delete_namespace() -> Result<(), KvError> {
            let delete_input = KvNamespaceDeleteInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "12345".to_string(),
            };
            let mock_server = create_succeeding_mock_server(delete_input.clone()).await;

            let kv = create_kv(mock_server.uri());
            kv.delete_namespace(delete_input).await?;

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error() -> Result<(), KvError> {
            let delete_input = KvNamespaceDeleteInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "12345".to_string(),
            };
            let mock_server = create_failing_mock_server(
                delete_input.clone(),
                vec![ApiError {
                    code: 10013,
                    message: "remove namespace: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let delete_namespace_result = kv.delete_namespace(delete_input).await;

            assert!(delete_namespace_result.is_err());

            let error = delete_namespace_result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(input: KvNamespaceDeleteInput) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(200)
                    .set_body_json(ApiResponse::<Option<()>> { result: None });

            Mock::given(method("DELETE"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: KvNamespaceDeleteInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("DELETE"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}",
                    input.account_id, input.namespace_id
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    fn create_kv(host_url: String) -> Kv {
        Kv::new(
            Arc::new(Credentials::UserAuthToken {
                token: "12345".to_string(),
            }),
            Some(format!("{}/client/v4", host_url)),
            None,
        )
    }
}
