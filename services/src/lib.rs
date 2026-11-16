mod error;
pub mod files;
pub mod s3_bucket;
pub mod session;
pub mod users;

use aws_config::{BehaviorVersion, SdkConfig};
pub use error::{Error, Result};

pub async fn load_aws_config() -> SdkConfig {
    aws_config::load_defaults(BehaviorVersion::v2026_11_10()).await
}
