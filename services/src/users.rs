use db::users::{UserRepo, UserRepoTrait};
use uuid::Uuid;

use crate::error::{Result, internal};

#[derive(Clone)]
pub struct UserService {
    repo: UserRepo,
}

impl UserService {
    pub fn new(repo: UserRepo) -> Self {
        Self { repo }
    }
    pub async fn create_user_if_missing(&self, id: Uuid, email: &str) -> Result<()> {
        self.repo
            .create_user_if_missing(id, email)
            .await
            .map_err(internal)
    }
    pub async fn get_user_email(&self, id: Uuid) -> Result<Option<String>> {
        self.repo.get_user_email(id).await.map_err(internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use sqlx::PgPool;

    fn service(pool: PgPool) -> UserService {
        UserService::new(UserRepo::new(pool))
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn created_user_email_is_returned(pool: PgPool) {
        let svc = service(pool);
        let id = Uuid::new_v4();

        svc.create_user_if_missing(id, "a@test.local")
            .await
            .unwrap();

        assert_eq!(
            svc.get_user_email(id).await.unwrap().as_deref(),
            Some("a@test.local")
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn unknown_user_has_no_email(pool: PgPool) {
        let svc = service(pool);

        assert_eq!(svc.get_user_email(Uuid::new_v4()).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn repeat_create_updates_email(pool: PgPool) {
        let svc = service(pool);
        let id = Uuid::new_v4();

        svc.create_user_if_missing(id, "first@test.local")
            .await
            .unwrap();
        svc.create_user_if_missing(id, "second@test.local")
            .await
            .unwrap();

        assert_eq!(
            svc.get_user_email(id).await.unwrap().as_deref(),
            Some("second@test.local")
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn db_error_is_mapped_to_internal(pool: PgPool) {
        let svc = service(pool);
        svc.create_user_if_missing(Uuid::new_v4(), "dup@test.local")
            .await
            .unwrap();

        // UNIQUE(email) violation from the repo surfaces as Error::Internal
        let err = svc
            .create_user_if_missing(Uuid::new_v4(), "dup@test.local")
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Internal(_)), "got {err:?}");
    }
}
