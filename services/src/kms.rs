use aws_sdk_kms::Client;
use aws_sdk_s3::primitives::Blob;

use crate::load_aws_config;

pub struct DataKey {
    pub plain_text: Vec<u8>,
    pub encrypted: Vec<u8>,
}

pub struct KMS {
    client: Client,
}

impl KMS {
    pub async fn new() -> Self {
        let config = load_aws_config().await;
        let client = Client::new(&config);
        Self { client }
    }
    pub async fn generate_data_key(
        &self,
        key_id: &str,
        doc_id: &str,
    ) -> Result<DataKey, aws_sdk_kms::Error> {
        let out = self
            .client
            .generate_data_key()
            .key_id(key_id)
            .key_spec(aws_sdk_kms::types::DataKeySpec::Aes256)
            .encryption_context("doc_id", doc_id)
            .send()
            .await?;

        Ok(DataKey {
            plain_text: out
                .plaintext()
                .map(|b| b.as_ref().to_vec())
                .unwrap_or_default(),
            encrypted: out
                .ciphertext_blob()
                .map(|b| b.as_ref().to_vec())
                .unwrap_or_default(),
        })
    }
    pub async fn decrypt_data_key(
        &self,
        key_id: &str,
        doc_id: &str,
        encrypted: &[u8],
    ) -> Result<Vec<u8>, aws_sdk_kms::Error> {
        let out = self
            .client
            .decrypt()
            .key_id(key_id)
            .ciphertext_blob(Blob::new(encrypted))
            .encryption_context("doc_id", doc_id)
            .send()
            .await?;
        Ok(out
            .plaintext()
            .map(|b| b.as_ref().to_vec())
            .unwrap_or_default())
    }
}
