use app::{App, shell};
use axum::routing::{get, post};
use db::{sessions::SessionRepo, users::UserRepo};
use leptos::context::provide_context;
use services::{session::SessionService, users::UserService};

use crate::{
    auth::AuthService,
    config::Config,
    routes::{
        auth::{callback, login, logout},
        health::health,
    },
    state::AppState,
};

mod auth;
mod config;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    dotenvy::dotenv().ok();

    let config = Config::init();
    let leptos_options = config.leptos_options.clone();
    let addr = leptos_options.site_addr;

    let pool = db::init_pool(config.database_url.as_str())
        .await
        .expect("failed to connect to db");

    db::migrate(&pool).await.expect("failed to run migrations");

    let users = UserService::new(UserRepo::new(pool.clone()));
    let sessions = SessionService::new(SessionRepo::new(pool)); // pool stops here
    let auth = AuthService::new(&config.cognito, users, sessions.clone()).await;

    let state = AppState { config, auth };

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", post(logout))
        .leptos_routes_with_context(
            &state,
            routes,
            move || provide_context(sessions.clone()),
            move || shell(leptos_options.clone()),
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
