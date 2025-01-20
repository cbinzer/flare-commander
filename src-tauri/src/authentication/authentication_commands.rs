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
}

impl From<AuthenticationError> for CommandError {
    fn from(error: AuthenticationError) -> Self {
        let message = match error {
            AuthenticationError::InvalidAccountId(_) => "Account ID is invalid",
            AuthenticationError::ExpiredToken => "Token is expired",
            AuthenticationError::DisabledToken => "Token is disabled",
            AuthenticationError::InvalidToken => "Token is invalid",
            AuthenticationError::Reqwest(_) => "An network error occurred",
            AuthenticationError::Unknown(_) => "An unknown error occurred",
        }
        .to_string();

        CommandError {
            kind: CommandErrorKind::Authentication,
            message,
        }
    }
}
