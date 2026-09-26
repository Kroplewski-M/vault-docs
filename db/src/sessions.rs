use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionRepo {
    pool: PgPool,
}

#[async_trait]
pub trait SessionRepoTrait: Send + Sync {
    async fn user_id_for_session(&self, id: &Uuid) -> sqlx::Result<Option<Uuid>>;
    async fn delete(&self, id: &Uuid) -> sqlx::Result<()>;
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
}
#[async_trait]
impl SessionRepoTrait for SessionRepo {
    async fn user_id_for_session(&self, id: &Uuid) -> sqlx::Result<Option<Uuid>> {
        sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = $1 AND expires_at > now()")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
    async fn delete(&self, id: &Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::users::{UserRepo, UserRepoTrait};

    // sessions.user_id references users(id), so every test needs a user first
    async fn user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        UserRepo::new(pool.clone())
            .create_user_if_missing(id, &format!("{id}@test.local"))
            .await
            .unwrap();
        id
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn live_session_resolves_to_user(pool: PgPool) {
        let repo = SessionRepo::new(pool.clone());
        let user_id = user(&pool).await;
        let id = Uuid::new_v4();
        repo.create(id, user_id, Utc::now() + Duration::hours(1))
            .await
            .unwrap();

        assert_eq!(repo.user_id_for_session(&id).await.unwrap(), Some(user_id));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn expired_session_resolves_to_none(pool: PgPool) {
        let repo = SessionRepo::new(pool.clone());
        let user_id = user(&pool).await;
        let id = Uuid::new_v4();
        repo.create(id, user_id, Utc::now() - Duration::seconds(1))
            .await
            .unwrap();

        assert_eq!(repo.user_id_for_session(&id).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn unknown_session_resolves_to_none(pool: PgPool) {
        let repo = SessionRepo::new(pool);
        assert_eq!(
            repo.user_id_for_session(&Uuid::new_v4()).await.unwrap(),
            None
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn deleted_session_resolves_to_none(pool: PgPool) {
        let repo = SessionRepo::new(pool.clone());
        let user_id = user(&pool).await;
        let id = Uuid::new_v4();
        repo.create(id, user_id, Utc::now() + Duration::hours(1))
            .await
            .unwrap();
        repo.delete(&id).await.unwrap();

        assert_eq!(repo.user_id_for_session(&id).await.unwrap(), None);
    }
}
