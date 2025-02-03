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
            .get(&format!(
                "{}/client/v4/accounts/{}/storage/kv/namespaces",
                self.api_url, account_id
            ))
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        Ok(namespaces_result.result.unwrap())
    }
}

#[cfg(test)]
mod test {
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
            messages: vec![],
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
        let namespaces = kv_service.get_namespaces("account_id", "token").await?;

        assert_eq!(namespaces.len(), 3);
        assert_eq!(namespaces, expected_namespaces);

        Ok(())
    }
}
