use thiserror::Error;

/// Errors for file storage operations on S3.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("S3 error: {0}")]
    S3(#[from] aws_sdk_s3::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid file location for S3")]
    InvalidLocation,
}

/// Errors for queue operations on Redis.
#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Errors for metadata store operations on Redis.
#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
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
pub trait MetadataStore {
    async fn set_stage(&self, id: TaskID, stage: ProcessingStage) -> Result<(), MetadataError>;
    async fn get_stage(&self, id: TaskID) -> Result<ProcessingStage, MetadataError>;
    async fn set_metadata(&self, id: TaskID, meta: DocumentMetadata) -> Result<(), MetadataError>;
    async fn get_metadata(&self, id: TaskID) -> Result<DocumentMetadata, MetadataError>;
}
