use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct S3BucketService {
    client: Client,
    bucket_name: String,
}

impl S3BucketService {
    pub async fn new(bucket_name: String) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::v2026_11_10()).await;
        let client = Client::new(&config);
        Self {
            client,
            bucket_name,
        }
    }
    pub async fn upload(&self, key: &str, bytes: Vec<u8>) -> Result<(), aws_sdk_s3::Error> {
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
