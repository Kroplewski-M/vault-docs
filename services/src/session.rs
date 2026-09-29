use chrono::{Duration, Utc};
use db::sessions::{SessionRepo, SessionRepoTrait};
use uuid::Uuid;

use crate::{Result, error::internal};

pub const SESSION_TTL_DAYS: i64 = 7;

#[derive(Clone)]
pub struct SessionService {
    repo: SessionRepo,
}

impl SessionService {
    pub fn new(repo: SessionRepo) -> Self {
        Self { repo }
    }
    pub async fn create(&self, user_id: Uuid) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(SESSION_TTL_DAYS);
        self.repo
            .create(id, user_id, expires_at)
            .await
            .map_err(internal)?;
        Ok(id)
    }
    pub async fn user_id_for_session(&self, session_id: Uuid) -> Result<Option<Uuid>> {
        self.repo
            .user_id_for_session(&session_id)
            .await
            .map_err(internal)
    }
    pub async fn delete(&self, session_id: Uuid) -> Result<()> {
        self.repo.delete(&session_id).await.map_err(internal)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use chrono::DateTime;
    use db::users::{UserRepo, UserRepoTrait};
    use sqlx::PgPool;

    fn service(pool: PgPool) -> SessionService {
        SessionService::new(SessionRepo::new(pool))
    }

    // sessions.user_id references users(id), so every session needs a user first
    async fn user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        UserRepo::new(pool.clone())
            .create_user_if_missing(id, &format!("{id}@test.local"))
            .await
            .unwrap();
        id
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn created_session_resolves_to_user(pool: PgPool) {
        let svc = service(pool.clone());
        let user_id = user(&pool).await;

        let id = svc.create(user_id).await.unwrap();

        assert_eq!(svc.user_id_for_session(id).await.unwrap(), Some(user_id));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn create_sets_expiry_to_ttl(pool: PgPool) {
        let svc = service(pool.clone());
        let user_id = user(&pool).await;

        let before = Utc::now();
        let id = svc.create(user_id).await.unwrap();
        let after = Utc::now();

        let expires_at: DateTime<Utc> =
            sqlx::query_scalar("SELECT expires_at FROM sessions WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();

        // 1s slack: Postgres stores microseconds, chrono has nanoseconds
        let ttl = Duration::days(SESSION_TTL_DAYS);
        let slack = Duration::seconds(1);
        assert!(
            expires_at >= before + ttl - slack && expires_at <= after + ttl + slack,
            "expires_at {expires_at} not ~{SESSION_TTL_DAYS} days out"
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn each_create_returns_new_id(pool: PgPool) {
        let svc = service(pool.clone());
        let user_id = user(&pool).await;

        let a = svc.create(user_id).await.unwrap();
        let b = svc.create(user_id).await.unwrap();

        assert_ne!(a, b);
        assert_eq!(svc.user_id_for_session(a).await.unwrap(), Some(user_id));
        assert_eq!(svc.user_id_for_session(b).await.unwrap(), Some(user_id));
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn unknown_session_resolves_to_none(pool: PgPool) {
        let svc = service(pool);

        assert_eq!(svc.user_id_for_session(Uuid::new_v4()).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn deleted_session_resolves_to_none(pool: PgPool) {
        let svc = service(pool.clone());
        let user_id = user(&pool).await;
        let id = svc.create(user_id).await.unwrap();

        svc.delete(id).await.unwrap();

        assert_eq!(svc.user_id_for_session(id).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn db_error_is_mapped_to_internal(pool: PgPool) {
        let svc = service(pool);

        // no such user, so the sessions.user_id foreign key rejects the insert
        let err = svc.create(Uuid::new_v4()).await.unwrap_err();

        assert!(matches!(err, Error::Internal(_)), "got {err:?}");
    }
}
