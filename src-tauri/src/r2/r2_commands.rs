use crate::cloudflare::Cloudflare;
use crate::cloudflare::common::Credentials;
use crate::cloudflare::r2::{BucketError, Buckets, BucketsListInput, R2Error};
use log::error;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn list_buckets(
    credentials: Credentials,
    input: BucketsListInput,
) -> Result<Buckets, R2CommandError> {
    let cloudflare = Cloudflare::new(credentials, None);
    let r2 = cloudflare.r2;

    Ok(r2.list_buckets(input).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2CommandError {
    kind: R2CommandErrorKind,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum R2CommandErrorKind {
    BucketValidation,
    Authorization,
    Authentication,
    Unknown,
}

impl From<R2Error> for R2CommandError {
    fn from(value: R2Error) -> Self {
        match value {
            R2Error::Token(token_err) => {
                error!("A token error occurred on interacting with R2: {token_err}");
                R2CommandError {
                    kind: R2CommandErrorKind::Authentication,
                    message: "Authentication error".to_string(),
                }
            }
            R2Error::Bucket(bucket_err) => match bucket_err {
                BucketError::AccessDenied => {
                    error!("Access denied on accessing R2 buckets");
                    R2CommandError {
                        kind: R2CommandErrorKind::Authorization,
                        message: "Access denied on accessing R2 buckets".to_string(),
                    }
                }
                BucketError::Validation(message) => {
                    error!("Validation error occurred on R2 buckets: {message}");
                    R2CommandError {
                        kind: R2CommandErrorKind::BucketValidation,
                        message,
                    }
                }
            },
            R2Error::Reqwest(reqwest_err) => {
                error!("A reqwest error occurred on interacting with R2: {reqwest_err}");
                R2CommandError {
                    kind: R2CommandErrorKind::Unknown,
                    message: "A network error occurred".to_string(),
                }
            }
            R2Error::Unknown(unknown_err) => {
                error!("An unknown R2 error occurred: {unknown_err}");
                R2CommandError {
                    kind: R2CommandErrorKind::Unknown,
                    message: "An unknown error occurred".to_string(),
                }
            }
        }
    }
}
