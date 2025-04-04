use crate::cloudflare::read_key_value::url_encode_key;
use crate::common::common_models::Credentials;
use crate::common::common_utils::get_cloudflare_env;
use crate::kv::kv_models::{map_api_errors, GetKeysInput, KvError, KvItem, KvKey, KvKeys};
use chrono::DateTime;
use cloudflare::endpoints::workerskv::list_namespaces::{ListNamespaces, ListNamespacesParams};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::async_api::Client;

use cloudflare::framework::response::ApiSuccess;
use cloudflare::framework::HttpApiClientConfig;
use log::error;
use reqwest::multipart::Form;
use reqwest::StatusCode;
use serde_json::Value;
use url::Url;

use super::kv_models::{GetKvItemInput, WriteKvItemInput};

pub struct KvService {
    api_url: Option<Url>,
    http_client: reqwest::Client,
}

impl KvService {
    pub fn new(api_url: Option<Url>) -> Self {
        let http_client = reqwest::Client::new();

        Self {
            api_url,
            http_client,
        }
    }

    pub async fn get_namespaces(
        &self,
        credentials: &Credentials,
    ) -> Result<Vec<WorkersKvNamespace>, KvError> {
        let http_client = self.create_http_client(credentials)?;
        let list_namespaces_endpoint =
            Self::create_list_namespaces_endpoint(credentials.account_id());
        let response = http_client.request(&list_namespaces_endpoint).await;

        match response {
            Ok(api_success) => Ok(api_success.result),
            Err(api_failure) => Err(api_failure.into()),
        }
    }

    pub async fn get_kv_item<'a>(
        &self,
        credentials: &Credentials,
        input: GetKvItemInput<'a>,
    ) -> Result<KvItem, KvError> {
        let base_url: Url = (&get_cloudflare_env(&self.api_url)).into();
        let url = format!(
            "{}accounts/{}/storage/kv/namespaces/{}/values/{}",
            base_url,
            credentials.account_id(),
            input.namespace_id,
            url_encode_key(input.key),
        );

        let token = credentials.token().unwrap_or_default();
        let response = self.http_client.get(&url).bearer_auth(token).send().await?;

        match response.status() {
            StatusCode::OK => {
                let expiration = response
                    .headers()
                    .get("expiration")
                    .and_then(|header_val| header_val.to_str().ok())
                    .and_then(|str_val| str_val.parse::<i64>().ok())
                    .and_then(DateTime::from_timestamp_millis);
                let value = response.text().await?;

                Ok(KvItem {
                    key: input.key.to_string(),
                    value,
                    expiration,
                    metadata: None,
                })
            }
            _ => {
                let api_response = response.json::<ApiSuccess<()>>().await?;
                Err(map_api_errors(api_response.errors))
            }
        }
    }

    pub async fn get_keys<'a>(
        &self,
        credentials: &Credentials,
        input: GetKeysInput<'a>,
    ) -> Result<KvKeys, KvError> {
        let base_url: Url = (&get_cloudflare_env(&self.api_url)).into();
        let url = format!(
            "{}accounts/{}/storage/kv/namespaces/{}/keys",
            base_url,
            credentials.account_id(),
            input.namespace_id,
        );

        let token = credentials.token().unwrap_or_default();
        let response = self
            .http_client
            .get(&url)
            .bearer_auth(token)
            .query(&[
                ("limit", Some("10".to_string())),
                ("cursor", input.cursor),
                ("prefix", None),
            ])
            .send()
            .await?;
        let api_response = response.json::<ApiSuccess<Option<Vec<KvKey>>>>().await?;

        if !api_response.errors.is_empty() {
            return Err(map_api_errors(api_response.errors));
        }

        let cursor = match api_response.result_info {
            None => None,
            Some(json) => match json.get("cursor") {
                Some(Value::String(value)) => Some(value.to_string()),
                _ => None,
            },
        };

        Ok(KvKeys {
            cursor,
            keys: api_response.result.unwrap_or_default(),
        })
    }

    pub async fn write_kv_item<'a>(
        &self,
        credentials: &Credentials,
        input: WriteKvItemInput<'a>,
    ) -> Result<KvItem, KvError> {
        let base_url: Url = (&get_cloudflare_env(&self.api_url)).into();
        let url = format!(
            "{}accounts/{}/storage/kv/namespaces/{}/values/{}",
            base_url,
            credentials.account_id(),
            input.namespace_id,
            url_encode_key(input.key),
        );

        let token = credentials.token().unwrap_or_default();
        let expiration = input
            .expiration
            .map(|expiration_date| expiration_date.timestamp_millis().to_string());
        let request = self
            .http_client
            .put(url)
            .bearer_auth(token)
            .query(&[("expiration", expiration)]);

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
            StatusCode::OK => Ok(KvItem {
                key: input.key.to_string(),
                value,
                expiration: input.expiration,
                metadata: input.metadata,
            }),
            _ => {
                let api_response = response.json::<ApiSuccess<()>>().await?;
                let kv_error = map_api_errors(api_response.errors);
                error!("Error writing KV item: {:?}", kv_error);

                Err(kv_error)
            }
        }
    }

    fn create_http_client(&self, credentials: &Credentials) -> Result<Client, KvError> {
        Ok(Client::new(
            credentials.into(),
            HttpApiClientConfig::default(),
            get_cloudflare_env(&self.api_url),
        )?)
    }

    fn create_list_namespaces_endpoint(account_id: &str) -> ListNamespaces {
        ListNamespaces {
            account_identifier: account_id,
            params: ListNamespacesParams {
                page: None,
                per_page: Some(100),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::kv::kv_service::KvService;
    use url::Url;

    mod get_namespaces {
        use crate::authentication::authentication_models::{AuthenticationError, ResponseInfo};
        use crate::common::common_models::Credentials;
        use crate::kv::kv_models::KvError::Authentication;
        use crate::kv::kv_models::{KvError, KvNamespace, PagePaginationArray, PaginationInfo};
        use crate::kv::kv_service::test::create_kv_service;
        use cloudflare::endpoints::workerskv::WorkersKvNamespace;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_namespaces() -> Result<(), KvError> {
            let expected_namespaces = vec![
                WorkersKvNamespace {
                    id: "namespace_id_1".to_string(),
                    title: "namespace_title_1".to_string(),
                },
                WorkersKvNamespace {
                    id: "namespace_id_2".to_string(),
                    title: "namespace_title_2".to_string(),
                },
                WorkersKvNamespace {
                    id: "namespace_id_3".to_string(),
                    title: "namespace_title_3".to_string(),
                },
            ];
            let account_id = "account_id".to_string();

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

            let kv_service = create_kv_service(mock_server.uri());
            let namespaces = kv_service
                .get_namespaces(&Credentials::UserAuthToken {
                    account_id,
                    token: "token".to_string(),
                })
                .await?;

            assert_eq!(namespaces.len(), 3);
            assert_eq!(namespaces, expected_namespaces);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_an_unknown_error_occurred(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id".to_string();
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

            let kv_service = create_kv_service(mock_server.uri());
            let namespaces_result = kv_service
                .get_namespaces(&Credentials::UserAuthToken {
                    account_id,
                    token: "token".to_string(),
                })
                .await;

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
            let account_id = "account_id".to_string();
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

            let kv_service = create_kv_service(mock_server.uri());
            let namespaces_result = kv_service
                .get_namespaces(&Credentials::UserAuthToken {
                    account_id,
                    token: "token".to_string(),
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
        async fn should_respond_with_an_authentication_error_if_the_request_could_not_be_authenticated(
        ) -> Result<(), AuthenticationError> {
            let account_id = "account_id".to_string();
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

            let kv_service = create_kv_service(mock_server.uri());
            let namespaces_result = kv_service
                .get_namespaces(&Credentials::UserAuthToken {
                    account_id,
                    token: "token".to_string(),
                })
                .await;

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

    mod get_keys {
        use crate::common::common_models::Credentials;
        use crate::kv::kv_models::{GetKeysInput, KvError, KvKey};
        use crate::kv::kv_service::test::create_kv_service;
        use crate::test::test_models::ApiSuccess;
        use cloudflare::framework::response::ApiError;
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_kv_keys() -> Result<(), KvError> {
            let expected_kv_keys = vec![
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
            ];
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";

            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Vec<KvKey>> {
                    result: expected_kv_keys.clone(),
                    errors: vec![],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
                    credentials.account_id(),
                    namespace
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let get_keys_input = GetKeysInput {
                namespace_id: namespace,
                cursor: None,
            };

            let keys = kv_service.get_keys(&credentials, get_keys_input).await?;

            assert_eq!(keys.keys, expected_kv_keys);

            Ok(())
        }

        #[tokio::test]
        async fn should_get_kv_keys_after_given_cursor() -> Result<(), KvError> {
            let expected_kv_keys = vec![
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
            ];
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";
            let cursor = "my_cursor";

            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Vec<KvKey>> {
                    result: expected_kv_keys.clone(),
                    errors: vec![],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
                    credentials.account_id(),
                    namespace
                )))
                .and(query_param("cursor", cursor))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let get_keys_input = GetKeysInput {
                namespace_id: namespace,
                cursor: Some(cursor.to_string()),
            };

            let keys = kv_service.get_keys(&credentials, get_keys_input).await?;

            assert_eq!(keys.keys, expected_kv_keys);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(404).set_body_json(ApiSuccess::<Option<String>> {
                    result: None,
                    errors: vec![ApiError {
                        code: 10013,
                        message: "list keys: 'namespace not found'".to_string(),
                        other: Default::default(),
                    }],
                    result_info: None,
                });

            let account_id = "account_id".to_string();
            let namespace_id = "12345";
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces/{namespace_id}/keys"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id,
                token: "my_token".to_string(),
            };
            let result = kv_service
                .get_keys(
                    &credentials,
                    GetKeysInput {
                        namespace_id,
                        cursor: None,
                    },
                )
                .await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }
    }

    mod get_kv_item {
        use chrono::{DateTime, Utc};
        use cloudflare::framework::response::ApiError;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        use crate::common::common_models::Credentials;
        use crate::kv::kv_models::{KvError, KvItem};
        use crate::kv::kv_service::test::create_kv_service;
        use crate::kv::kv_service::GetKvItemInput;
        use crate::test::test_models::ApiSuccess;

        #[tokio::test]
        async fn should_get_kv_item() -> Result<(), KvError> {
            let expected_kv_item = KvItem {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp_millis(Utc::now().timestamp_millis()),
                metadata: None,
            };
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";

            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200)
                .set_body_string(&expected_kv_item.value)
                .append_header(
                    "expiration",
                    expected_kv_item.expiration.unwrap().timestamp_millis(),
                );
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    credentials.account_id(),
                    namespace,
                    expected_kv_item.key
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let result = kv_service
                .get_kv_item(
                    &credentials,
                    GetKvItemInput {
                        namespace_id: namespace,
                        key: &expected_kv_item.key,
                    },
                )
                .await;

            assert!(result.is_ok());
            assert_eq!(result?, expected_kv_item);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(404).set_body_json(ApiSuccess::<Option<String>> {
                    result: None,
                    errors: vec![ApiError {
                        code: 10013,
                        message: "get: 'namespace not found'".to_string(),
                        other: Default::default(),
                    }],
                    result_info: None,
                });

            let account_id = "account_id".to_string();
            let namespace_id = "12345";
            let key = "key1";
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces/{namespace_id}/values/{key}"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id,
                token: "my_token".to_string(),
            };
            let result = kv_service
                .get_kv_item(&credentials, GetKvItemInput { namespace_id, key })
                .await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_key_not_found_error_if_a_key_not_exist() -> Result<(), KvError>
        {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace_id = "my_namespace";
            let key = "key1";

            let mock_server = MockServer::start().await;
            let response_template_value =
                ResponseTemplate::new(404).set_body_json(ApiSuccess::<()> {
                    result: (),
                    errors: vec![ApiError {
                        code: 10009,
                        message: "get: 'key not found'".to_string(),
                        other: Default::default(),
                    }],
                    result_info: None,
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{namespace_id}/values/{key}",
                    credentials.account_id(),
                )))
                .respond_with(response_template_value)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let params = GetKvItemInput { namespace_id, key };

            let result = kv_service.get_kv_item(&credentials, params).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::KeyNotFound));

            Ok(())
        }
    }

    mod write_kv_item {
        use chrono::{DateTime, Utc};
        use cloudflare::framework::response::ApiError;
        use serde_json::json;
        use wiremock::{
            matchers::{method, path, query_param},
            Mock, MockServer, ResponseTemplate,
        };

        use crate::{
            common::common_models::Credentials,
            kv::kv_models::{KvError, KvItem, WriteKvItemInput},
            test::test_models::ApiSuccess,
        };

        use super::create_kv_service;

        #[tokio::test]
        async fn should_write_kv_item() -> Result<(), KvError> {
            let expected_kv_item = KvItem {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp_millis(Utc::now().timestamp_millis()),
                metadata: Some(json!({
                    "key": "value"
                })),
            };
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";

            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Option<String>> {
                    result: None,
                    errors: vec![],
                    result_info: None,
                });
            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                    credentials.account_id(),
                    namespace,
                    expected_kv_item.key,
                )))
                .and(query_param(
                    "expiration",
                    expected_kv_item
                        .expiration
                        .unwrap()
                        .timestamp_millis()
                        .to_string(),
                ))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let updated_kv_item = kv_service
                .write_kv_item(
                    &credentials,
                    WriteKvItemInput {
                        namespace_id: namespace,
                        key: &expected_kv_item.key,
                        value: Some(expected_kv_item.value.clone()),
                        expiration: expected_kv_item.expiration,
                        metadata: expected_kv_item.metadata.clone(),
                    },
                )
                .await?;

            assert_eq!(updated_kv_item, expected_kv_item);

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_namespace_not_found_error_if_a_namespace_not_exist(
        ) -> Result<(), KvError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(404).set_body_json(ApiSuccess::<Option<String>> {
                    result: None,
                    errors: vec![ApiError {
                        code: 10013,
                        message: "put: 'namespace not found'".to_string(),
                        other: Default::default(),
                    }],
                    result_info: None,
                });

            let account_id = "account_id".to_string();
            let namespace_id = "12345";
            let key = "key1";
            Mock::given(method("PUT"))
                .and(path(format!(
                    "/client/v4/accounts/{account_id}/storage/kv/namespaces/{namespace_id}/values/{key}"
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id,
                token: "my_token".to_string(),
            };
            let result = kv_service
                .write_kv_item(
                    &credentials,
                    WriteKvItemInput {
                        namespace_id,
                        key,
                        value: Some("value".to_string()),
                        expiration: None,
                        metadata: None,
                    },
                )
                .await;

            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::NamespaceNotFound));

            Ok(())
        }
    }

    fn create_kv_service(mock_server_uri: String) -> KvService {
        let base_api_url = format!("{}/client/v4/", mock_server_uri);
        let api_url = Url::parse(base_api_url.as_str()).unwrap();
        KvService::new(Some(api_url))
    }
}
