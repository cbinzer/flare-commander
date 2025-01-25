use crate::app_state::AppState;
use crate::authentication::authentication_models::{Account, AuthenticationError};
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn login(
    account_id: &str,
    token: &str,
    state: State<'_, AppState>,
) -> Result<Account, CommandError> {
    Ok(state.auth_service.login(account_id, token).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandError {
    kind: CommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandErrorKind {
    Authentication,
    InvalidToken,
    DisabledToken,
    ExpiredToken,
    InvalidAccountId,
    Unknown,
}

impl From<AuthenticationError> for CommandError {
    fn from(error: AuthenticationError) -> Self {
        match error {
            AuthenticationError::InvalidAccountId(_) => CommandError {
                kind: CommandErrorKind::InvalidAccountId,
                message: "Account ID is invalid".to_string(),
            },
            AuthenticationError::ExpiredToken => CommandError {
                kind: CommandErrorKind::ExpiredToken,
                message: "Token is expired".to_string(),
            },
            AuthenticationError::DisabledToken => CommandError {
                kind: CommandErrorKind::DisabledToken,
                message: "Token is disabled".to_string(),
            },
            AuthenticationError::InvalidToken => CommandError {
                kind: CommandErrorKind::InvalidToken,
                message: "Token is invalid".to_string(),
            },
            AuthenticationError::Reqwest(_) => CommandError {
                kind: CommandErrorKind::Unknown,
                message: "An network error occurred".to_string(),
            },
            AuthenticationError::Unknown(_) => CommandError {
                kind: CommandErrorKind::Unknown,
                message: "An unknown error occurred".to_string(),
            },
        }
    }
}
