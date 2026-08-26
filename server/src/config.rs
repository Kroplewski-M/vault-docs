use leptos::config::{LeptosOptions, get_configuration};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub leptos_options: LeptosOptions,
}

impl Config {
    pub fn init() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL IS NOT SET IN THE ENV");

        let leptos_options = get_configuration(None)
            .expect("failed to load leptos configuration")
            .leptos_options;
        Self {
            database_url,
            leptos_options,
        }
    }
}
