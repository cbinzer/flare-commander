use crate::authentication::authentication_models::{
    map_api_errors, Account, AccountWithCredentials, AuthenticationError, Token, TokenStatus,
};
use crate::cloudflare::account_details::AccountDetails;
use crate::common::common_models::Credentials;
use crate::common::common_utils::get_cloudflare_env;
use cloudflare::endpoints::user::GetUserTokenStatus;
use cloudflare::framework::async_api::Client;
use cloudflare::framework::HttpApiClientConfig;
use url::Url;

pub struct AuthenticationService {
    api_url: Option<Url>,
}

impl AuthenticationService {
    pub fn new(api_url: Option<Url>) -> Self {
        Self { api_url }
    }

    pub async fn verify_credentials(
        &self,
        credentials: &Credentials,
    ) -> Result<AccountWithCredentials, AuthenticationError> {
        let http_client = self.create_http_client(credentials)?;
        let verified_token = self.verify_token(&http_client).await?;

        match verified_token.status {
            TokenStatus::Active => {
                let account = self.verify_account_id(credentials, &http_client).await?;
                Ok(AccountWithCredentials {
                    id: account.id,
                    name: account.name,
                    credentials: credentials.clone().into(),
                })
            }
            TokenStatus::Disabled => Err(AuthenticationError::DisabledToken),
            TokenStatus::Expired => Err(AuthenticationError::ExpiredToken),
        }
    }

    fn create_http_client(&self, credentials: &Credentials) -> Result<Client, AuthenticationError> {
        Ok(Client::new(
            credentials.into(),
            HttpApiClientConfig::default(),
            get_cloudflare_env(&self.api_url),
        )?)
    }

    async fn verify_token(&self, http_client: &Client) -> Result<Token, AuthenticationError> {
        let verify_token_endpoint = GetUserTokenStatus {};
        let api_result = http_client.request(&verify_token_endpoint).await?;

        if api_result.errors.is_empty() {
            let token = api_result.result;
            return Ok(Token {
                id: token.id,
                status: token.status.try_into()?,
            });
        }

        Err(map_api_errors(api_result.errors))
    }

    async fn verify_account_id(
        &self,
        credentials: &Credentials,
        http_client: &Client,
    ) -> Result<Account, AuthenticationError> {
        let account_details_endpoint = AccountDetails {
            account_identifier: credentials.account_id(),
        };
        let result = http_client.request(&account_details_endpoint).await?;

        if result.errors.is_empty() {
            return Ok(Account {
                id: result.result.id,
                name: result.result.name,
            });
        }

        Err(map_api_errors(result.errors))
    }
}

#[cfg(test)]
mod test {
    use crate::authentication::authentication_service::AuthenticationService;
    use url::Url;

    mod verify_token {
        use crate::authentication::authentication_models::{
            AuthenticationError, Token, TokenStatus,
        };
        use crate::authentication::authentication_service::test::create_authentication_service;
        use crate::common::common_models::Credentials;
        use crate::test::test_models::ApiSuccess;
        use cloudflare::endpoints::user::UserTokenStatus;
        use cloudflare::framework::response::ApiError;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_verify_an_active_token_as_active() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "12345".to_string(),
                        status: "active".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let http_client = authentication_service.create_http_client(&credentials)?;
            let token = authentication_service.verify_token(&http_client).await?;

            assert_eq!(
                token,
                Token {
                    id: "12345".to_string(),
                    status: TokenStatus::Active,
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_wrong_formated_token_as_invalid() -> Result<(), AuthenticationError>
        {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(400).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "".to_string(),
                        status: "".to_string(),
                    },
                    errors: vec![ApiError {
                        code: 6003,
                        message: "Invalid request headers".to_string(),
                        other: Default::default(),
                    }],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let http_client = authentication_service.create_http_client(&credentials)?;

            let token_result = authentication_service.verify_token(&http_client).await;
            assert!(token_result.is_err());

            let error = token_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_invalid_token_as_invalid() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(401).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "".to_string(),
                        status: "".to_string(),
                    },
                    errors: vec![ApiError {
                        code: 1000,
                        message: "Invalid API token".to_string(),
                        other: Default::default(),
                    }],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let http_client = authentication_service.create_http_client(&credentials)?;

            let token_result = authentication_service.verify_token(&http_client).await;
            assert!(token_result.is_err());

            let error = token_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
        ) -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(500).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "".to_string(),
                        status: "".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let http_client = authentication_service.create_http_client(&credentials)?;

            let verification_result = authentication_service.verify_token(&http_client).await;

            assert!(verification_result.is_err());

            let error = verification_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::Unknown(_)));

            let error_message = match error {
                AuthenticationError::Unknown(message) => message,
                _ => "".to_string(),
            };
            assert_eq!(error_message, "No errors in the response.");

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_an_unknown_error_occurred(
        ) -> Result<(), AuthenticationError> {
            let unknown_error_message = "Unknown error.";
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(400).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "".to_string(),
                        status: "".to_string(),
                    },
                    errors: vec![ApiError {
                        code: 1111,
                        message: unknown_error_message.to_string(),
                        other: Default::default(),
                    }],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let http_client = authentication_service.create_http_client(&credentials)?;

            let verification_result = authentication_service.verify_token(&http_client).await;

            assert!(verification_result.is_err());

            let error = verification_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::Unknown(_)));

            let error_message = match error {
                AuthenticationError::Unknown(message) => message,
                _ => "".to_string(),
            };
            assert_eq!(error_message, unknown_error_message);

            Ok(())
        }
    }

    mod verify_account_id {
        use crate::authentication::authentication_models::{Account, AuthenticationError};
        use crate::authentication::authentication_service::test::create_authentication_service;
        use crate::common::common_models::Credentials;
        use crate::test::test_models::ApiSuccess;
        use cloudflare::framework::response::ApiError;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_verify_an_existing_account_id_as_true() -> Result<(), AuthenticationError> {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Account> {
                    result: Account {
                        id: credentials.account_id().to_string(),
                        name: "My Account".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}",
                    credentials.account_id()
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let http_client = authentication_service.create_http_client(&credentials)?;

            let account = authentication_service
                .verify_account_id(&credentials, &http_client)
                .await?;

            assert_eq!(
                account,
                Account {
                    id: credentials.account_id().to_string(),
                    name: "My Account".to_string(),
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_non_existing_account_id_as_invalid(
        ) -> Result<(), AuthenticationError> {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(404).set_body_json(ApiSuccess::<Account> {
                result: Account { id: "".to_string(), name: "".to_string() },
                errors: vec![ApiError {
                    code: 7003,
                    message: "Could not route to /client/v4/accounts/12345, perhaps your object identifier is invalid?".to_string(),
                    other: Default::default(),
                }],
            });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}",
                    credentials.account_id()
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let http_client = authentication_service.create_http_client(&credentials)?;

            let account_result = authentication_service
                .verify_account_id(&credentials, &http_client)
                .await;

            assert!(account_result.is_err());

            let error = account_result.unwrap_err();
            println!("{:?}", error);
            assert!(matches!(error, AuthenticationError::InvalidAccountId(_)));

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_an_invalid_account_id_as_invalid() -> Result<(), AuthenticationError>
        {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let mock_server = MockServer::start().await;
            let response_template =
                ResponseTemplate::new(403).set_body_json(ApiSuccess::<Account> {
                    result: Account {
                        id: "".to_string(),
                        name: "".to_string(),
                    },
                    errors: vec![ApiError {
                        code: 9109,
                        message: "Invalid account identifier".to_string(),
                        other: Default::default(),
                    }],
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}",
                    credentials.account_id()
                )))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let http_client = authentication_service.create_http_client(&credentials)?;

            let account_result = authentication_service
                .verify_account_id(&credentials, &http_client)
                .await;
            assert!(account_result.is_err());

            let error = account_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidAccountId(_)));

            Ok(())
        }
    }

    mod verify_credentials {
        use crate::authentication::authentication_models::{
            Account, AccountWithCredentials, AuthenticationError,
        };
        use crate::authentication::authentication_service::test::create_authentication_service;
        use crate::common::common_models::Credentials;
        use crate::test::test_models::ApiSuccess;
        use cloudflare::endpoints::user::UserTokenStatus;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_verify_credentials() -> Result<(), AuthenticationError> {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };

            let mock_server = MockServer::start().await;
            let response_template_account =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Account> {
                    result: Account {
                        id: credentials.account_id().to_string(),
                        name: "My Account".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}",
                    credentials.account_id()
                )))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let token = UserTokenStatus {
                id: "12345".to_string(),
                status: "active".to_string(),
            };
            let response_template_token =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: token.clone(),
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token.clone())
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let account = authentication_service
                .verify_credentials(&credentials)
                .await?;
            assert_eq!(
                account,
                AccountWithCredentials {
                    id: credentials.account_id().to_string(),
                    name: "My Account".to_string(),
                    credentials: credentials.into()
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_disabled_token_error() -> Result<(), AuthenticationError> {
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };
            let mock_server = MockServer::start().await;
            let response_template_account =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Account> {
                    result: Account {
                        id: credentials.account_id().to_string(),
                        name: "My Account".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path(format!(
                    "/client/v4/accounts/{}",
                    credentials.account_id()
                )))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let response_template_token =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "12345".to_string(),
                        status: "disabled".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let verification_result = authentication_service
                .verify_credentials(&credentials)
                .await;
            assert!(verification_result.is_err());

            let error = verification_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::DisabledToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_expired_token_error() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template_account =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<Account> {
                    result: Account {
                        id: "12345".to_string(),
                        name: "My Account".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let response_template_token =
                ResponseTemplate::new(200).set_body_json(ApiSuccess::<UserTokenStatus> {
                    result: UserTokenStatus {
                        id: "12345".to_string(),
                        status: "expired".to_string(),
                    },
                    errors: vec![],
                });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token)
                .mount(&mock_server)
                .await;

            let authentication_service = create_authentication_service(mock_server.uri());
            let credentials = Credentials::UserAuthToken {
                account_id: "my_account_id".to_string(),
                token: "my_token".to_string(),
            };

            let verification_result = authentication_service
                .verify_credentials(&credentials)
                .await;
            assert!(verification_result.is_err());

            let error = verification_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::ExpiredToken));

            Ok(())
        }
    }

    fn create_authentication_service(mock_server_url: String) -> AuthenticationService {
        let base_api_url = format!("{}/client/v4/", mock_server_url);
        let api_url = Url::parse(base_api_url.as_str()).unwrap();
        AuthenticationService::new(Some(api_url))
    }
}
