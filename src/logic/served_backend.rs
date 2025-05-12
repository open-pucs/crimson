use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client as S3Client, types::ByteStream};
use redis::AsyncCommands;
use std::sync::Arc;

use crate::types::storage::{
    FileStore, TaskQueue, MetadataStore,
    TaskMessage, FileLocation, ProcessingStage, DocumentMetadata, TaskID,
};
use crate::storage::errors::{StoreError, QueueError, MetadataError};

/// Configuration for the S3 + Redis backend.
#[derive(Clone, Debug)]
pub struct S3RedisConfig {
    pub bucket: String,
    pub region: String,
    pub redis_url: String,
}

/// Combined handle exposing file store, task queue, and metadata store.
pub struct S3RedisBackend {
    pub file_store: Arc<S3FileStore>,
    pub task_queue: Arc<RedisTaskQueue>,
    pub metadata_store: Arc<RedisMetadataStore>,
}

impl S3RedisBackend {
    /// Initialize S3 client and Redis connections.
    pub async fn new(config: &S3RedisConfig) -> Result<Self, StoreError> {
        let region_provider = RegionProviderChain::first_try(config.region.clone())
            .or_default_provider()
            .or_else("us-east-1");
        let aws_cfg = aws_config::from_env()
            .region(region_provider)
            .load()
            .await;
        let s3_client = S3Client::new(&aws_cfg);
        let file_store = Arc::new(S3FileStore { client: s3_client, bucket: config.bucket.clone() });

        let redis_client = redis::Client::open(config.redis_url.clone()).map_err(QueueError::from)?;
        let conn = redis_client.get_async_connection().await.map_err(QueueError::from)?;
        let task_queue = Arc::new(RedisTaskQueue { conn: conn.clone() });
        let metadata_store = Arc::new(RedisMetadataStore { conn });

        Ok(Self { file_store, task_queue, metadata_store })
    }
}

/// S3-based implementation of FileStore.
pub struct S3FileStore {
    client: S3Client,
    bucket: String,
}

#[async_trait]
impl FileStore for S3FileStore {
    async fn upload(&self, data: &[u8], dest: &FileLocation) -> Result<(), StoreError> {
        let key = match dest {
            FileLocation::S3Uri(uri) => uri,
            _ => return Err(StoreError::InvalidLocation),
        };
        self.client.put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await?;
        Ok(())
    }

    async fn download(&self, src: &FileLocation) -> Result<Vec<u8>, StoreError> {
        let key = match src {
            FileLocation::S3Uri(uri) => uri,
            _ => return Err(StoreError::InvalidLocation),
        };
        let resp = self.client.get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        let bytes = resp.body.collect().await?.into_bytes().to_vec();
        Ok(bytes)
    }

    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError> {
        let key = match target {
            FileLocation::S3Uri(uri) => uri,
            _ => return Err(StoreError::InvalidLocation),
        };
        self.client.delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

/// Redis-based implementation of TaskQueue.
pub struct RedisTaskQueue {
    conn: redis::aio::Connection,
}

#[async_trait]
impl TaskQueue for RedisTaskQueue {
    async fn enqueue(&self, task: TaskMessage) -> Result<(), QueueError> {
        let json = serde_json::to_string(&task)?;
        let mut conn = self.conn.clone();
        conn.rpush("task_queue", json).await?;
        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<TaskMessage>, QueueError> {
        let mut conn = self.conn.clone();
        if let Some(json): Option<String> = conn.lpop("task_queue").await? {
            let task = serde_json::from_str(&json)?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
}

/// Redis-based implementation of MetadataStore.
pub struct RedisMetadataStore {
    conn: redis::aio::Connection,
}

#[async_trait]
impl MetadataStore for RedisMetadataStore {
    async fn set_stage(&self, id: TaskID, stage: ProcessingStage) -> Result<(), MetadataError> {
        let mut conn = self.conn.clone();
        let value = serde_json::to_string(&stage)?;
        conn.hset(format!("metadata:{}", id), "stage", value).await?;
        Ok(())
    }

    async fn get_stage(&self, id: TaskID) -> Result<ProcessingStage, MetadataError> {
        let mut conn = self.conn.clone();
        let raw: String = conn.hget(format!("metadata:{}", id), "stage").await?;
        let stage = serde_json::from_str(&raw)?;
        Ok(stage)
    }

    async fn set_metadata(&self, id: TaskID, meta: DocumentMetadata) -> Result<(), MetadataError> {
        let mut conn = self.conn.clone();
        let value = serde_json::to_string(&meta)?;
        conn.hset(format!("metadata:{}", id), "data", value).await?;
        Ok(())
    }

    async fn get_metadata(&self, id: TaskID) -> Result<DocumentMetadata, MetadataError> {
        let mut conn = self.conn.clone();
        let raw: String = conn.hget(format!("metadata:{}", id), "data").await?;
        let meta = serde_json::from_str(&raw)?;
        Ok(meta)
    }
}
