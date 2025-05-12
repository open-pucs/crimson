use std::collections::HashMap;
use thiserror::Error;
use async_trait::async_trait;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type TaskID = u64;

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum FileLocation {
    S3Uri(String),
    LocalPath(String),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Clone, Copy)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct DocStatusResponse {
    request_id: TaskID,
    request_check_url: String,
    markdown: Option<String>,
    status: ProcessingStage,
    success: bool,
    images: Option<HashMap<String, String>>,
    metadata: Option<HashMap<String, String>>,
    error: Option<String>,
}

fn make_request_url(id: TaskID) -> String {
    "/v1/status/".to_string() + &id.to_string()
}

#[derive(Debug, Clone)]
pub struct DocStatus {
    file_location: FileLocation,
    pub request_id: TaskID,
    // queue_id: u64,
    markdown: Option<String>,
    status: ProcessingStage,
    images: Option<HashMap<String, String>>,
    metadata: Option<HashMap<String, String>>,
    error: Option<String>,
}

impl From<DocStatus> for DocStatusResponse {
    fn from(input: DocStatus) -> Self {
        DocStatusResponse {
            request_id: input.request_id,
            request_check_url: make_request_url(input.request_id),
            markdown: input.markdown,
            status: input.status,
            success: input.status == ProcessingStage::Completed,
            images: input.images,
            metadata: input.metadata,
            error: input.error,
        }
    }
}

pub fn make_new_docstatus(id: TaskID, location: FileLocation) -> DocStatus {
    DocStatus {
        file_location: location,
        request_id: id,
        markdown: None,
        status: ProcessingStage::Waiting,
        metadata: None,
        images: None,
        error: None,
    }
}

/// Simplified task message carrying ID and file location.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: TaskID,
    pub location: FileLocation,
}

/// Abstract file storage (upload/download/delete).
#[async_trait]
pub trait FileStore {
    async fn upload(&self, data: &[u8], dest: &FileLocation) -> Result<(), StoreError>;
    async fn download(&self, src: &FileLocation) -> Result<Vec<u8>, StoreError>;
    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError>;
}

/// Abstract FIFO task queue for enqueuing and dequeuing tasks.
#[async_trait]
pub trait TaskQueue {
    async fn enqueue(&self, task: TaskMessage) -> Result<(), QueueError>;
    async fn dequeue(&self) -> Result<Option<TaskMessage>, QueueError>;
}

/// Metadata store for tracking processing stage and other data.
#[async_trait]
pub trait StatusStore {
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