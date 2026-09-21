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

// The full flow, tying `AuthService` (server/src/auth.rs) to HTTP:
//   GET /auth/login    -> login()    -> redirect to Cognito's hosted UI
//   GET /auth/callback -> callback() -> Cognito redirects back here
//   POST /auth/logout  -> logout()   -> redirect to Cognito's hosted logout
//
// `PrivateCookieJar` encrypts+signs cookies (using AppState's cookie `Key`),
// so the CSRF token/nonce/PKCE verifier in `oidc_flow` can't be read or
// tampered with by the browser or a man-in-the-middle between login and
// callback.

/// Starts a login: generates the OIDC flow secrets (see `AuthService::begin_login`
/// for what csrf/nonce/PKCE mean), stashes them in a short-lived cookie so
/// `callback` can verify them, and sends the browser to Cognito's hosted
/// login page.
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

/// The query params Cognito appends to the redirect back to us:
/// `code` is the one-time authorization code to exchange for tokens;
/// `state` is our CSRF token, echoed back unchanged.
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
    // Recover the flow secrets we stashed in `login`. If this cookie is
    // missing/invalid we have no way to verify the callback at all (no csrf
    // token to compare, no PKCE verifier, no nonce) — e.g. it expired, was
    // never set, or someone hit this endpoint directly without going
    // through /auth/login.
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
