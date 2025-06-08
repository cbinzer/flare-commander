use crate::authentication::authentication_service::AuthenticationService;

pub struct AppState {
    pub auth_service: AuthenticationService,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            auth_service: AuthenticationService::new(None),
        }
    }
}
