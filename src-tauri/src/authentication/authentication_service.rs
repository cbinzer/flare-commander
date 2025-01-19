use crate::authentication::authentication_models::{
    AuthenticationError, Envelope, ResponseInfo, TokenStatus, TokenVerificationResult,
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

    pub async fn verify_token(
        &self,
        token: &str,
    ) -> Result<TokenVerificationResult, AuthenticationError> {
        let verification_envelope: Envelope = self
            .http_client
            .get(format!("{}/client/v4/user/tokens/verify", self.api_url))
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        match verification_envelope.result {
            None => self.handle_verification_errors(verification_envelope.errors),
            Some(verification) => Ok(verification.status.into()),
        }
    }

    fn handle_verification_errors(
        &self,
        errors: Vec<ResponseInfo>,
    ) -> Result<TokenVerificationResult, AuthenticationError> {
        if errors.is_empty() {
            return Err(AuthenticationError::Unknown(
                "No errors in the response.".to_string(),
            ));
        }

        let error = &errors[0];
        match error.code {
            1000 => Ok(TokenVerificationResult::Invalid),
            6003 => Ok(TokenVerificationResult::Invalid),
            _ => Err(AuthenticationError::Unknown(error.message.clone())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::authentication::authentication_models::TokenVerification;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn should_verify_an_active_token_as_active() -> Result<(), AuthenticationError> {
        let mock_server = MockServer::start().await;
        let response_template = ResponseTemplate::new(200).set_body_json(Envelope {
            success: true,
            result: Some(TokenVerification {
                id: "12345".to_string(),
                status: TokenStatus::Active,
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
        let verification_result = authentication_service.verify_token("my_token").await?;

        assert_eq!(verification_result, TokenVerificationResult::Active);

        Ok(())
    }

    #[tokio::test]
    async fn should_verify_a_wrong_formated_token_as_invalid() -> Result<(), AuthenticationError> {
        let mock_server = MockServer::start().await;
        let response_template = ResponseTemplate::new(400).set_body_json(Envelope {
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
        let verification_result = authentication_service.verify_token("my_token").await?;

        assert_eq!(verification_result, TokenVerificationResult::Invalid);

        Ok(())
    }

    #[tokio::test]
    async fn should_verify_a_invalid_token_as_invalid() -> Result<(), AuthenticationError> {
        let mock_server = MockServer::start().await;
        let response_template = ResponseTemplate::new(401).set_body_json(Envelope {
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
        let verification_result = authentication_service.verify_token("my_token").await?;

        assert_eq!(verification_result, TokenVerificationResult::Invalid);

        Ok(())
    }

    #[tokio::test]
    async fn should_respond_with_an_unknown_error_if_no_errors_are_available(
    ) -> Result<(), AuthenticationError> {
        let mock_server = MockServer::start().await;
        let response_template = ResponseTemplate::new(500).set_body_json(Envelope {
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
        let response_template = ResponseTemplate::new(400).set_body_json(Envelope {
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
