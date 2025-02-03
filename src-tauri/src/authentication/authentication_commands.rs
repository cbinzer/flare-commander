use crate::app_state::AppState;
use crate::authentication::authentication_models::{AccountWithToken, AuthenticationError};
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn login(
    account_id: &str,
    token: &str,
    state: State<'_, AppState>,
) -> Result<AccountWithToken, AuthenticationCommandError> {
    Ok(state.auth_service.login(account_id, token).await?)
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
            AuthenticationError::InvalidAccountId(_) => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::InvalidAccountId,
                message: "Account ID is invalid".to_string(),
            },
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
            AuthenticationError::Reqwest(_) => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::Unknown,
                message: "An network error occurred".to_string(),
            },
            AuthenticationError::Unknown(_) => AuthenticationCommandError {
                kind: AuthenticationCommandErrorKind::Unknown,
                message: "An unknown error occurred".to_string(),
            },
        }
    }
}
