use std::path::Path;

use db::files::{FilesRepo, FilesRepoTrait};
use domain::files::{FileMetaData, FileStatus, UploadedFile};
use uuid::Uuid;

use crate::{Error, Result, error::internal, s3_bucket::S3Bucket};
#[derive(Clone)]
pub struct FileService {
    file_repo: FilesRepo,
}

impl FileService {
    pub fn new(file_repo: FilesRepo) -> Self {
        Self { file_repo }
    }
    pub async fn upload_file(
        &self,
        user_id: Uuid,
        original_name: String,
        bytes: &[u8],
    ) -> Result<FileMetaData> {
        let file = UploadedFile::new(original_name, bytes);
        let FileStatus::Valid(format) = file.status() else {
            return Err(Error::InvalidFile(file.errors().to_vec()));
        };
        let file_name = &file.name().clone();

        let base_file_name = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        let meta = FileMetaData {
            id: file.id,
            original_name: base_file_name.to_string(),
            ext: format.extension().to_string(),
            mime_type: format.mime_type().to_string(),
            size: file.size(),
            user_id,
        };
        // 1. encrypt  2. put to S3 under meta.id
        self.file_repo.insert_file(&meta).await.map_err(internal)?;
        let bucket = S3Bucket::new().await;
        bucket
            .upload(&file.id.to_string(), bytes.to_vec())
            .await
            .map_err(internal)?;
        // if the insert fails, best-effort delete the S3 object so it isn't orphaned
        Ok(meta)
    }
}
