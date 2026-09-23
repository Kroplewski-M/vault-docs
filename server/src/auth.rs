use axum::http::StatusCode;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest,
};
use serde::{Deserialize, Serialize};
use services::{session::SessionService, users::UserService};
use url::Url;
use uuid::Uuid;

use crate::config::CognitoConfig;

// --- Cognito / OpenID Connect (OIDC) login flow, at a glance ---
//
// OIDC is an identity layer on top of OAuth2: OAuth2 gets you an access token
// ("this bearer can call an API"), OIDC adds an *ID token* — a signed JWT that
// says "this specific user authenticated, and here are their claims (email,
// subject id, etc)". Cognito acts as our OIDC "provider" (the thing that
// authenticates the user and issues tokens); our server is the "relying
// party" / OIDC client (the thing that trusts the provider's tokens).
//
// The flow used here is "Authorization Code + PKCE":
//   1. begin_login  — browser is redirected to Cognito's hosted login page,
//      carrying a CSRF token, a nonce, and a PKCE challenge (all explained
//      below). Cognito shows a login form; the user isn't on our server yet.
//   2. Cognito redirects back to /auth/callback with a short-lived
//      `code` + the `state` (our csrf token) in the query string.
//   3. finish_login — we exchange that `code` for tokens directly
//      server-to-server (not via the browser), verify the returned ID
//      token, and pull the user's id/email out of its claims.
//
// See server/src/routes/auth.rs for how the login/callback/logout HTTP
// handlers wire this up with cookies.

type OidcClient = CoreClient<
    EndpointSet,      // auth url
    EndpointNotSet,   // device auth url
    EndpointNotSet,   // introspection url
    EndpointNotSet,   // revocation url
    EndpointMaybeSet, // token url
    EndpointMaybeSet, // userinfo url
>;

/// The three secrets generated in `begin_login` that we must still have
/// access to in `finish_login`, once Cognito redirects the user back to us.
/// Since our server is stateless between requests, these are round-tripped
/// in an encrypted cookie (see `FLOW_COOKIE` in routes/auth.rs) rather than
/// kept in memory.
#[derive(Serialize, Deserialize)]
pub struct LoginFlow {
    /// CSRF ("Cross-Site Request Forgery") token, sent to Cognito as the
    /// OAuth2 `state` parameter and echoed back unchanged on the callback.
    /// We check it matches (`finish_login`'s `state != flow.csrf`) to prove
    /// the callback we're handling is a response to a login *we* started,
    /// not a forged callback request an attacker tricked the victim's
    /// browser into hitting.
    csrf: String,
    /// A one-time random value embedded inside the *ID token* itself (not
    /// just the redirect, like the CSRF token above). Cognito copies it
    /// into the token it signs, and we check it matches when verifying the
    /// token's claims. This stops an attacker from taking a previously
    /// issued, still-validly-signed ID token from another login and
    /// replaying it here to impersonate that login.
    nonce: String,
    /// The PKCE ("Proof Key for Code Exchange") verifier.
    /// PKCE protects the authorization `code` from interception: the code
    /// travels through the browser redirect (visible in URLs, browser
    /// history, referrer headers), so anyone who grabs it could otherwise
    /// exchange it for tokens themselves. Instead, at login we send Cognito
    /// only a *hash* of this verifier (the "challenge"); at callback we
    /// must present the original verifier, which never touched the browser.
    /// Cognito re-hashes it and checks it matches before handing out tokens,
    /// so a stolen `code` alone is useless without this secret.
    pkce_verifier: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidState,
    Upstream(String),
    InvalidToken(String),
    Service(services::Error),
}
impl AuthError {
    pub fn status(&self) -> StatusCode {
        match self {
            AuthError::InvalidState => StatusCode::BAD_REQUEST,
            AuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            AuthError::Upstream(_) => StatusCode::BAD_GATEWAY,
            AuthError::Service(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidState => write!(f, "invalid state"),
            AuthError::Upstream(m) => write!(f, "upstream error: {m}"),
            AuthError::InvalidToken(m) => write!(f, "invalid token: {m}"),
            AuthError::Service(e) => write!(f, "{e}"),
        }
    }
}

impl From<services::Error> for AuthError {
    fn from(e: services::Error) -> Self {
        AuthError::Service(e)
    }
}

#[derive(Clone)]
pub struct AuthService {
    oidc: OidcClient,
    http: reqwest::Client,
    cognito: CognitoConfig,
    users: UserService,
    pub sessions: SessionService,
}

impl AuthService {
    pub async fn new(
        cognito: &CognitoConfig,
        users: UserService,
        sessions: SessionService,
    ) -> Self {
        let http = reqwest::ClientBuilder::new()
            // following redirects opens the client up to SSRF
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("failed to build http client");

        // The "issuer" is the OIDC provider's base URL — it's what identifies
        // *who* issued a token (checked against the `iss` claim later) and is
        // the address OIDC discovery is done against. For Cognito this is
        // always `https://cognito-idp.<region>.amazonaws.com/<user_pool_id>`.
        let issuer = IssuerUrl::new(format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            cognito.region, cognito.user_pool_id
        ))
        .expect("invalid cognito issuer url");

        // OIDC discovery: every compliant provider serves a well-known JSON
        // document (`<issuer>/.well-known/openid-configuration`) describing
        // its endpoints (authorize url, token url, its public signing keys,
        // etc). Fetching it means we don't have to hardcode Cognito's URLs
        // ourselves — only the issuer above.
        let metadata = CoreProviderMetadata::discover_async(issuer, &http)
            .await
            .expect("cognito discovery failed");

        // `client_id`/`client_secret` identify *our app* to Cognito — they
        // come from the "App client" you register in the Cognito user pool.
        // This is what makes us an OIDC "client" (relying party): Cognito
        // only issues tokens for redirect URIs/scopes that app client is
        // configured to allow.
        let oidc = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(cognito.client_id.clone()),
            Some(ClientSecret::new(cognito.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(cognito.redirect_uri.clone()).expect("invalid redirect uri"),
        );

        Self {
            oidc,
            http,
            cognito: cognito.clone(),
            users,
            sessions,
        }
    }

    /// Step 1 of login: build Cognito's hosted-login URL to redirect the
    /// browser to, plus the secrets we'll need to verify the response later.
    pub fn begin_login(&self) -> (Url, LoginFlow) {
        // `challenge` = SHA-256 hash of `verifier`, sent to Cognito now.
        // `verifier` is kept by us and only revealed in `finish_login`, see
        // the PKCE explanation on `LoginFlow::pkce_verifier` above.
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf, nonce) = self
            .oidc
            .authorize_url(
                // "Authorization Code" is the OAuth2/OIDC grant type where
                // the browser first gets a short-lived `code` (via redirect),
                // which our server then exchanges for tokens itself — as
                // opposed to e.g. the implicit flow, which puts tokens
                // straight in the browser URL. Doing the exchange
                // server-side means the client_secret and tokens never pass
                // through the browser.
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random, // becomes the OAuth2 `state` param
                Nonce::new_random,     // embedded inside the ID token itself
            )
            // "scope" tells Cognito which claims we want back in the ID
            // token/userinfo. `openid` (added automatically by this crate)
            // is what makes this OIDC rather than plain OAuth2; `email`
            // additionally asks for the user's email claim.
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(challenge)
            .url();

        let flow = LoginFlow {
            csrf: csrf.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
        };
        (url, flow)
    }

    /// Step 2 of login: called from the `/auth/callback` handler once Cognito
    /// has redirected the user back with a `code` and `state`. Verifies
    /// everything, then creates (or reuses) a local user and session.
    /// Returns the new session id.
    pub async fn finish_login(
        &self,
        flow: LoginFlow,
        code: String,
        state: String,
    ) -> Result<Uuid, AuthError> {
        // CSRF check: the `state` Cognito echoed back must match the token
        // we generated in `begin_login`. See `LoginFlow::csrf` for why.
        if state != flow.csrf {
            return Err(AuthError::InvalidState);
        }

        // The actual "code exchange": trade the one-time `code` for tokens
        // by calling Cognito's token endpoint directly (server-to-server,
        // authenticated with our client_id/client_secret). We also send back
        // the PKCE `verifier` here — Cognito hashes it and checks it matches
        // the `challenge` we sent in `begin_login` before it'll hand out
        // anything, proving this exchange request came from whoever started
        // the login, not someone who merely intercepted the `code`.
        let token = self
            .oidc
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| AuthError::Upstream(e.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
            .request_async(&self.http)
            .await
            .map_err(|e| AuthError::Upstream(e.to_string()))?;

        // The token response contains an access token (for calling APIs —
        // unused here) and, because we're doing OIDC, an ID token: a signed
        // JWT asserting who just logged in.
        let id_token = token
            .extra_fields()
            .id_token()
            .ok_or_else(|| AuthError::InvalidToken("no id_token in response".into()))?;

        // Verifying the ID token checks: its signature (using Cognito's
        // public keys from the discovery document), that `iss` matches our
        // issuer, that `aud` matches our client_id, that it hasn't expired,
        // and — passing our nonce here — that its `nonce` claim matches the
        // one we generated, so this token was issued for *this* login
        // attempt and isn't a replay of an older one.
        let verifier = self.oidc.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &Nonce::new(flow.nonce)) // signature, iss, aud, exp, nonce
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        // `subject` (the `sub` claim) is the provider's stable, unique id for
        // this user — Cognito issues these as UUIDs, so we can use it
        // directly as our own user id.
        let user_id = Uuid::parse_str(claims.subject().as_str())
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let email = claims
            .email()
            .ok_or_else(|| AuthError::InvalidToken("no email claim".into()))?;

        // First login for this Cognito user creates our local user row;
        // later logins just find the existing one.
        self.users
            .create_user_if_missing(user_id, email.as_str())
            .await?;
        // Our own session is unrelated to Cognito's tokens: from here on the
        // browser just holds our session cookie, not the ID/access tokens.
        Ok(self.sessions.create(user_id).await?)
    }

    /// Deletes the session (if any) and returns the Cognito logout URL to redirect to.
    ///
    /// This only ends *our* session; Cognito's hosted `/logout` endpoint is
    /// what actually clears the user's login state on Cognito's side (its
    /// own hosted-UI session cookie), which is why we redirect there rather
    /// than just deleting our cookie locally — otherwise the user could hit
    /// `/auth/login` again and get silently signed back in without
    /// re-entering credentials.
    pub async fn logout(&self, session_id: Option<Uuid>) -> Result<String, AuthError> {
        if let Some(id) = session_id {
            self.sessions.delete(id).await?;
        }
        let url = Url::parse_with_params(
            &format!("{}/logout", self.cognito.domain),
            [
                ("client_id", self.cognito.client_id.as_str()),
                ("logout_uri", self.cognito.logout_redirect_uri.as_str()),
            ],
        )
        .map_err(|e| AuthError::Upstream(e.to_string()))?;
        Ok(url.into())
    }
}
