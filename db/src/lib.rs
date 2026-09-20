use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod sessions;
pub mod users;

pub async fn init_pool(url: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new().max_connections(10).connect(url).await
}
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../migrations").run(pool).await
}
