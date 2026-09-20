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

type OidcClient = CoreClient<
    EndpointSet,      // auth url
    EndpointNotSet,   // device auth url
    EndpointNotSet,   // introspection url
    EndpointNotSet,   // revocation url
    EndpointMaybeSet, // token url
    EndpointMaybeSet, // userinfo url
>;

/// Kept in an encrypted cookie between /auth/login and /auth/callback.
#[derive(Serialize, Deserialize)]
pub struct LoginFlow {
    csrf: String,
    nonce: String,
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
    sessions: SessionService,
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

        let issuer = IssuerUrl::new(format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            cognito.region, cognito.user_pool_id
        ))
        .expect("invalid cognito issuer url");

        let metadata = CoreProviderMetadata::discover_async(issuer, &http)
            .await
            .expect("cognito discovery failed");

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

    pub fn begin_login(&self) -> (Url, LoginFlow) {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf, nonce) = self
            .oidc
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
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

    /// Returns the new session id.
    pub async fn finish_login(
        &self,
        flow: LoginFlow,
        code: String,
        state: String,
    ) -> Result<Uuid, AuthError> {
        if state != flow.csrf {
            return Err(AuthError::InvalidState);
        }

        let token = self
            .oidc
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| AuthError::Upstream(e.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
            .request_async(&self.http)
            .await
            .map_err(|e| AuthError::Upstream(e.to_string()))?;

        let id_token = token
            .extra_fields()
            .id_token()
            .ok_or_else(|| AuthError::InvalidToken("no id_token in response".into()))?;

        let verifier = self.oidc.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &Nonce::new(flow.nonce)) // signature, iss, aud, exp, nonce
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let user_id = Uuid::parse_str(claims.subject().as_str())
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let email = claims
            .email()
            .ok_or_else(|| AuthError::InvalidToken("no email claim".into()))?;

        self.users
            .create_user_if_missing(user_id, email.as_str())
            .await?;
        Ok(self.sessions.create(user_id).await?)
    }

    /// Deletes the session (if any) and returns the Cognito logout URL to redirect to.
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
