use cloudflare::framework::response::ApiError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct ApiSuccess<ResultType> {
    pub result: ResultType,
    #[serde(default)]
    pub errors: Vec<ApiError>,
}
