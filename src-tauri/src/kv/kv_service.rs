use crate::cloudflare::read_key_value::url_encode_key;
use crate::common::common_models::Credentials;
use crate::common::common_utils::get_cloudflare_env;
use crate::kv::kv_models::{
    map_api_errors, KvError, KvItem, KvItemsDeletionInput, KvItemsDeletionResult,
    KvKeyPairCreateInput,
};
use chrono::DateTime;

use cloudflare::framework::response::ApiSuccess;
use log::error;
use reqwest::multipart::Form;
use reqwest::StatusCode;
use url::Url;

use super::kv_models::{GetKvItemInput, KvKeyPairUpsertInput};

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
                    .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0));
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

    pub async fn create_kv_item(
        &self,
        credentials: &Credentials,
        input: &KvKeyPairCreateInput,
    ) -> Result<KvItem, KvError> {
        // Check if the item already exists
        let kv_item_result = self.get_kv_item(credentials, input.into()).await;
        match kv_item_result {
            Ok(_) => Err(KvError::KeyAlreadyExists(input.key.clone())),
            Err(error) => match error {
                KvError::KeyNotFound => self.write_kv_item(credentials, input.into()).await,
                _ => Err(error),
            },
        }
    }

    pub async fn write_kv_item(
        &self,
        credentials: &Credentials,
        input: KvKeyPairUpsertInput,
    ) -> Result<KvItem, KvError> {
        let base_url: Url = (&get_cloudflare_env(&self.api_url)).into();
        let url = format!(
            "{}accounts/{}/storage/kv/namespaces/{}/values/{}",
            base_url,
            credentials.account_id(),
            input.namespace_id,
            url_encode_key(input.key.as_str()),
        );

        let token = credentials.token().unwrap_or_default();
        let expiration = input
            .expiration
            .map(|expiration_date| expiration_date.timestamp().to_string());
        let expiration_ttl = input.expiration_ttl.map(|ttl| ttl.to_string());
        let request = self.http_client.put(url).bearer_auth(token).query(&[
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
}

#[cfg(test)]
mod test {
    use crate::kv::kv_service::KvService;
    use url::Url;

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
            kv::kv_models::{KvError, KvItem, KvKeyPairUpsertInput},
            test::test_models::ApiSuccess,
        };

        use super::create_kv_service;

        #[tokio::test]
        async fn should_write_kv_item() -> Result<(), KvError> {
            let expected_kv_item = KvItem {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
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
                    expected_kv_item.expiration.unwrap().timestamp().to_string(),
                ))
                .and(query_param("expiration_ttl", "60"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let updated_kv_item = kv_service
                .write_kv_item(
                    &credentials,
                    KvKeyPairUpsertInput {
                        namespace_id: namespace.to_string(),
                        key: expected_kv_item.key.clone(),
                        value: Some(expected_kv_item.value.clone()),
                        expiration: expected_kv_item.expiration,
                        expiration_ttl: Some(60),
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
                    KvKeyPairUpsertInput {
                        namespace_id: namespace_id.to_string(),
                        key: key.to_string(),
                        value: Some("value".to_string()),
                        expiration: None,
                        expiration_ttl: None,
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

    mod create_kv_item {
        use crate::common::common_models::Credentials;
        use crate::kv::kv_models::{KvError, KvItem, KvKeyPairCreateInput};
        use crate::kv::kv_service::test::create_kv_service;
        use crate::test::test_models::ApiSuccess;
        use chrono::{DateTime, Utc};
        use cloudflare::framework::response::ApiError;
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_create_a_kv_item() -> Result<(), KvError> {
            let expected_kv_item = KvItem {
                key: "key1".to_string(),
                value: "value".to_string(),
                expiration: DateTime::from_timestamp(Utc::now().timestamp(), 0),
                metadata: Some(json!({
                    "key": "value"
                })),
            };
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace = "my_namespace";
            let key = expected_kv_item.key.clone();
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
                    "/client/v4/accounts/{}/storage/kv/namespaces/{namespace}/values/{key}",
                    credentials.account_id(),
                )))
                .respond_with(response_template_value)
                .mount(&mock_server)
                .await;

            let response_template_write =
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
                    expected_kv_item.expiration.unwrap().timestamp().to_string(),
                ))
                .respond_with(response_template_write)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let updated_kv_item = kv_service
                .create_kv_item(
                    &credentials,
                    &KvKeyPairCreateInput {
                        namespace_id: namespace.to_string(),
                        key: expected_kv_item.key.clone(),
                        value: Some(expected_kv_item.value.clone()),
                        expiration: expected_kv_item.expiration,
                        expiration_ttl: None,
                        metadata: expected_kv_item.metadata.clone(),
                    },
                )
                .await?;

            assert_eq!(updated_kv_item, expected_kv_item);

            Ok(())
        }

        #[tokio::test]
        async fn should_result_with_a_item_already_exists_error() -> Result<(), KvError> {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let namespace_id = "my_namespace";
            let key = "key1";

            let mock_server = MockServer::start().await;
            let response_template_value = ResponseTemplate::new(200).set_body_string("value");
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/storage/kv/namespaces/{namespace_id}/values/{key}",
                    credentials.account_id(),
                )))
                .respond_with(response_template_value)
                .mount(&mock_server)
                .await;

            let kv_service = create_kv_service(mock_server.uri());
            let input = KvKeyPairCreateInput {
                namespace_id: namespace_id.to_string(),
                key: key.to_string(),
                value: None,
                expiration: None,
                expiration_ttl: None,
                metadata: None,
            };

            let result = kv_service.create_kv_item(&credentials, &input).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(matches!(error, KvError::KeyAlreadyExists(_)));

            Ok(())
        }
    }

    fn create_kv_service(mock_server_uri: String) -> KvService {
        let base_api_url = format!("{}/client/v4/", mock_server_uri);
        let api_url = Url::parse(base_api_url.as_str()).unwrap();
        KvService::new(Some(api_url))
    }
}
