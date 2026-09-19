pub struct AppState {
    pub pool: PgPool,
    pub oidc: OidcClient,
}
