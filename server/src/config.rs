use axum_extra::extract::cookie::Key;
use leptos::config::{LeptosOptions, get_configuration};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub leptos_options: LeptosOptions,
    pub cookie_key: Key,
}

impl Config {
    pub fn init() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL IS NOT SET IN THE ENV");

        let leptos_options = get_configuration(None)
            .expect("failed to load leptos configuration")
            .leptos_options;
        let cookie_key = Key::try_from(
            std::env::var("COOKIE_KEY")
                .expect("COOKIE_KEY IS NOT SET IN THE ENV")
                .as_bytes(),
        )
        .expect("COOKIE_KEY must be atleast 64 bytes");

        Self {
            database_url,
            leptos_options,
            cookie_key,
        }
    }
}
