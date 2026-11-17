use async_trait::async_trait;
use aws_sdk_s3::Client;

use crate::load_aws_config;

#[derive(Clone)]
pub struct S3BucketService {
    client: Client,
    bucket_name: String,
}
#[async_trait]
pub trait S3BucketServiceTrait: Send + Sync {
    async fn upload(&self, key: &str, bytes: Vec<u8>) -> Result<(), aws_sdk_s3::Error>;
}

impl S3BucketService {
    pub async fn new(bucket_name: String) -> Self {
        let config = load_aws_config().await;
        let client = Client::new(&config);
        Self {
            client,
            bucket_name,
        }
    }
}
#[async_trait]
impl S3BucketServiceTrait for S3BucketService {
    async fn upload(&self, key: &str, bytes: Vec<u8>) -> Result<(), aws_sdk_s3::Error> {
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(bytes.into())
            .send()
            .await?;
        Ok(())
    }
}
