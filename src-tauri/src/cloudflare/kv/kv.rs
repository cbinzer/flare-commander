use super::{KvPair, KvPairCreateInput, KvPairGetInput, KvPairsDeleteInput, KvPairsDeleteResult};
use crate::cloudflare::common::{
    ApiCursorPaginatedResponse, ApiError, ApiErrorResponse, ApiPaginatedResponse, ApiResponse,
    Credentials, TokenError, API_URL,
};
use crate::cloudflare::kv::utils::url_encode_key;
use crate::cloudflare::kv::{
    KvError, KvKey, KvKeys, KvKeysListInput, KvNamespace, KvNamespaceCreateInput,
    KvNamespaceDeleteInput, KvNamespaceGetInput, KvNamespaceUpdateInput, KvNamespaces,
    KvNamespacesListInput, KvPairWriteInput,
};
use chrono::DateTime;
use reqwest::multipart::Form;
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
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    pub async fn list_keys(&self, input: KvKeysListInput) -> Result<KvKeys, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/keys",
            self.api_url, input.account_id, input.namespace_id
        );

        let limit = input.limit.map_or(Some("1000".to_string()), |l| {
            if l < 10 {
                Some("10".to_string())
            } else {
                Some(l.to_string())
            }
        });
        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .query(&[
                ("limit", limit),
                ("cursor", input.cursor),
                ("prefix", input.prefix),
            ])
            .send()
            .await?;

        self.handle_api_response::<ApiCursorPaginatedResponse<Vec<KvKey>>, KvKeys>(response)
            .await
    }

    pub async fn get_kv_pair(&self, input: KvPairGetInput) -> Result<KvPair, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.api_url,
            input.account_id,
            input.namespace_id,
            url_encode_key(&input.key)
        );

        let response = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let expiration = response
                    .headers()
                    .get("expiration")
                    .and_then(|header_val| header_val.to_str().ok())
                    .and_then(|str_val| str_val.parse::<i64>().ok())
                    .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0));
                let value = response.text().await?;

                Ok(KvPair {
                    key: input.key,
                    value,
                    expiration,
                    metadata: None,
                })
            }
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    pub async fn create_kv_pair(&self, input: KvPairCreateInput) -> Result<KvPair, KvError> {
        // Check if the item already exists
        let kv_pair_result = self.get_kv_pair((&input).into()).await;
        match kv_pair_result {
            Ok(_) => Err(KvError::KeyAlreadyExists(input.key.clone())),
            Err(error) => match error {
                KvError::KeyNotFound => self.write_kv_pair(input.into()).await,
                _ => Err(error),
            },
        }
    }

    pub async fn write_kv_pair(&self, input: KvPairWriteInput) -> Result<KvPair, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.api_url,
            input.account_id,
            input.namespace_id,
            url_encode_key(&input.key)
        );

        let expiration = input
            .expiration
            .map(|expiration_date| expiration_date.timestamp().to_string());
        let expiration_ttl = input.expiration_ttl.map(|ttl| ttl.to_string());
        let request = self
            .http_client
            .put(url)
            .headers(self.credentials.headers())
            .query(&[
                ("expiration", expiration),
                ("expiration_ttl", expiration_ttl),
            ]);

        let value = input.value.unwrap_or_default();
        let mut metadata = String::from("null");
        if let Some(metadata_value) = &input.metadata {
            metadata = serde_json::to_string(&metadata_value).unwrap_or_default();
        }

        let form_data = Form::new()
            .text("value", value.clone())
            .text("metadata", metadata);
        let response = request.multipart(form_data).send().await?;

        match response.status() {
            StatusCode::OK => Ok(KvPair {
                key: input.key.to_string(),
                value,
                expiration: input.expiration,
                metadata: input.metadata,
            }),
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    pub async fn delete_kv_pairs(
        &self,
        input: KvPairsDeleteInput,
    ) -> Result<KvPairsDeleteResult, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/bulk/delete",
            self.api_url, input.account_id, input.namespace_id,
        );

        let response = self
            .http_client
            .post(url)
            .headers(self.credentials.headers())
            .json(&input.keys)
            .send()
            .await?;

        self.handle_api_response::<ApiResponse<KvPairsDeleteResult>, KvPairsDeleteResult>(response)
            .await
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
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    async fn handle_api_error_response(&self, response: Response) -> KvError {
        let api_response_result = response.json::<ApiErrorResponse>().await;
        match api_response_result {
            Ok(api_response) => self.map_api_errors(api_response.errors),
            Err(error) => error.into(),
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

    mod list_keys {
        use crate::cloudflare::common::{
            ApiCursorPaginatedResponse, ApiError, ApiErrorResponse, CursorPageInfo,
        };
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvKey, KvKeys, KvKeysListInput};
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_list_kv_keys() -> Result<(), KvError> {
            let expected_kv_keys = KvKeys {
                keys: vec![
                    KvKey {
                        name: "key1".to_string(),
                        expiration: None,
                        metadata: Some(json!({
                            "key": "value"
                        })),
                    },
                    KvKey {
                        name: "key2".to_string(),
                        expiration: None,
                        metadata: None,
                    },
                    KvKey {
                        name: "key3".to_string(),
                        expiration: None,
                        metadata: None,
                    },
                ],
                count: 3,
                cursor: None,
            };

            let list_input = KvKeysListInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                limit: None,
                cursor: None,
                prefix: None,
            };
            let mock_server =
                create_succeeding_mock_server(list_input.clone(), expected_kv_keys.clone()).await;

            let kv = create_kv(mock_server.uri());
            let kv_keys = kv.list_keys(list_input).await?;

            assert_eq!(kv_keys, expected_kv_keys);

            Ok(())
        }

        #[tokio::test]
        async fn should_list_kv_keys_after_given_cursor() -> Result<(), KvError> {
            let expected_kv_keys = KvKeys {
                keys: vec![
                    KvKey {
                        name: "key1".to_string(),
                        expiration: None,
                        metadata: None,
                    },
                    KvKey {
                        name: "key2".to_string(),
                        expiration: None,
                        metadata: None,
                    },
                    KvKey {
                        name: "key3".to_string(),
                        expiration: None,
                        metadata: None,
                    },
                ],
                count: 3,
                cursor: None,
            };

            let list_input = KvKeysListInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                limit: None,
                cursor: Some("my_cursor".to_string()),
                prefix: None,
            };
            let mock_server =
                create_succeeding_mock_server(list_input.clone(), expected_kv_keys.clone()).await;

            let kv = create_kv(mock_server.uri());
            let kv_keys = kv.list_keys(list_input).await?;

            assert_eq!(kv_keys, expected_kv_keys);

            Ok(())
        }

        #[tokio::test]
        async fn should_list_keys_with_given_prefix() -> Result<(), KvError> {
            let expected_kv_keys = KvKeys {
                keys: vec![KvKey {
                    name: "key1".to_string(),
                    expiration: None,
                    metadata: None,
                }],
                cursor: None,
                count: 1,
            };

            let list_input = KvKeysListInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                limit: None,
                cursor: None,
                prefix: Some("my_prefix".to_string()),
            };
            let mock_server =
                create_succeeding_mock_server(list_input.clone(), expected_kv_keys.clone()).await;

            let kv = create_kv(mock_server.uri());
            let kv_keys = kv.list_keys(list_input).await?;

            assert_eq!(kv_keys, expected_kv_keys);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let list_input = KvKeysListInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "12345".to_string(),
                limit: None,
                cursor: None,
                prefix: None,
            };
            let mock_server = create_failing_mock_server(
                list_input.clone(),
                vec![ApiError {
                    code: 10013,
                    message: "list keys: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let result = kv.list_keys(list_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(input: KvKeysListInput, keys: KvKeys) -> MockServer {
            let mock_server = MockServer::start().await;
            let mut mock_builder = Mock::given(method("GET")).and(path(format!(
                "/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
                input.account_id, input.namespace_id
            )));

            if let Some(limit) = input.limit {
                mock_builder = mock_builder.and(query_param("limit", limit.to_string()));
            } else {
                mock_builder = mock_builder.and(query_param("limit", "1000"));
            }

            if let Some(cursor) = input.cursor {
                mock_builder = mock_builder.and(query_param("cursor", cursor));
            }

            if let Some(prefix) = input.prefix {
                mock_builder = mock_builder.and(query_param("prefix", prefix));
            }

            let response_template = ResponseTemplate::new(200).set_body_json(
                ApiCursorPaginatedResponse::<Vec<KvKey>> {
                    result: keys.keys,
                    result_info: CursorPageInfo {
                        cursor: keys.cursor,
                        count: keys.count,
                    },
                },
            );
            mock_builder
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: KvKeysListInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
                    input.account_id, input.namespace_id
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod get_kv_pair {
        use chrono::{DateTime, Utc};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        use crate::cloudflare::common::{ApiError, ApiErrorResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairGetInput};

        #[tokio::test]
        async fn should_get_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: None,
            };
            let get_input = KvPairGetInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
            };

            let mock_server = create_succeeding_mock_server(&get_input, &expected_kv_pair).await;
            let kv = create_kv(mock_server.uri());
            let result = kv.get_kv_pair(get_input).await?;

            assert_eq!(result, expected_kv_pair);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let get_input = KvPairGetInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
            };
            let mock_server = create_failing_mock_server(
                &get_input,
                vec![ApiError {
                    code: 10013,
                    message: "get: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let result = kv.get_kv_pair(get_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_key_not_found_error_if_a_key_not_exist() -> Result<(), KvError>
        {
            let get_input = KvPairGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                key: "key".to_string(),
            };
            let mock_server = create_failing_mock_server(
                &get_input,
                vec![ApiError {
                    code: 10009,
                    message: "get: 'key not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let result = kv.get_kv_pair(get_input).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::KeyNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: &KvPairGetInput,
            pair: &KvPair,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200)
                .set_body_string(pair.value.clone())
                .append_header("expiration", pair.expiration.unwrap().timestamp());

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: &KvPairGetInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod create_kv_pair {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairCreateInput};
        use chrono::{DateTime, Utc};
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_create_a_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(json!({
                    "key": "value"
                })),
            };
            let create_input = KvPairCreateInput {
                account_id: "account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: expected_kv_pair.key.clone(),
                value: Some(expected_kv_pair.value.clone()),
                expiration: expected_kv_pair.expiration,
                expiration_ttl: None,
                metadata: expected_kv_pair.metadata.clone(),
            };
            let mock_server = create_succeeding_mock_server(&create_input, &expected_kv_pair).await;

            let kv = create_kv(mock_server.uri());
            let updated_kv_pair = kv.create_kv_pair(create_input).await?;

            assert_eq!(updated_kv_pair, expected_kv_pair);

            Ok(())
        }

        #[tokio::test]
        async fn should_result_with_a_item_already_exists_error() -> Result<(), KvError> {
            let create_input = KvPairCreateInput {
                account_id: "account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
                value: None,
                expiration: None,
                expiration_ttl: None,
                metadata: None,
            };
            let mock_server = create_failing_mock_server(&create_input).await;

            let kv = create_kv(mock_server.uri());
            let result = kv.create_kv_pair(create_input).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::KeyAlreadyExists(_)));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: &KvPairCreateInput,
            pair: &KvPair,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template_get =
                ResponseTemplate::new(404).set_body_json(ApiErrorResponse {
                    errors: vec![ApiError {
                        code: 10009,
                        message: "get: 'key not found'".to_string(),
                    }],
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(response_template_get)
                .mount(&mock_server)
                .await;

            let response_template_write = ResponseTemplate::new(200).set_body_string("");
            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key,
                )))
                .and(query_param(
                    "expiration",
                    pair.expiration.unwrap().timestamp().to_string(),
                ))
                .respond_with(response_template_write)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(input: &KvPairCreateInput) -> MockServer {
            let mock_server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(ResponseTemplate::new(200).set_body_string(""))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod write_kv_pair {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse};
        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairWriteInput};
        use chrono::{DateTime, Utc};
        use serde_json::json;
        use wiremock::{
            matchers::{method, path, query_param},
            Mock, MockServer, ResponseTemplate,
        };

        #[tokio::test]
        async fn should_write_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(json!({
                    "key": "value"
                })),
            };

            let write_input = KvPairWriteInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: expected_kv_pair.key.clone(),
                value: Some(expected_kv_pair.value.clone()),
                expiration: expected_kv_pair.expiration,
                expiration_ttl: Some(60),
                metadata: expected_kv_pair.metadata.clone(),
            };
            let mock_server = create_succeeding_mock_server(&write_input).await;

            let kv = create_kv(mock_server.uri());
            let updated_kv_pair = kv.write_kv_pair(write_input).await?;

            assert_eq!(updated_kv_pair, expected_kv_pair);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let write_input = KvPairWriteInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
                value: None,
                expiration: None,
                expiration_ttl: None,
                metadata: None,
            };
            let mock_server = create_failing_mock_server(
                &write_input,
                vec![ApiError {
                    code: 10013,
                    message: "put: 'namespace not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv(mock_server.uri());
            let result = kv.write_kv_pair(write_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(input: &KvPairWriteInput) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200).set_body_string("");
            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key,
                )))
                .and(query_param(
                    "expiration",
                    input.expiration.unwrap().timestamp().to_string(),
                ))
                .and(query_param(
                    "expiration_ttl",
                    input.expiration_ttl.unwrap().to_string(),
                ))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: &KvPairWriteInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    input.account_id, input.namespace_id, input.key,
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod delete_kv_pairs {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};

        use crate::cloudflare::kv::kv::test::create_kv;
        use crate::cloudflare::kv::{KvError, KvPairsDeleteInput, KvPairsDeleteResult};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_delete_kv_pairs() -> Result<(), KvError> {
            let delete_input = KvPairsDeleteInput {
                account_id: "account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                keys: vec!["key1".to_string(), "key2".to_string()],
            };

            let expected_result = KvPairsDeleteResult {
                successful_key_count: delete_input.keys.len() as u32,
                unsuccessful_keys: vec![],
            };
            let mock_server =
                create_succeeding_mock_server(&delete_input, &expected_result.clone()).await;
            let kv = create_kv(mock_server.uri());
            let delete_result = kv.delete_kv_pairs(delete_input).await?;

            assert_eq!(expected_result, delete_result);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_unsuccessful_deleted_keys_if_it_is_not_possible_to_delete_some(
        ) -> Result<(), KvError> {
            let delete_input = KvPairsDeleteInput {
                account_id: "account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                keys: vec!["key1".to_string(), "key2".to_string()],
            };
            let expected_result = KvPairsDeleteResult {
                successful_key_count: 1,
                unsuccessful_keys: vec![delete_input.keys[1].clone()],
            };
            let mock_server =
                create_succeeding_mock_server(&delete_input, &expected_result.clone()).await;
            let kv = create_kv(mock_server.uri());

            let deletion_result = kv.delete_kv_pairs(delete_input).await?;

            assert_eq!(expected_result, deletion_result);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let delete_input = KvPairsDeleteInput {
                account_id: "account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                keys: vec!["key1".to_string(), "key2".to_string()],
            };
            let mock_server = create_failing_mock_server(
                &delete_input,
                vec![ApiError {
                    code: 10013,
                    message: "bulk remove keys: 'namespace not found'".to_string(),
                }],
            )
            .await;
            let kv = create_kv(mock_server.uri());

            let delete_result = kv.delete_kv_pairs(delete_input).await;
            assert!(delete_result.is_err());

            let error = delete_result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: &KvPairsDeleteInput,
            result: &KvPairsDeleteResult,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template_value =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvPairsDeleteResult> {
                    result: result.clone(),
                });

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/delete",
                    input.account_id, input.namespace_id
                )))
                .respond_with(response_template_value)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: &KvPairsDeleteInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template_value =
                ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors });

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/delete",
                    input.account_id, input.namespace_id
                )))
                .respond_with(response_template_value)
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
