use cloudflare::framework::response::ApiError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct ApiSuccess<ResultType> {
    pub result: ResultType,
    #[serde(default)]
    pub errors: Vec<ApiError>,
    pub result_info: Option<Value>,
}
