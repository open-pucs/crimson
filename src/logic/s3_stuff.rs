use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Credentials;

use crate::types::S3Location;

use super::local_store::S3ConfigParams;

// Build a Region, Credentials and (if provided) custom Endpoint
pub async fn make_s3_client(s3_config: &S3ConfigParams, s3_loc: &S3Location) -> S3Client {
    let region = Region::new(s3_loc.region.clone());
    let creds = Credentials::new(
        &s3_config.access_key,
        &s3_config.secret_key,
        None, // no session token
        None, // no expiration
        "manual",
    );

    // Start from the env-loader so we still pick up other settings (timeouts, retry, etc)
    let cfg_loader = aws_config::defaults(BehaviorVersion::latest())
        .region(region.clone())
        .credentials_provider(creds)
        .endpoint_url(&s3_loc.endpoint);

    let sdk_config = cfg_loader.load().await;
    S3Client::new(&sdk_config)
}
