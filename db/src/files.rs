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
