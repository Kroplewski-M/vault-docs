use axum_extra::extract::cookie::Key;
use leptos::config::{LeptosOptions, get_configuration};

#[derive(Clone)]
pub struct CognitoConfig {
    pub region: String,
    pub user_pool_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub domain: String,
    pub redirect_uri: String,
    pub logout_redirect_uri: String,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub leptos_options: LeptosOptions,
    pub cookie_key: Key,
    pub secure_cookies: bool,
    pub cognito: CognitoConfig,
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} IS NOT SET IN THE ENV"))
}

impl Config {
    pub fn init() -> Self {
        let leptos_options = get_configuration(None)
            .expect("failed to load leptos configuration")
            .leptos_options;

        Self {
            database_url: env("DATABASE_URL"),
            leptos_options,
            cookie_key: Key::try_from(env("COOKIE_KEY").as_bytes())
                .expect("COOKIE_KEY must be at least 64 bytes"),
            secure_cookies: std::env::var("APP_ENV").map_or(true, |v| v != "development"),
            cognito: CognitoConfig {
                region: env("COGNITO_REGION"),
                user_pool_id: env("COGNITO_USER_POOL_ID"),
                client_id: env("AWS_COGNITO_CLIENT_ID"),
                client_secret: env("AWS_COGNITO_CLIENT_SECRET"),
                domain: env("COGNITO_DOMAIN").trim_end_matches('/').to_string(),
                redirect_uri: env("COGNITO_REDIRECT_URI"),
                logout_redirect_uri: env("COGNITO_LOGOUT_REDIRECT_URI"),
            },
        }
    }
}
