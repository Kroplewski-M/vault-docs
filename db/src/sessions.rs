use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionRepo {
    pool: PgPool,
}
impl SessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(user_id)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn user_id_for_session(&self, id: Uuid) -> sqlx::Result<Option<Uuid>> {
        sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = $1 AND expires_at > now()")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
