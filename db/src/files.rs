use async_trait::async_trait;
use domain::files::FileMetaData;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct FilesRepo {
    pool: PgPool,
}

impl FilesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait FilesRepoTrait: Send + Sync {
    async fn insert_file_metadata(&self, file: &FileMetaData) -> Result<Uuid, sqlx::Error>;
    async fn delete_file_metadata(&self, id: &Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl FilesRepoTrait for FilesRepo {
    async fn insert_file_metadata(&self, file: &FileMetaData) -> Result<Uuid, sqlx::Error> {
        let id = sqlx::query_scalar(
            "INSERT INTO files (id, original_name, ext, mime_type, size, user_id)
                    VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING id",
        )
        .bind(file.id)
        .bind(&file.original_name)
        .bind(&file.ext)
        .bind(&file.mime_type)
        .bind(file.size)
        .bind(file.user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }
    async fn delete_file_metadata(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::users::{UserRepo, UserRepoTrait};

    // files.user_id references users(id), so every test needs a user first
    async fn user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        UserRepo::new(pool.clone())
            .create_user_if_missing(id, &format!("{id}@test.local"))
            .await
            .unwrap();
        id
    }

    fn metadata(user_id: Uuid) -> FileMetaData {
        FileMetaData {
            id: Uuid::new_v4(),
            original_name: "notes.md".into(),
            ext: "md".into(),
            mime_type: "text/markdown".into(),
            size: 42,
            user_id,
        }
    }

    async fn count(pool: &PgPool, id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn insert_returns_given_id_and_stores_fields(pool: PgPool) {
        let repo = FilesRepo::new(pool.clone());
        let file = metadata(user(&pool).await);

        let id = repo.insert_file_metadata(&file).await.unwrap();
        assert_eq!(id, file.id);

        let row: (String, String, String, i64, Uuid) = sqlx::query_as(
            "SELECT original_name, ext, mime_type, size, user_id FROM files WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            row,
            (
                file.original_name,
                file.ext,
                file.mime_type,
                file.size,
                file.user_id
            )
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn insert_duplicate_id_is_rejected(pool: PgPool) {
        let repo = FilesRepo::new(pool.clone());
        let file = metadata(user(&pool).await);
        repo.insert_file_metadata(&file).await.unwrap();

        let err = repo.insert_file_metadata(&file).await.unwrap_err();
        assert!(
            err.as_database_error()
                .is_some_and(|e| e.is_unique_violation()),
            "expected unique violation, got {err:?}"
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn insert_for_unknown_user_is_rejected(pool: PgPool) {
        let repo = FilesRepo::new(pool);
        let err = repo
            .insert_file_metadata(&metadata(Uuid::new_v4()))
            .await
            .unwrap_err();
        assert!(
            err.as_database_error()
                .is_some_and(|e| e.is_foreign_key_violation()),
            "expected foreign key violation, got {err:?}"
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn insert_negative_size_is_rejected(pool: PgPool) {
        let repo = FilesRepo::new(pool.clone());
        let mut file = metadata(user(&pool).await);
        file.size = -1;

        let err = repo.insert_file_metadata(&file).await.unwrap_err();
        assert!(
            err.as_database_error()
                .is_some_and(|e| e.is_check_violation()),
            "expected check violation, got {err:?}"
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn delete_removes_only_that_file(pool: PgPool) {
        let repo = FilesRepo::new(pool.clone());
        let user_id = user(&pool).await;
        let keep = metadata(user_id);
        let remove = metadata(user_id);
        repo.insert_file_metadata(&keep).await.unwrap();
        repo.insert_file_metadata(&remove).await.unwrap();

        repo.delete_file_metadata(&remove.id).await.unwrap();

        assert_eq!(count(&pool, remove.id).await, 0);
        assert_eq!(count(&pool, keep.id).await, 1);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn delete_unknown_id_is_ok(pool: PgPool) {
        // the upload rollback path may call this for a row that's already gone
        let repo = FilesRepo::new(pool);
        repo.delete_file_metadata(&Uuid::new_v4()).await.unwrap();
    }
}
