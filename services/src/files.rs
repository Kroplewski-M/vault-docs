use std::{path::Path, sync::Arc};

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Generate, Payload},
};
use db::files::FilesRepoTrait;
use domain::{
    files::{EncryptedFileMetaData, FileMetaData, FileStatus, UploadedFile},
    models::EncVersion,
};
use uuid::Uuid;

use crate::{
    Error, Result, error::internal, kms::KmsServiceTrait, s3_bucket::S3BucketServiceTrait,
};
#[derive(Clone)]
pub struct FileService {
    file_repo: Arc<dyn FilesRepoTrait>,
    s3_service: Arc<dyn S3BucketServiceTrait>,
    kms_service: Arc<dyn KmsServiceTrait>,
}

impl FileService {
    pub fn new(
        file_repo: Arc<dyn FilesRepoTrait>,
        s3_service: Arc<dyn S3BucketServiceTrait>,
        kms_service: Arc<dyn KmsServiceTrait>,
    ) -> Self {
        Self {
            file_repo,
            s3_service,
            kms_service,
        }
    }
    pub async fn encrypt_file(&self, bytes: &[u8], file_id: Uuid) -> Result<EncryptedFileMetaData> {
        let key = self
            .kms_service
            .generate_data_key(&file_id.to_string())
            .await
            .map_err(internal)?;
        let nonce = aes_gcm::aead::Nonce::<Aes256Gcm>::generate();
        let cipher = Aes256Gcm::new_from_slice(&key.plain_text).map_err(internal)?;
        let encrypted_file = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: bytes,
                    aad: file_id.as_bytes(),
                },
            )
            .map_err(internal)?;

        Ok(EncryptedFileMetaData {
            bytes: encrypted_file,
            nonce: nonce.to_vec(),
            encryption_key: key,
            enc_version: EncVersion::V1,
        })
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

        let encrypted_file = self.encrypt_file(bytes, file.id).await.map_err(internal)?;

        let meta = FileMetaData {
            id: file.id,
            original_name: base_file_name.to_string(),
            ext: format.extension().to_string(),
            mime_type: format.mime_type().to_string(),
            size: file.size(),
            user_id,
            enc_key: encrypted_file.encryption_key.encrypted,
            nonce: encrypted_file.nonce,
            enc_version: encrypted_file.enc_version,
        };

        // 1. encrypt  2. put to S3 under meta.id
        let file_id = self
            .file_repo
            .insert_file_metadata(&meta)
            .await
            .map_err(internal)?;

        match self
            .s3_service
            .upload(&file.id.to_string(), encrypted_file.bytes)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                self.file_repo
                    .delete_file_metadata(&file_id)
                    .await
                    .map_err(internal)?;
                return Err(internal(err));
            }
        }

        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use aws_sdk_s3::types::error::NoSuchBucket;
    use db::{
        files::FilesRepo,
        users::{UserRepo, UserRepoTrait},
    };
    use domain::{files::UploadError, models::DataKey};
    use sqlx::PgPool;

    const PDF_BYTES: &[u8] = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n";
    const TEXT_BYTES: &[u8] = b"# hello\nsome notes";

    #[derive(Default)]
    struct FakeS3 {
        fail: bool,
        uploaded: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl S3BucketServiceTrait for FakeS3 {
        async fn upload(
            &self,
            key: &str,
            _bytes: Vec<u8>,
        ) -> std::result::Result<(), aws_sdk_s3::Error> {
            self.uploaded.lock().unwrap().push(key.to_string());
            if self.fail {
                return Err(aws_sdk_s3::Error::NoSuchBucket(
                    NoSuchBucket::builder().build(),
                ));
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeKms {
        generated: Mutex<Vec<String>>, // doc_ids passed to generate_data_key
    }
    #[async_trait::async_trait]
    impl KmsServiceTrait for FakeKms {
        async fn generate_data_key(
            &self,
            doc_id: &str,
        ) -> std::result::Result<DataKey, aws_sdk_kms::Error> {
            self.generated.lock().unwrap().push(doc_id.to_string());
            Ok(DataKey {
                plain_text: vec![7u8; 32], // fixed 32-byte key (AES-256)
                encrypted: format!("enc:{doc_id}").into_bytes(), // fake wrapped key
            })
        }

        async fn decrypt_data_key(
            &self,
            _doc_id: &str,
            _encrypted: &[u8],
        ) -> std::result::Result<Vec<u8>, aws_sdk_kms::Error> {
            Ok(vec![7u8; 32])
        }
    }

    fn service(pool: PgPool, s3: Arc<FakeS3>) -> FileService {
        FileService::new(
            Arc::new(FilesRepo::new(pool)),
            s3,
            Arc::new(FakeKms::default()),
        )
    }

    // files.user_id references users(id), so every upload needs a user first
    async fn user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        UserRepo::new(pool.clone())
            .create_user_if_missing(id, &format!("{id}@test.local"))
            .await
            .unwrap();
        id
    }

    async fn file_count(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn invalid_file_is_rejected_without_insert_or_upload(pool: PgPool) {
        let s3 = Arc::new(FakeS3::default());
        let err = service(pool.clone(), s3.clone())
            .upload_file(user(&pool).await, "virus.exe".into(), TEXT_BYTES)
            .await
            .unwrap_err();

        assert!(matches!(&err, Error::InvalidFile(e) if e == &[UploadError::UnsupportedExtension]));
        assert_eq!(file_count(&pool).await, 0);
        assert!(s3.uploaded.lock().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn s3_failure_deletes_row_and_returns_internal(pool: PgPool) {
        let s3 = Arc::new(FakeS3 {
            fail: true,
            ..Default::default()
        });
        let err = service(pool.clone(), s3.clone())
            .upload_file(user(&pool).await, "notes.md".into(), TEXT_BYTES)
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Internal(_)), "got {err:?}");
        assert_eq!(s3.uploaded.lock().unwrap().len(), 1); // upload was attempted
        assert_eq!(file_count(&pool).await, 0); // ...and rolled back
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn valid_file_strips_extension_and_sets_format(pool: PgPool) {
        let user_id = user(&pool).await;
        let meta = service(pool.clone(), Arc::new(FakeS3::default()))
            .upload_file(user_id, "report.pdf".into(), PDF_BYTES)
            .await
            .unwrap();

        assert_eq!(meta.original_name, "report");
        assert_eq!(meta.ext, "pdf");
        assert_eq!(meta.mime_type, "application/pdf");
        assert_eq!(meta.size, PDF_BYTES.len() as i64);
        assert_eq!(meta.user_id, user_id);

        let row: (String, String, String, i64, Uuid) = sqlx::query_as(
            "SELECT original_name, ext, mime_type, size, user_id FROM files WHERE id = $1",
        )
        .bind(meta.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            row,
            (
                meta.original_name,
                meta.ext,
                meta.mime_type,
                meta.size,
                meta.user_id
            )
        );
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn only_last_extension_is_stripped(pool: PgPool) {
        let meta = service(pool.clone(), Arc::new(FakeS3::default()))
            .upload_file(user(&pool).await, "notes.v2.md".into(), TEXT_BYTES)
            .await
            .unwrap();

        assert_eq!(meta.original_name, "notes.v2");
        assert_eq!(meta.ext, "md");
        assert_eq!(meta.mime_type, "text/markdown");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn successful_upload_keeps_row_and_uses_file_id_as_key(pool: PgPool) {
        let s3 = Arc::new(FakeS3::default());
        let meta = service(pool.clone(), s3.clone())
            .upload_file(user(&pool).await, "notes.txt".into(), TEXT_BYTES)
            .await
            .unwrap();

        assert_eq!(file_count(&pool).await, 1);
        assert_eq!(*s3.uploaded.lock().unwrap(), vec![meta.id.to_string()]);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn insert_failure_skips_upload(pool: PgPool) {
        let s3 = Arc::new(FakeS3::default());
        // unknown user -> FK violation on insert
        let err = service(pool.clone(), s3.clone())
            .upload_file(Uuid::new_v4(), "notes.md".into(), TEXT_BYTES)
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Internal(_)), "got {err:?}");
        assert!(s3.uploaded.lock().unwrap().is_empty());
        assert_eq!(file_count(&pool).await, 0);
    }
}
