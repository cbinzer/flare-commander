use crate::common::common_models::Credentials;
use crate::common::common_utils::get_cloudflare_env;
use crate::kv::kv_models::{KvError, KvKeyValue, KvKeyValueList};
use cloudflare::endpoints::workerskv::list_namespace_keys::{
    ListNamespaceKeys, ListNamespaceKeysParams,
};
use cloudflare::endpoints::workerskv::list_namespaces::{ListNamespaces, ListNamespacesParams};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::async_api::Client;
use cloudflare::framework::HttpApiClientConfig;
use serde_json::Value;
use url::Url;

pub struct KvService {
    api_url: Option<Url>,
}

impl KvService {
    pub fn new(api_url: Option<Url>) -> Self {
        Self { api_url }
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

    pub async fn get_key_values(
        &self,
        namespace: &str,
        credentials: &Credentials,
    ) -> Result<KvKeyValueList, KvError> {
        let http_client = self.create_http_client(credentials)?;
        let keys_endpoint = ListNamespaceKeys {
            account_identifier: credentials.account_id(),
            namespace_identifier: namespace,
            params: ListNamespaceKeysParams {
                limit: None,
                cursor: None,
                prefix: None,
            },
        };

        let response = http_client.request(&keys_endpoint).await?;
        let cursor = match response.result_info {
            None => None,
            Some(json) => match json.get("cursor") {
                Some(Value::String(value)) => Some(value.to_string()),
                _ => None,
            },
        };

        Ok(KvKeyValueList {
            cursor,
            data: response
                .result
                .iter()
                .map(|value| KvKeyValue {
                    key: value.name.to_string(),
                    value: value.name.to_string(),
                    expiration: value.expiration,
                })
                .collect(),
        })
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

    mod get_key_values {
        use crate::common::common_models::Credentials;
        use crate::kv::kv_models::{KvError, KvKeyValue, KvKeyValueList};
        use crate::kv::kv_service::test::create_kv_service;
        use crate::test::test_models::ApiSuccess;
        use cloudflare::endpoints::workerskv::Key;
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_get_key_values() -> Result<(), KvError> {
            let expected_key_value_list = KvKeyValueList {
                cursor: Some("12345".to_string()),
                data: vec![
                    KvKeyValue {
                        key: "key1".to_string(),
                        value: "key1".to_string(),
                        expiration: None,
                    },
                    KvKeyValue {
                        key: "key2".to_string(),
                        value: "key2".to_string(),
                        expiration: None,
                    },
                    KvKeyValue {
                        key: "key3".to_string(),
                        value: "key3".to_string(),
                        expiration: None,
                    },
                ],
            };
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";

            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Vec<Key>> {
                    result: expected_key_value_list
                        .data
                        .clone()
                        .into_iter()
                        .map(|item| Key {
                            name: item.key,
                            expiration: item.expiration,
                        })
                        .collect(),
                    errors: vec![],
                    result_info: Some(json!({
                        "cursor": "12345",
                    })),
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
            let key_value_list = kv_service.get_key_values(&namespace, &credentials).await?;

            assert_eq!(key_value_list, expected_key_value_list);

            Ok(())
        }
    }

    fn create_kv_service(mock_server_uri: String) -> KvService {
        let base_api_url = format!("{}/client/v4/", mock_server_uri);
        let api_url = Url::parse(base_api_url.as_str()).unwrap();
        KvService::new(Some(api_url))
    }
}
