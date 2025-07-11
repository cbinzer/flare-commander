use super::{
    KvPair, KvPairCreateInput, KvPairGetInput, KvPairsDeleteInput, KvPairsDeleteResult,
    KvPairsGetInput, KvValues, KvValuesGetInput, KvValuesRaw, KvValuesResult,
};
use crate::cloudflare::common::{
    ApiCursorPaginatedResponse, ApiError, ApiErrorResponse, ApiPaginatedResponse, ApiResponse,
    Credentials, TokenError, API_URL,
};
use crate::cloudflare::kv::utils::url_encode_key;
use crate::cloudflare::kv::{
    KvError, KvKey, KvKeys, KvKeysListInput, KvNamespace, KvNamespaceCreateInput,
    KvNamespaceDeleteInput, KvNamespaceGetInput, KvNamespaceUpdateInput, KvNamespaces,
    KvNamespacesListInput, KvPairMetadata, KvPairMetadataGetInput, KvPairWriteInput,
};
use chrono::DateTime;
use reqwest::multipart::{Form, Part};
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::option::Option;
use std::sync::Arc;
use tokio::join;

pub struct KvClient {
    api_url: Arc<String>,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,
}

impl KvClient {
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

        let get_val_req = self
            .http_client
            .get(&url)
            .headers(self.credentials.headers());
        let get_metadata_req = self.get_kv_pair_metadata(input.clone().into());
        let (resp_result, metadata_result) = join!(get_val_req.send(), get_metadata_req);

        let response = resp_result?;
        match response.status() {
            StatusCode::OK => {
                let expiration = response
                    .headers()
                    .get("expiration")
                    .and_then(|header_val| header_val.to_str().ok())
                    .and_then(|str_val| str_val.parse::<i64>().ok())
                    .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0));
                let value = response.bytes().await?.to_vec();

                Ok(KvPair {
                    key: input.key,
                    value,
                    expiration,
                    metadata: metadata_result?,
                })
            }
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    pub async fn get_kv_pair_metadata(
        &self,
        input: KvPairMetadataGetInput,
    ) -> Result<KvPairMetadata, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
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

        self.handle_api_response::<ApiResponse<KvPairMetadata>, KvPairMetadata>(response)
            .await
    }

    pub async fn get_kv_pairs(&self, input: KvPairsGetInput) -> Result<Vec<KvPair>, KvError> {
        let kv_values_result = self.get_kv_values(input.clone().into()).await;
        match kv_values_result {
            Ok(kv_values) => match kv_values {
                KvValuesResult::Raw(_) => {
                    Err(KvError::Unknown("Cannot handle raw KV values.".to_string()))
                }
                KvValuesResult::WithMetadata(values_with_metadata) => Ok(values_with_metadata
                    .values
                    .iter()
                    .map(|(key, value)| {
                        let mut byte_value: Vec<u8> = vec![];
                        if let Some(value_str) = value.value.as_str() {
                            byte_value = value_str.as_bytes().to_vec();
                        }

                        KvPair {
                            key: key.clone(),
                            value: byte_value,
                            expiration: value.expiration,
                            metadata: value.metadata.clone(),
                        }
                    })
                    .collect()),
            },
            Err(error) => match error {
                KvError::NonTextValue => {
                    let mut kv_pairs = Vec::<KvPair>::new();
                    for key in input.keys {
                        let kv_pair_get_input = KvPairGetInput {
                            account_id: input.account_id.clone(),
                            namespace_id: input.namespace_id.clone(),
                            key: key.clone(),
                        };
                        let kv_pair = self.get_kv_pair(kv_pair_get_input).await?;
                        kv_pairs.push(kv_pair);
                    }

                    Ok(kv_pairs)
                }
                _ => Err(error),
            },
        }
    }

    pub async fn get_kv_values(&self, input: KvValuesGetInput) -> Result<KvValuesResult, KvError> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/bulk/get",
            self.api_url, input.account_id, input.namespace_id
        );

        let response = self
            .http_client
            .post(&url)
            .json(&input)
            .headers(self.credentials.headers())
            .send()
            .await?;

        if let Some(with_metadata) = input.with_metadata {
            if with_metadata {
                let values_with_metadata = self
                    .handle_api_response::<ApiResponse<KvValues>, KvValues>(response)
                    .await?;
                return Ok(KvValuesResult::WithMetadata(values_with_metadata));
            }
        }

        let values_raw = self
            .handle_api_response::<ApiResponse<KvValuesRaw>, KvValuesRaw>(response)
            .await?;
        Ok(KvValuesResult::Raw(values_raw))
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

        let value_as_bytes: Vec<u8> = Vec::from(value.clone());
        let part = Part::bytes(value_as_bytes);
        let form_data = Form::new().part("value", part).text("metadata", metadata);
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
            10029 => KvError::NonTextValue,
            10033 => KvError::InvalidExpiration,
            10147 => KvError::InvalidMetadata,
            _ => KvError::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cloudflare::common::Credentials;
    use crate::cloudflare::kv::KvClient;
    use std::sync::Arc;

    mod list_namespaces {
        use crate::cloudflare::common::{
            ApiError, ApiErrorResponse, ApiPaginatedResponse, OrderDirection, PageInfo, TokenError,
        };
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{
            KvError, KvNamespace, KvNamespaces, KvNamespacesListInput, KvNamespacesOrderBy,
        };

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

            let kv = create_kv_client(mock_server.uri().to_string());
            let namespaces = kv.list_namespaces(list_namespaces_input).await?;
            assert_eq!(namespaces.items.len(), 3);
            assert_eq!(namespaces, expected_namespaces);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
        ) -> Result<(), TokenError> {
            let account_id = "account_id".to_string();
            let mock_server = create_failing_mock_server(&account_id, vec![]).await;

            let kv = create_kv_client(mock_server.uri().to_string());
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
        ) -> Result<(), TokenError> {
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

            let kv = create_kv_client(mock_server.uri());
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
                    result_info: PageInfo {
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
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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
            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv_service = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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
        use serde_json::json;
        use std::collections::HashMap;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairGetInput, KvPairMetadata};

        #[tokio::test]
        async fn should_get_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: Vec::from("value"),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(HashMap::from([(
                    "key".to_string(),
                    json!({
                        "subkey": "subvalue"
                    }),
                )])),
            };
            let get_input = KvPairGetInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
            };

            let mock_server = create_succeeding_mock_server(&get_input, &expected_kv_pair).await;
            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(ApiResponse::<
                    KvPairMetadata,
                > {
                    result: pair.metadata.clone(),
                }))
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

    mod get_kv_pair_metadata {
        use std::collections::HashMap;

        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{KvError, KvPairMetadata, KvPairMetadataGetInput};
        use serde_json::{json, Value};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_existing_kv_pair_metadata() -> Result<(), KvError> {
            let expected_metadata = Some(HashMap::<String, Value>::from([(
                "key".to_string(),
                json!({
                    "key": {
                        "subkey": "subvalue"
                    }
                }),
            )]));
            let input = KvPairMetadataGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                key: "key1".to_string(),
            };
            let mock_server = create_succeeding_mock_server(&input, &expected_metadata).await;

            let kv_client = create_kv_client(mock_server.uri());
            let metadata = kv_client.get_kv_pair_metadata(input).await?;

            assert_eq!(metadata, expected_metadata);

            Ok(())
        }

        #[tokio::test]
        async fn should_get_undefined_kv_pair_metadata() -> Result<(), KvError> {
            let expected_metadata = None;
            let input = KvPairMetadataGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                key: "key1".to_string(),
            };
            let mock_server = create_succeeding_mock_server(&input, &expected_metadata).await;

            let kv_client = create_kv_client(mock_server.uri());
            let metadata = kv_client.get_kv_pair_metadata(input).await?;

            assert_eq!(metadata, expected_metadata);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_key_not_found_error_if_a_key_not_exist() -> Result<(), KvError>
        {
            let get_metadata_input = KvPairMetadataGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                key: "key".to_string(),
            };
            let mock_server = create_failing_mock_server(
                &get_metadata_input,
                vec![ApiError {
                    code: 10009,
                    message: "metadata: 'key not found'".to_string(),
                }],
            )
            .await;

            let kv = create_kv_client(mock_server.uri());
            let result = kv.get_kv_pair_metadata(get_metadata_input).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::KeyNotFound));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: &KvPairMetadataGetInput,
            metadata: &KvPairMetadata,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvPairMetadata> {
                    result: metadata.clone(),
                });

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: &KvPairMetadataGetInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod get_kv_pairs {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{
            KvError, KvPair, KvPairMetadata, KvPairsGetInput, KvValue, KvValues,
        };
        use chrono::{DateTime, Utc};
        use std::collections::HashMap;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_kv_pairs_with_text_values() -> Result<(), KvError> {
            let kv_values = KvValues {
                values: HashMap::from([
                    (
                        "key1".to_string(),
                        KvValue {
                            value: "string value".into(),
                            metadata: Some(HashMap::from([("key".to_string(), "value".into())])),
                            expiration: None,
                        },
                    ),
                    (
                        "key2".to_string(),
                        KvValue {
                            value: "string value".into(),
                            metadata: None,
                            expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                        },
                    ),
                ]),
            };
            let input = KvPairsGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                keys: kv_values.values.keys().cloned().collect(),
            };
            let mock_server =
                create_succeeding_mock_server_with_text_values(&input, &kv_values).await;

            let mut expected_kv_pairs = kv_values
                .values
                .iter()
                .map(|(k, v)| KvPair {
                    key: k.clone(),
                    value: v.value.clone().as_str().unwrap().as_bytes().to_vec(),
                    metadata: v.metadata.clone(),
                    expiration: v.expiration,
                })
                .collect::<Vec<KvPair>>();
            expected_kv_pairs.sort_by_key(|pair| pair.key.clone());

            let kv = create_kv_client(mock_server.uri());
            let mut kv_pairs = kv.get_kv_pairs(input).await?;
            kv_pairs.sort_by_key(|pair| pair.key.clone());

            assert_eq!(expected_kv_pairs, kv_pairs);

            Ok(())
        }

        #[tokio::test]
        async fn should_get_kv_pairs_with_binary_values() -> Result<(), KvError> {
            let expected_kv_pairs = vec![
                KvPair {
                    key: "key1".to_string(),
                    value: "value 1".as_bytes().to_vec(),
                    metadata: Some(HashMap::from([("key".to_string(), "value".into())])),
                    expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                },
                KvPair {
                    key: "key2".to_string(),
                    value: "value 2".as_bytes().to_vec(),
                    metadata: None,
                    expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                },
            ];
            let input = KvPairsGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                keys: expected_kv_pairs
                    .iter()
                    .map(|pair| pair.key.clone())
                    .collect(),
            };
            let mock_server =
                create_succeeding_mock_server_with_binary_values(&input, &expected_kv_pairs).await;

            let kv = create_kv_client(mock_server.uri());
            let kv_pairs = kv.get_kv_pairs(input).await?;

            assert_eq!(expected_kv_pairs, kv_pairs);

            Ok(())
        }

        async fn create_succeeding_mock_server_with_text_values(
            input: &KvPairsGetInput,
            values: &KvValues,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiResponse::<KvValues> {
                    result: values.clone(),
                });

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/get",
                    input.account_id, input.namespace_id,
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_succeeding_mock_server_with_binary_values(
            input: &KvPairsGetInput,
            pairs: &[KvPair],
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/get",
                    input.account_id, input.namespace_id,
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse {
                    errors: vec![ApiError {
                        code: 10029,
                        message: "bulk get keys: 'At least one of the requested keys corresponds to a non-text value'".to_string(),
                    }],
                }))
                .mount(&mock_server)
                .await;

            for pair in pairs.iter() {
                let response_template = ResponseTemplate::new(200)
                    .set_body_string(pair.value.clone())
                    .append_header("expiration", pair.expiration.unwrap().timestamp());

                Mock::given(method("GET"))
                    .and(path(format!(
                        "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                        input.account_id, input.namespace_id, pair.key
                    )))
                    .respond_with(response_template)
                    .mount(&mock_server)
                    .await;

                Mock::given(method("GET"))
                    .and(path(format!(
                        "/client/v4/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
                        input.account_id, input.namespace_id, pair.key
                    )))
                    .respond_with(ResponseTemplate::new(200).set_body_json(ApiResponse::<
                        KvPairMetadata,
                    > {
                        result: pair.metadata.clone(),
                    }))
                    .mount(&mock_server)
                    .await;
            }

            mock_server
        }
    }

    mod get_kv_values {
        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{
            KvError, KvValue, KvValues, KvValuesGetInput, KvValuesRaw, KvValuesResult,
        };
        use chrono::{DateTime, Utc};
        use serde_json::json;
        use std::collections::HashMap;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_kv_values_raw() -> Result<(), KvError> {
            let expected_values = KvValuesRaw {
                values: HashMap::from([
                    ("key1".to_string(), "string value".into()),
                    ("key2".to_string(), 1.into()),
                    ("key3".to_string(), true.into()),
                    ("key4".to_string(), json!({"subkey":"value"})),
                ]),
            };
            let input = KvValuesGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                keys: expected_values.values.keys().cloned().collect(),
                value_type: None,
                with_metadata: None,
            };
            let mock_server = create_succeeding_mock_server(
                &input,
                &KvValuesResult::Raw(expected_values.clone()),
            )
            .await;

            let kv = create_kv_client(mock_server.uri());
            let values_result = kv.get_kv_values(input).await?;

            let values_raw = if let KvValuesResult::Raw(values) = values_result {
                values
            } else {
                panic!(
                    "Expected KvValuesResult::Raw,
                            but got a different variant"
                );
            };
            assert_eq!(expected_values, values_raw);

            Ok(())
        }

        #[tokio::test]
        async fn should_get_kv_values_with_metadata() -> Result<(), KvError> {
            let expected_values = KvValues {
                values: HashMap::from([
                    (
                        "key1".to_string(),
                        KvValue {
                            value: "string value".into(),
                            metadata: Some(HashMap::from([("key".to_string(), "value".into())])),
                            expiration: None,
                        },
                    ),
                    (
                        "key2".to_string(),
                        KvValue {
                            value: "string value".into(),
                            metadata: None,
                            expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                        },
                    ),
                ]),
            };
            let input = KvValuesGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                keys: expected_values.values.keys().cloned().collect(),
                value_type: None,
                with_metadata: Some(true),
            };
            let mock_server = create_succeeding_mock_server(
                &input,
                &KvValuesResult::WithMetadata(expected_values.clone()),
            )
            .await;

            let kv = create_kv_client(mock_server.uri());
            let values_result = kv.get_kv_values(input).await?;

            let values_with_meta = if let KvValuesResult::WithMetadata(values) = values_result {
                values
            } else {
                panic!(
                    "Expected KvValuesResult::Raw,
                            but got a different variant"
                );
            };
            assert_eq!(expected_values, values_with_meta);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_a_non_text_value_error() -> Result<(), KvError> {
            let input = KvValuesGetInput {
                account_id: "account_id".to_string(),
                namespace_id: "namespace_id".to_string(),
                keys: vec!["key1".to_string()],
                value_type: None,
                with_metadata: Some(true),
            };
            let mock_server = create_failing_mock_server(
                &input,
                vec![ApiError {
                    code: 10029,
                    message: "bulk get keys: 'At least one of the requested keys corresponds to a non-text value'".to_string(),
                }],
            )
                .await;

            let kv = create_kv_client(mock_server.uri());
            let values_result = kv.get_kv_values(input).await;
            assert!(values_result.is_err());

            let error = values_result.unwrap_err();
            assert!(matches!(error, KvError::NonTextValue));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: &KvValuesGetInput,
            values: &KvValuesResult,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template = match &values {
                KvValuesResult::Raw(values_raw) => {
                    ResponseTemplate::new(200).set_body_json(ApiResponse::<KvValuesRaw> {
                        result: values_raw.clone(),
                    })
                }
                KvValuesResult::WithMetadata(values_with_meta) => ResponseTemplate::new(200)
                    .set_body_json(ApiResponse::<KvValues> {
                        result: values_with_meta.clone(),
                    }),
            };

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/get",
                    input.account_id, input.namespace_id,
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        async fn create_failing_mock_server(
            input: &KvValuesGetInput,
            errors: Vec<ApiError>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/get",
                    input.account_id, input.namespace_id,
                )))
                .respond_with(ResponseTemplate::new(400).set_body_json(ApiErrorResponse { errors }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod create_kv_pair {
        use std::collections::HashMap;

        use crate::cloudflare::common::{ApiError, ApiErrorResponse, ApiResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairCreateInput, KvPairMetadata};
        use chrono::{DateTime, Utc};
        use serde_json::Value;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_create_a_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: Vec::from("value"),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(HashMap::<String, Value>::from([(
                    "key".to_string(),
                    Value::String("value".to_string()),
                )])),
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
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

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/metadata/{}",
                    input.account_id, input.namespace_id, input.key
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(ApiResponse::<
                    KvPairMetadata,
                > {
                    result: None,
                }))
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    mod write_kv_pair {
        use std::collections::HashMap;

        use crate::cloudflare::common::{ApiError, ApiErrorResponse};
        use crate::cloudflare::kv::kv_client::test::create_kv_client;
        use crate::cloudflare::kv::{KvError, KvPair, KvPairWriteInput};
        use chrono::{DateTime, Utc};
        use serde_json::Value;
        use wiremock::{
            matchers::{method, path, query_param},
            Mock, MockServer, ResponseTemplate,
        };

        #[tokio::test]
        async fn should_write_kv_pair() -> Result<(), KvError> {
            let expected_kv_pair = KvPair {
                key: "key1".to_string(),
                value: Vec::from("value"),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(HashMap::<String, Value>::from([(
                    "key".to_string(),
                    Value::String("value".to_string()),
                )])),
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

            let kv = create_kv_client(mock_server.uri());
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

            let kv = create_kv_client(mock_server.uri());
            let result = kv.write_kv_pair(write_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_invalid_metadata_error_if_a_metadata_is_invalid(
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
                    code: 10147,
                    message: "metadata must be valid json".to_string(),
                }],
            )
            .await;

            let kv = create_kv_client(mock_server.uri());
            let result = kv.write_kv_pair(write_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::InvalidMetadata));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_invalid_expiration_error_if_a_expiration_is_invalid(
        ) -> Result<(), KvError> {
            let write_input = KvPairWriteInput {
                account_id: "my_account_id".to_string(),
                namespace_id: "my_namespace".to_string(),
                key: "key1".to_string(),
                value: None,
                expiration: Some(Utc::now()),
                expiration_ttl: None,
                metadata: None,
            };
            let mock_server = create_failing_mock_server(
                &write_input,
                vec![ApiError {
                    code: 10033,
                    message: "put: 'Invalid expiration of 1750594320. Please specify integer greater than the current number of seconds since the UNIX epoch.'".to_string(),
                }],
            )
                .await;

            let kv = create_kv_client(mock_server.uri());
            let result = kv.write_kv_pair(write_input).await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::InvalidExpiration));

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

        use crate::cloudflare::kv::kv_client::test::create_kv_client;
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
            let kv = create_kv_client(mock_server.uri());
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
            let kv = create_kv_client(mock_server.uri());

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
            let kv = create_kv_client(mock_server.uri());

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

    fn create_kv_client(host_url: String) -> KvClient {
        KvClient::new(
            Arc::new(Credentials::UserAuthToken {
                token: "12345".to_string(),
            }),
            Some(Arc::new(format!("{host_url}/client/v4"))),
            None,
        )
    }
}
