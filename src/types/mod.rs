use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};
use thiserror::Error;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type TaskID = u64;

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum FileLocation {
    S3Location(S3Location),
    LocalPath(PathBuf),
}

pub type LocalPath = PathBuf;

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Clone, Copy)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}
impl ProcessingStage {
    fn is_successful(&self) -> bool {
        self == &ProcessingStage::Completed
    }
    fn is_finished(&self) -> bool {
        self == &ProcessingStage::Completed || self == &ProcessingStage::Errored
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct DocStatusResponse {
    request_id: TaskID,
    request_check_url: String,
    request_check_leaf: String,
    markdown: Option<String>,
    status: ProcessingStage,
    success: bool,
    completed: bool,
    images: Option<HashMap<String, String>>,
    metadata: Option<HashMap<String, String>>,
    error: Option<String>,
}

pub static DOMAIN: LazyLock<String> =
    LazyLock::new(|| std::env::var("DOMAIN").expect("DOMAIN environment variable must be set"));

fn make_request_url(id: TaskID) -> String {
    (DOMAIN).to_string() + &make_request_leaf(id)
}
fn make_request_leaf(id: TaskID) -> String {
    format!("/v1/status/{}", id)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, JsonSchema)]
pub enum MarkdownConversionMethod {
    #[default]
    Simple,
    Marker,
}

#[derive(Debug, Clone)]
pub struct DocStatus {
    pub file_location: FileLocation,
    pub request_id: TaskID,
    pub conversion_method: MarkdownConversionMethod,
    // queue_id: u64,
    pub markdown: Option<String>,
    pub status: ProcessingStage,
    pub images: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub error: Option<String>,
}
impl DocStatus {
    pub fn new_from_id_loc(
        id: TaskID,
        location: FileLocation,
        method: MarkdownConversionMethod,
    ) -> Self {
        DocStatus {
            file_location: location,
            conversion_method: method,
            request_id: id,
            markdown: None,
            status: ProcessingStage::Waiting,
            metadata: None,
            images: None,
            error: None,
        }
    }
}

impl From<DocStatus> for DocStatusResponse {
    fn from(input: DocStatus) -> Self {
        DocStatusResponse {
            request_id: input.request_id,
            request_check_url: make_request_url(input.request_id),
            request_check_leaf: make_request_leaf(input.request_id),
            markdown: input.markdown,
            status: input.status,
            success: input.status.is_successful(),
            completed: input.status.is_finished(),
            images: input.images,
            metadata: input.metadata,
            error: input.error,
        }
    }
}

/// Simplified task message carrying ID and file location.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: TaskID,
    pub location: FileLocation,
}

/// Abstract file storage (upload/download/delete).
// #[async_trait]
pub trait FileStoreImplementation {
    async fn upload_from_file(
        &self,
        local_path: LocalPath,
        upload_key: String,
    ) -> Result<FileLocation, StoreError>;
    async fn download_to_file(&self, src: &FileLocation) -> Result<LocalPath, StoreError>;
    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError>;
}

/// Abstract FIFO task queue for enqueuing and dequeuing tasks.
pub trait TaskQueueImplementation {
    async fn enqueue(self, task: TaskMessage) -> Result<(), QueueError>;
    async fn dequeue(self) -> Result<Option<TaskMessage>, QueueError>;
}

/// Metadata store for tracking processing stage and other data.
pub trait StatusStoreImplementation {
    async fn set_doc_status(&self, status: DocStatus) -> Result<(), DocStatusError>;
    async fn get_doc_status(&self, id: TaskID) -> Result<DocStatus, DocStatusError>;
}

// Errors for file storage operations on S3.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("S3 error: {0}")]
    S3(#[from] aws_sdk_s3::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error in Local File System")]
    LocalFile,
    #[error("Invalid file location for File")]
    InvalidLocation,
}

/// Errors for queue operations on Redis.
#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Processing Queue is Empty")]
    QueueEmpty,
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Errors for metadata store operations on Redis.
#[derive(Error, Debug)]
pub enum DocStatusError {
    #[error("Doc ID Not Found")]
    DocidNotFound,
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct S3Location {
    pub key: String,
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
}
// https://examplebucket.sfo3.digitaloceanspaces.com/this/is/the/file/key
//
// For this example the
// bucket: examplebucket
// region: sfo3
// endpoint: https://sfo3.digitaloceanspaces.com
// key: this/is/the/file/key
impl From<S3Location> for String {
    fn from(location: S3Location) -> String {
        // strip off the "https://" prefix on the endpoint to get e.g. "sfo3.digitaloceanspaces.com"
        let host_part = location
            .endpoint
            .strip_prefix("https://")
            .unwrap_or(&location.endpoint);

        // make sure we don’t end up with duplicate or missing slashes
        let key_part = location.key.trim_start_matches('/');

        // build "https://{bucket}.{host_part}/{key…}"
        let mut url = format!("https://{}.{}", location.bucket, host_part);
        if !key_part.is_empty() {
            url.push('/');
            url.push_str(key_part);
        }
        url
    }
}

//
// Try to parse the URL back into its components.
// Expects URLs of the form
//   "https://{bucket}.{region}.{rest…}/{key…}"
//
impl TryFrom<String> for S3Location {
    type Error = StoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // 1) strip the scheme
        let without_scheme = value
            .strip_prefix("https://")
            .ok_or(StoreError::InvalidLocation)?;

        // 2) split into host vs. path
        //    e.g. "examplebucket.sfo3.digitaloceanspaces.com/this/is/…"
        let mut split = without_scheme.splitn(2, '/');
        let host = split.next().unwrap();
        let key = split.next().unwrap_or("").to_string();

        // 3) break host into bucket, region, and the rest of the domain
        //    host_parts = ["examplebucket", "sfo3", "digitaloceanspaces.com"]
        let host_parts: Vec<&str> = host.split('.').collect();
        if host_parts.len() < 3 {
            return Err(StoreError::InvalidLocation);
        }
        let bucket = host_parts[0].to_string();
        let region = host_parts[1].to_string();
        let domain_rest = host_parts[2..].join(".");

        // 4) rebuild the endpoint (scheme + region + rest-of-domain)
        let endpoint = format!("https://{}.{}", region, domain_rest);

        Ok(S3Location {
            key,
            bucket,
            region,
            endpoint,
        })
    }
}
