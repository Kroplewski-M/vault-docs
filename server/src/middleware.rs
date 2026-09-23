use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::PrivateCookieJar;
use domain::models::CurrentUser;
use uuid::Uuid;

use crate::{routes::auth::SESSION_COOKIE, state::AppState};

/// Resolves the session cookie to a user and stashes it in the request
/// extensions for server fns / SSR to pick up.
pub async fn session(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    mut req: Request,
    next: Next,
) -> Response {
    let session_id = jar
        .get(SESSION_COOKIE)
        .and_then(|c| Uuid::parse_str(c.value()).ok());
    if let Some(id) = session_id
        && let Ok(Some(user_id)) = state.auth.sessions.user_id_for_session(id).await
    {
        req.extensions_mut().insert(CurrentUser { id: user_id });
    }
    let path = req.uri().path();
    if path.starts_with("/api/")
        && !path.starts_with("/api/public/")
        && req.extensions().get::<CurrentUser>().is_none()
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(req).await
}
