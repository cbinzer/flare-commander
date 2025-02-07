use crate::authentication::authentication_models::{
    Account, AccountWithToken, AuthenticationError, Envelope, ResponseInfo, Token, TokenStatus,
};
use std::sync::Arc;

pub struct AuthenticationService {
    api_url: String,
    http_client: Arc<reqwest::Client>,
}

impl AuthenticationService {
    pub fn new(api_url: &str, http_client: Arc<reqwest::Client>) -> Self {
        Self {
            api_url: api_url.to_string(),
            http_client,
        }
    }

    pub async fn login(
        &self,
        account_id: &str,
        token: &str,
    ) -> Result<AccountWithToken, AuthenticationError> {
        let verified_token = self.verify_token(token).await?;
        match verified_token.status {
            TokenStatus::Active => {
                let account = self.verify_account_id(account_id, token).await?;
                // let api_token = self.get_token_by_id(&verified_token.id, token).await?;

                Ok(AccountWithToken {
                    id: account.id,
                    name: account.name,
                    token: Token {
                        value: Some(token.to_string()),
                        ..verified_token
                    },
                })
            }
            TokenStatus::Disabled => Err(AuthenticationError::DisabledToken),
            TokenStatus::Expired => Err(AuthenticationError::ExpiredToken),
        }
    }

    async fn verify_token(&self, token: &str) -> Result<Token, AuthenticationError> {
        let token_envelope: Envelope<Token> = self
            .http_client
            .get(format!("{}/client/v4/user/tokens/verify", self.api_url))
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        match token_envelope.result {
            None => Err(self.map_token_errors(token_envelope.errors)),
            Some(token) => Ok(token),
        }
    }

    // async fn get_token_by_id(
    //     &self,
    //     token_id: &str,
    //     token: &str,
    // ) -> Result<Token, AuthenticationError> {
    //     let token_envelope: Envelope<Token> = self
    //         .http_client
    //         .get(format!("{}/client/v4/user/tokens/{token_id}", self.api_url))
    //         .bearer_auth(token)
    //         .send()
    //         .await?
    //         .json()
    //         .await?;
    //
    //     println!("{:?}", &token_envelope);
    //     match token_envelope.result {
    //         None => Err(self.map_token_errors(token_envelope.errors)),
    //         Some(token) => Ok(token),
    //     }
    // }

    async fn verify_account_id(
        &self,
        account_id: &str,
        token: &str,
    ) -> Result<Account, AuthenticationError> {
        let account_envelope: Envelope<Account> = self
            .http_client
            .get(format!("{}/client/v4/accounts/{account_id}", self.api_url))
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        match account_envelope.result {
            None => Err(self.map_account_errors(account_envelope.errors)),
            Some(account) => Ok(account),
        }
    }

    fn map_token_errors(&self, errors: Vec<ResponseInfo>) -> AuthenticationError {
        if errors.is_empty() {
            return AuthenticationError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            1000 => AuthenticationError::InvalidToken,
            6003 => AuthenticationError::InvalidToken,
            _ => AuthenticationError::Unknown(error.message.clone()),
        }
    }

    fn map_account_errors(&self, errors: Vec<ResponseInfo>) -> AuthenticationError {
        if errors.is_empty() {
            return AuthenticationError::Unknown("No errors in the response.".to_string());
        }

        let error = &errors[0];
        match error.code {
            7003 => AuthenticationError::InvalidAccountId(error.message.clone()),
            9109 => AuthenticationError::InvalidAccountId(error.message.clone()),
            _ => AuthenticationError::Unknown(error.message.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    mod verify_token {
        use crate::authentication::authentication_models::{
            AuthenticationError, Envelope, ResponseInfo, Token, TokenStatus,
        };
        use crate::authentication::authentication_service::AuthenticationService;
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_verify_an_active_token_as_active() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Token {
                    id: "12345".to_string(),
                    value: None,
                    status: TokenStatus::Active,
                    policies: None,
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);
            let token = authentication_service.verify_token("my_token").await?;

            assert_eq!(
                token,
                Token {
                    id: "12345".to_string(),
                    value: None,
                    status: TokenStatus::Active,
                    policies: None,
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_wrong_formated_token_as_invalid() -> Result<(), AuthenticationError>
        {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(400).set_body_json(Envelope::<Token> {
                success: false,
                result: None,
                messages: vec![],
                errors: vec![ResponseInfo {
                    code: 6003,
                    message: "Invalid request headers".to_string(),
                }],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let token_result = authentication_service.verify_token("my_token").await;
            assert!(token_result.is_err());

            let error = token_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_invalid_token_as_invalid() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(401).set_body_json(Envelope::<Token> {
                success: false,
                result: None,
                messages: vec![],
                errors: vec![ResponseInfo {
                    code: 1000,
                    message: "Invalid API token".to_string(),
                }],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let token_result = authentication_service.verify_token("my_token").await;
            assert!(token_result.is_err());

            let error = token_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
        ) -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(500).set_body_json(Envelope::<Token> {
                success: false,
                result: None,
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);
            let verification_result = authentication_service.verify_token("my_token").await;

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
            let response_template = ResponseTemplate::new(400).set_body_json(Envelope::<Token> {
                success: false,
                result: None,
                messages: vec![],
                errors: vec![ResponseInfo {
                    code: 1111,
                    message: unknown_error_message.to_string(),
                }],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);
            let verification_result = authentication_service.verify_token("my_token").await;

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
        use crate::authentication::authentication_models::{
            Account, AuthenticationError, Envelope, ResponseInfo,
        };
        use crate::authentication::authentication_service::AuthenticationService;
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_verify_an_existing_account_id_as_true() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Account {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);
            let account = authentication_service
                .verify_account_id("12345", "my_token")
                .await?;

            assert_eq!(
                account,
                Account {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_verify_a_non_existing_account_id_as_invalid(
        ) -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(404).set_body_json(Envelope::<Account> {
                success: false,
                result: None,
                errors: vec![ResponseInfo {
                    code: 7003,
                    message: "Could not route to /client/v4/accounts/12345, perhaps your object identifier is invalid?".to_string(),
                }],
                messages: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let account_result = authentication_service
                .verify_account_id("12345", "my_token")
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
            let mock_server = MockServer::start().await;
            let response_template = ResponseTemplate::new(403).set_body_json(Envelope::<Account> {
                success: false,
                result: None,
                errors: vec![ResponseInfo {
                    code: 9109,
                    message: "Invalid account identifier".to_string(),
                }],
                messages: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let account_result = authentication_service
                .verify_account_id("12345", "my_token")
                .await;
            assert!(account_result.is_err());

            let error = account_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::InvalidAccountId(_)));

            Ok(())
        }
    }

    mod login {
        use crate::authentication::authentication_models::{
            Account, AccountWithToken, AuthenticationError, Envelope, Token, TokenStatus,
        };
        use crate::authentication::authentication_service::AuthenticationService;
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn should_login() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template_account = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Account {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let token = Token {
                id: "12345".to_string(),
                value: Some("my_token".to_string()),
                status: TokenStatus::Active,
                policies: None,
            };
            let response_template_token = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(token.clone()),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token.clone())
                .mount(&mock_server)
                .await;
            Mock::given(method("GET"))
                .and(path(
                    format!("/client/v4/user/tokens/{}", token.id).as_str(),
                ))
                .respond_with(response_template_token)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let account = authentication_service.login("12345", "my_token").await?;
            assert_eq!(
                account,
                AccountWithToken {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                    token
                }
            );

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_disabled_token_error() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template_account = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Account {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let response_template_token = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Token {
                    id: "12345".to_string(),
                    status: TokenStatus::Disabled,
                    policies: None,
                    value: None,
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let login_result = authentication_service.login("12345", "my_token").await;
            assert!(login_result.is_err());

            let error = login_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::DisabledToken));

            Ok(())
        }

        #[tokio::test]
        async fn should_respond_with_expired_token_error() -> Result<(), AuthenticationError> {
            let mock_server = MockServer::start().await;
            let response_template_account = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Account {
                    id: "12345".to_string(),
                    name: "My Account".to_string(),
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/accounts/12345"))
                .respond_with(response_template_account)
                .mount(&mock_server)
                .await;

            let response_template_token = ResponseTemplate::new(200).set_body_json(Envelope {
                success: true,
                result: Some(Token {
                    id: "12345".to_string(),
                    value: None,
                    status: TokenStatus::Expired,
                    policies: None,
                }),
                messages: vec![],
                errors: vec![],
            });
            Mock::given(method("GET"))
                .and(path("/client/v4/user/tokens/verify"))
                .respond_with(response_template_token)
                .mount(&mock_server)
                .await;

            let http_client = Arc::new(reqwest::Client::new());
            let authentication_service =
                AuthenticationService::new(mock_server.uri().as_str(), http_client);

            let login_result = authentication_service.login("12345", "my_token").await;
            assert!(login_result.is_err());

            let error = login_result.unwrap_err();
            assert!(matches!(error, AuthenticationError::ExpiredToken));

            Ok(())
        }
    }
}
