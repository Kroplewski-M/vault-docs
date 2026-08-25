use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_pool(url: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new().max_connections(10).connect(url).await
}
