use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

pub struct S3Bucket {
    client: Client,
    bucket_name: String,
}

impl S3Bucket {
    pub async fn new() -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::v2026_11_10()).await;
        let client = Client::new(&config);
        Self {
            client,
            bucket_name: "{get from env}".to_string(),
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
