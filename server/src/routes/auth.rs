use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use serde::Deserialize;
use services::session::SESSION_TTL_DAYS;
use time::Duration;
use uuid::Uuid;

use crate::{auth::LoginFlow, state::AppState};

const FLOW_COOKIE: &str = "oidc_flow";
const SESSION_COOKIE: &str = "session";

pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Redirect) {
    let (url, flow) = state.auth.begin_login();
    let cookie = Cookie::build((
        FLOW_COOKIE,
        serde_json::to_string(&flow).expect("failed to serialize login flow"),
    ))
    .http_only(true)
    .same_site(SameSite::Lax) // Strict would drop it on the redirect back from Cognito
    .secure(state.config.secure_cookies)
    .path("/auth")
    .max_age(Duration::minutes(10))
    .build();
    (jar.add(cookie), Redirect::to(url.as_str()))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

pub async fn callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(params): Query<CallbackParams>,
) -> Result<(PrivateCookieJar, Redirect), StatusCode> {
    let flow: LoginFlow = jar
        .get(FLOW_COOKIE)
        .and_then(|c| serde_json::from_str(c.value()).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let session_id = state
        .auth
        .finish_login(flow, params.code, params.state)
        .await
        .map_err(|e| {
            leptos::logging::error!("login failed: {e}");
            e.status()
        })?;

    let session = Cookie::build((SESSION_COOKIE, session_id.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.config.secure_cookies)
        .path("/")
        .max_age(Duration::days(SESSION_TTL_DAYS))
        .build();

    // removal only works if the path matches the one the cookie was set with
    let jar = jar
        .remove(Cookie::build((FLOW_COOKIE, "")).path("/auth").build())
        .add(session);
    Ok((jar, Redirect::to("/vault")))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), StatusCode> {
    let session_id = jar
        .get(SESSION_COOKIE)
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let url = state.auth.logout(session_id).await.map_err(|e| {
        leptos::logging::error!("logout failed: {e}");
        e.status()
    })?;

    let jar = jar.remove(Cookie::build((SESSION_COOKIE, "")).path("/").build());
    Ok((jar, Redirect::to(&url)))
}
