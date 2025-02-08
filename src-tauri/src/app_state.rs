use crate::authentication::authentication_service::AuthenticationService;
use crate::kv::kv_service::KvService;

pub struct AppState {
    pub auth_service: AuthenticationService,
    pub kv_service: KvService,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            auth_service: AuthenticationService::new(None),
            kv_service: KvService::new(None),
        }
    }
}
