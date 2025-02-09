use cloudflare::framework::Environment;
use url::Url;

pub fn get_cloudflare_env(api_url: &Option<Url>) -> Environment {
    match &api_url {
        Some(api_url) => Environment::Custom(api_url.clone()),
        None => Environment::Production,
    }
}
