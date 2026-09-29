use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepo {
    pool: PgPool,
}

#[async_trait]
pub trait UserRepoTrait: Send + Sync {
    async fn create_user_if_missing(&self, id: Uuid, email: &str) -> sqlx::Result<()>;
    async fn get_user_email(&self, id: Uuid) -> sqlx::Result<Option<String>>;
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepoTrait for UserRepo {
    async fn create_user_if_missing(&self, id: Uuid, email: &str) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO users (id, email) VALUES ($1, $2) ON CONFLICT (id) 
                    DO UPDATE SET email = EXCLUDED.email, updated_at = now()",
        )
        .bind(id)
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn get_user_email(&self, id: Uuid) -> sqlx::Result<Option<String>> {
        sqlx::query_scalar("SELECT email::text FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn creates_new_user(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        let id = Uuid::new_v4();

        repo.create_user_if_missing(id, "a@test.local")
            .await
            .unwrap();
        let user = repo.get_user_email(id).await.unwrap();
        assert_eq!(user.unwrap(), "a@test.local");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn existing_user_is_updated(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        let id = Uuid::new_v4();

        repo.create_user_if_missing(id, "first@test.local")
            .await
            .unwrap();
        // a repeat login is not an error, and doesn't overwrite the stored email
        repo.create_user_if_missing(id, "second@test.local")
            .await
            .unwrap();

        let user = repo.get_user_email(id).await.unwrap();
        assert_eq!(user.unwrap(), "second@test.local");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn email_taken_by_other_user_is_rejected(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        repo.create_user_if_missing(Uuid::new_v4(), "dup@test.local")
            .await
            .unwrap();

        // `ON CONFLICT (id)` only covers the id; the UNIQUE(email) constraint
        // still fires, and CITEXT makes the check case-insensitive
        let err = repo
            .create_user_if_missing(Uuid::new_v4(), "DUP@test.local")
            .await
            .unwrap_err();

        assert!(
            err.as_database_error()
                .is_some_and(|e| e.is_unique_violation()),
            "expected unique violation, got {err:?}"
        );
    }
    #[sqlx::test(migrations = "../migrations")]
    async fn get_user_email_returns_email(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        let email = "test@test.local";
        repo.create_user_if_missing(Uuid::new_v4(), email)
            .await
            .unwrap();

        let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        let user_email = repo.get_user_email(user_id).await.unwrap();
        assert_eq!(user_email.unwrap(), email);
    }
    #[sqlx::test(migrations = "../migrations")]
    async fn get_user_email_returns_none(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());

        let user_email = repo.get_user_email(Uuid::new_v4()).await.unwrap();
        assert_eq!(user_email, None);
    }
}
