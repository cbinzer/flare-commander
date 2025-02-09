use crate::app_state::AppState;
use crate::authentication::authentication_models::{AccountWithCredentials, AuthenticationError};
use crate::common::common_models::Credentials;
use log::error;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn login(
    credentials: Credentials,
    state: State<'_, AppState>,
) -> Result<AccountWithCredentials, AuthenticationCommandError> {
    Ok(state.auth_service.login(&credentials).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationCommandError {
    kind: AuthenticationCommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthenticationCommandErrorKind {
    Authentication,
    InvalidToken,
    DisabledToken,
    ExpiredToken,
    InvalidAccountId,
    Unknown,
}

impl From<AuthenticationError> for AuthenticationCommandError {
    fn from(error: AuthenticationError) -> Self {
        match error {
            AuthenticationError::InvalidAccountId(account_id_err) => {
                error!("Invalid account id error occurred: {}", account_id_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::InvalidAccountId,
                    message: "Account ID is invalid".to_string(),
                }
            }
            AuthenticationError::ExpiredToken => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::ExpiredToken,
                message: "Token is expired".to_string(),
            },
            AuthenticationError::DisabledToken => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::DisabledToken,
                message: "Token is disabled".to_string(),
            },
            AuthenticationError::InvalidToken => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::InvalidToken,
                message: "Token is invalid".to_string(),
            },
            AuthenticationError::Reqwest(reqwest_err) => {
                error!("A reqwest error occurred: {}", reqwest_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "An network error occurred".to_string(),
                }
            }
            AuthenticationError::Unknown(unknown_err) => {
                error!("An unknown error occurred: {}", unknown_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}
