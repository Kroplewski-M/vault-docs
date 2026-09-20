use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use leptos::config::LeptosOptions;

use crate::{auth::AuthService, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub auth: AuthService,
}
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.config.leptos_options.clone()
    }
}
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.config.cookie_key.clone()
    }
}
