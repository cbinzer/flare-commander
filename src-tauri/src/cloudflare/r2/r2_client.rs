use crate::cloudflare::common::{
    API_URL, ApiCursorPaginatedResponse, ApiError, ApiErrorResponse, Credentials, TokenError,
};
use crate::cloudflare::r2::{BucketError, Buckets, BucketsListInput, BucketsListResponse, R2Error};
use reqwest::header::HeaderValue;
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub struct R2Client {
    api_url: Arc<String>,
    credentials: Arc<Credentials>,
    http_client: Arc<reqwest::Client>,
}

impl R2Client {
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

    pub async fn list_buckets(&self, input: BucketsListInput) -> Result<Buckets, R2Error> {
        let url = format!("{}/accounts/{}/r2/buckets", self.api_url, input.account_id,);

        let mut headers = self.credentials.headers();
        if let Some(jurisdiction) = &input.jurisdiction {
            headers.insert(
                "cf-r2-jurisdiction",
                HeaderValue::from_str(&jurisdiction.to_string()).expect("Invalid jurisdiction"),
            );
        }
        let query_parameters: HashMap<String, String> = input.into();

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .query(&query_parameters)
            .send()
            .await?;

        self.handle_api_response::<ApiCursorPaginatedResponse<BucketsListResponse>, Buckets>(
            response,
        )
        .await
    }

    async fn handle_api_response<T: for<'a> Deserialize<'a>, R: From<T>>(
        &self,
        response: Response,
    ) -> Result<R, R2Error> {
        match response.status() {
            StatusCode::OK => {
                let api_result: T = response.json().await?;
                Ok(api_result.into())
            }
            _ => Err(self.handle_api_error_response(response).await),
        }
    }

    async fn handle_api_error_response(&self, response: Response) -> R2Error {
        let api_response_result = response.json::<ApiErrorResponse>().await;
        match api_response_result {
            Ok(api_response) => Self::map_api_errors(api_response.errors),
            Err(error) => error.into(),
        }
    }

    fn map_api_errors(errors: Vec<ApiError>) -> R2Error {
        if errors.is_empty() {
            return R2Error::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            10000 | 10001 => R2Error::Token(TokenError::Invalid),
            10003 => R2Error::Bucket(BucketError::AccessDenied),
            10023 | 10029 => R2Error::Bucket(BucketError::Validation(error.message.clone())),
            _ => R2Error::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cloudflare::common::Credentials;
    use crate::cloudflare::r2::R2Client;
    use std::sync::Arc;

    mod list_buckets {
        use crate::cloudflare::common::{
            ApiCursorPaginatedResponse, ApiError, ApiErrorResponse, CursorPageInfo, OrderDirection,
        };
        use crate::cloudflare::r2::r2_client::test::create_r2_client;
        use crate::cloudflare::r2::{
            Bucket, BucketError, BucketJurisdiction, BucketLocation, BucketOrder,
            BucketStorageClass, Buckets, BucketsListInput, BucketsListResponse, R2Error,
        };
        use rstest::*;
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_list_all_buckets() -> Result<(), R2Error> {
            let expected_buckets = Buckets {
                items: vec![
                    Bucket {
                        name: "bucket1".to_string(),
                        creation_date: chrono::Utc::now(),
                        jurisdiction: None,
                        location: None,
                        storage_class: None,
                    },
                    Bucket {
                        name: "bucket2".to_string(),
                        creation_date: chrono::Utc::now(),
                        jurisdiction: Some(BucketJurisdiction::Eu),
                        location: Some(BucketLocation::WEur),
                        storage_class: Some(BucketStorageClass::Standard),
                    },
                ],
                page_info: Some(CursorPageInfo {
                    cursor: Some("my_cursor".to_string()),
                    count: None,
                    per_page: Some(2),
                }),
            };
            let buckets_list_input = BucketsListInput {
                account_id: "test_account".to_string(),
                cursor: Some("cursor".to_string()),
                direction: Some(OrderDirection::Asc),
                name_contains: Some("bucket".to_string()),
                order: Some(BucketOrder::Name),
                per_page: Some(2),
                start_after: Some("after_id".to_string()),
                jurisdiction: Some(BucketJurisdiction::Eu),
            };

            let mock_server =
                create_succeeding_mock_server(buckets_list_input.clone(), expected_buckets.clone())
                    .await;
            let r2_client = create_r2_client(mock_server.uri());

            let buckets = r2_client.list_buckets(buckets_list_input).await?;

            assert_eq!(buckets, expected_buckets);
            Ok(())
        }

        #[tokio::test]
        #[rstest]
        #[case(10023, "continuation token is not valid".to_string())]
        #[case(10029, "per_page must be between 1 and 1000000".to_string())]
        async fn should_handle_validation_errors(
            #[case] code: u16,
            #[case] message: String,
        ) -> Result<(), R2Error> {
            let input = BucketsListInput {
                account_id: "account_id".to_string(),
                ..BucketsListInput::default()
            };

            let api_error = ApiError {
                code,
                message: message.clone(),
            };

            let mock_server = create_failing_mock_server(&input.account_id, api_error).await;
            let r2_client = create_r2_client(mock_server.uri());
            let result = r2_client.list_buckets(input).await;
            assert!(result.is_err());

            let error = result.err().unwrap();
            assert!(matches!(
                error,
                R2Error::Bucket(BucketError::Validation(msg)) if msg == message
            ));

            Ok(())
        }

        #[tokio::test]
        async fn should_handle_access_denied_error() -> Result<(), R2Error> {
            let buckets_list_input = BucketsListInput {
                account_id: "account_id".to_string(),
                ..BucketsListInput::default()
            };
            let api_error = ApiError {
                code: 10003,
                message: "Access Denied".to_string(),
            };

            let mock_server =
                create_failing_mock_server(&buckets_list_input.account_id, api_error).await;
            let r2_client = create_r2_client(mock_server.uri());

            let result = r2_client.list_buckets(buckets_list_input).await;
            assert!(result.is_err());

            let error = result.err().unwrap();
            assert!(matches!(error, R2Error::Bucket(BucketError::AccessDenied)));

            Ok(())
        }

        async fn create_succeeding_mock_server(
            input: BucketsListInput,
            buckets: Buckets,
        ) -> MockServer {
            let mock_server = MockServer::start().await;

            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiCursorPaginatedResponse::<
                    BucketsListResponse,
                > {
                    result: BucketsListResponse {
                        buckets: buckets.items,
                    },
                    result_info: buckets.page_info,
                });

            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}/r2/buckets",
                    input.account_id
                )))
                .and(query_param("cursor", input.cursor.unwrap_or_default()))
                .and(query_param(
                    "direction",
                    input.direction.unwrap().to_string(),
                ))
                .and(query_param(
                    "name_contains",
                    input.name_contains.unwrap_or_default(),
                ))
                .and(query_param("order", input.order.unwrap().to_string()))
                .and(query_param("per_page", input.per_page.unwrap().to_string()))
                .and(query_param(
                    "start_after",
                    input.start_after.unwrap_or_default(),
                ))
                .and(header(
                    "cf-r2-jurisdiction",
                    input.jurisdiction.unwrap().to_string(),
                ))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }

        pub async fn create_failing_mock_server(account_id: &str, error: ApiError) -> MockServer {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(400).set_body_json(ApiErrorResponse {
                errors: vec![error],
            });

            Mock::given(method("GET"))
                .and(path(format!("/client/v4/accounts/{account_id}/r2/buckets")))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            mock_server
        }
    }

    fn create_r2_client(host_url: String) -> R2Client {
        R2Client::new(
            Arc::new(Credentials::UserAuthToken {
                token: "12345".to_string(),
            }),
            Some(Arc::new(format!("{host_url}/client/v4"))),
            None,
        )
    }
}
