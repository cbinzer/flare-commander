use chrono::{DateTime, Utc};
use cloudflare::framework::Environment;
use serde::Serializer;
use url::Url;

pub fn get_cloudflare_env(api_url: &Option<Url>) -> Environment {
    match &api_url {
        Some(api_url) => Environment::Custom(api_url.clone()),
        None => Environment::Production,
    }
}

pub fn serialize_datetime<S>(
    datetime: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match datetime {
        Some(dt) => serializer.serialize_i64(dt.timestamp()),
        None => serializer.serialize_none(),
    }
}
