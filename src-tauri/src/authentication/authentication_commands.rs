use crate::cloudflare::account::AccountError;
use crate::cloudflare::common::{Credentials, TokenStatus};
use crate::cloudflare::user::UserError;
use crate::cloudflare::Cloudflare;
use log::error;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn verify_account_and_credentials(
    account_id: String,
    credentials: Credentials,
) -> Result<AccountWithCredentials, AuthenticationCommandError> {
    let cloudflare_client = Cloudflare::new(credentials.clone(), None);
    let user_client = cloudflare_client.user;
    let account_client = cloudflare_client.accounts;

    let verified_token = user_client.verify_token().await?;
    match verified_token.status {
        TokenStatus::Active => {
            let account = account_client.get_account(&account_id).await?;
            Ok(AccountWithCredentials {
                id: account.id,
                name: account.name,
                credentials,
            })
        }
        TokenStatus::Disabled => Err(AuthenticationCommandError {
            kind: AuthenticationCommandErrorKind::DisabledToken,
            message: "Token is disabled".to_string(),
        }),
        TokenStatus::Expired => Err(AuthenticationCommandError {
            kind: AuthenticationCommandErrorKind::ExpiredToken,
            message: "Token is expired".to_string(),
        }),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountWithCredentials {
    pub id: String,
    pub name: String,
    pub credentials: Credentials,
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

impl From<UserError> for AuthenticationCommandError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::Token(token_error) => {
                error!("Token error occurred: {}", token_error);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::InvalidToken,
                    message: "Token is invalid".to_string(),
                }
            }
            UserError::Reqwest(reqwest_err) => {
                error!("A reqwest error occurred: {}", reqwest_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "A network error occurred".to_string(),
                }
            }
            UserError::Unknown(unknown_err) => {
                error!("An unknown error occurred: {}", unknown_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}

impl From<AccountError> for AuthenticationCommandError {
    fn from(error: AccountError) -> Self {
        match error {
            AccountError::InvalidIdAccountId => {
                error!("Invalid account id error occurred");
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::InvalidAccountId,
                    message: "Account ID is invalid".to_string(),
                }
            }
            AccountError::Reqwest(reqwest_err) => {
                error!("A reqwest error occurred: {}", reqwest_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "A network error occurred".to_string(),
                }
            }
            AccountError::Unknown(unknown_err) => {
                error!("An unknown error occurred: {}", unknown_err);
                AuthenticationCommandError {
                    kind: AuthenticationCommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}
