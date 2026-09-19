use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn upsert(&self, id: Uuid, email: impl Into<String>) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
            .bind(id)
            .bind(email.into())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
