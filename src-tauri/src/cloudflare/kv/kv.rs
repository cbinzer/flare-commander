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

#[cfg(test)]
mod test {
    use crate::cloudflare::common::Credentials;
    use crate::cloudflare::kv::Kv;
    use std::sync::Arc;

    mod list_namespaces {
        use crate::cloudflare::common::{OrderDirection, PageInfo};
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
            let mock_server = create_mock_server(
                &list_namespaces_input.account_id,
                Some(expected_namespaces.clone()),
                vec![],
                200,
                Some(list_namespaces_input.clone()),
            )
            .await;
            let kv = create_kv(mock_server.uri().to_string());
            let namespaces = kv.list_namespaces(list_namespaces_input).await?;

            assert_eq!(namespaces.items.len(), 3);
            assert_eq!(namespaces, expected_namespaces);

            Ok(())
        }

        async fn create_mock_server(
            account_id: &str,
            namespaces: Option<KvNamespaces>,
            errors: Vec<ApiError>,
            code: u16,
            input: Option<KvNamespacesListInput>,
        ) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template = if let Some(namespaces) = namespaces {
                ResponseTemplate::new(code).set_body_json(
                    ApiPaginatedResponse::<Vec<KvNamespace>> {
                        result: namespaces.clone().items,
                        result_info: crate::common::common_models::PageInfo {
                            total_count: namespaces.items.len(),
                            count: namespaces.items.len(),
                            page: 1,
                            per_page: 20,
                        },
                    },
                )
            } else {
                ResponseTemplate::new(code).set_body_json(ApiErrorResponse { errors })
            };

            let mut mock_builder = Mock::given(method("GET")).and(path(format!(
                "/client/v4/accounts/{account_id}/storage/kv/namespaces"
            )));

            if let Some(input) = input {
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
            }

            mock_builder
                .respond_with(response_template)
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
