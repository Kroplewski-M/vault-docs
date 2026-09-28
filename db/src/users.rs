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
        sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
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

    // `email` is CITEXT, which sqlx won't decode into `String`, so cast it to text
    async fn emails_for(pool: &PgPool, id: Uuid) -> Vec<String> {
        sqlx::query_scalar("SELECT email::text FROM users WHERE id = $1")
            .bind(id)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn creates_new_user(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        let id = Uuid::new_v4();

        repo.create_user_if_missing(id, "a@test.local")
            .await
            .unwrap();

        assert_eq!(emails_for(&pool, id).await, ["a@test.local"]);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn existing_user_is_left_untouched(pool: PgPool) {
        let repo = UserRepo::new(pool.clone());
        let id = Uuid::new_v4();

        repo.create_user_if_missing(id, "first@test.local")
            .await
            .unwrap();
        // a repeat login is not an error, and doesn't overwrite the stored email
        repo.create_user_if_missing(id, "second@test.local")
            .await
            .unwrap();

        assert_eq!(emails_for(&pool, id).await, ["first@test.local"]);
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
}
