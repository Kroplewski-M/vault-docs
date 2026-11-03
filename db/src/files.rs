use async_trait::async_trait;
use domain::files::FileMetaData;
use sqlx::PgPool;

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
    async fn insert_file(&self, file: &FileMetaData) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl FilesRepoTrait for FilesRepo {
    async fn insert_file(&self, file: &FileMetaData) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO files (id, original_name, ext, mime_type, size, user_id)
                     VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(file.id)
        .bind(&file.original_name)
        .bind(&file.ext)
        .bind(&file.mime_type)
        .bind(file.size)
        .bind(file.user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
