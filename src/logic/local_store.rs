use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use tokio::sync::Mutex;

use crate::types::{
    DocStatus, DocStatusError, FileLocation, FileStoreImplementation, LocalPath, QueueError,
    StatusStoreImplementation, StoreError, TaskID, TaskMessage, TaskQueueImplementation,
};
use aws_config::{self, BehaviorVersion, SdkConfig};
use aws_sdk_s3::Client;

/// Local filesystem-based implementation of FileStore.
#[derive(Debug, Clone)]
pub struct LocalFileStore {
    base_path: PathBuf,
    s3_config: S3ConfigParams,
}
impl Default for LocalFileStore {
    fn default() -> Self {
        let base_path = std::env::var("LOCAL_STORE_PATH")
            .unwrap_or_else(|_| String::from("./data"))
            .into();
        LocalFileStore {
            base_path,
            s3_config: S3ConfigParams::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct S3ConfigParams {
    endpoint: String,
    region: String,
    default_bucket: String,
    access_key: String,
    secret_key: String,
}
impl Default for S3ConfigParams {
    fn default() -> Self {
        S3ConfigParams {
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_err| {
                let default_endpoint = "https://sfo3.digitaloceanspaces.com";
                println!(
                    "S3_ENDPOINT not defined, defaulting to {}",
                    default_endpoint
                );
                default_endpoint.into()
            }),
            region: std::env::var("S3_REGION").unwrap_or_else(|_err| {
                let default_region = "sfo3";
                println!("S3_REGION not defined, defaulting to {}", default_region);
                default_region.into()
            }),
            default_bucket: std::env::var("S3_CRIMSON_BUCKET").unwrap_or_else(|_err| {
                let default_bucket = "crimsondocs";
                println!(
                    "S3_CRIMSON_BUCKET not defined, defaulting to {}",
                    default_bucket
                );
                default_bucket.into()
            }),

            access_key: std::env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY Not Set"),
            secret_key: std::env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY Not Set"),
        }
    }
}

impl LocalFileStore {
    /// Create a new LocalFileStore with the given base directory.
    pub fn new(base_path: PathBuf, s3_config: S3ConfigParams) -> Self {
        LocalFileStore {
            base_path,
            s3_config,
        }
    }
}

impl FileStoreImplementation for LocalFileStore {
    async fn upload_from_file(
        &self,
        local_path: LocalPath,
        _: String,
    ) -> Result<FileLocation, StoreError> {
        Ok(FileLocation::LocalPath(local_path))
    }

    async fn download_to_file(&self, src: &FileLocation) -> Result<LocalPath, StoreError> {
        match src {
            FileLocation::LocalPath(rel) => Ok(rel.clone()),
            FileLocation::S3Location(s3_loc) => {
                let bucket = &s3_loc.bucket;
                let key = &s3_loc.bucket;

                // Parse S3 URI of form s3://bucket/key
                // Load AWS config and create client
                let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                let client = Client::new(&config);
                // Download object
                let resp = client
                    .get_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|err| StoreError::S3(err.into()))?;
                // Read body
                let data = resp
                    .body
                    .collect()
                    .await
                    .map_err(|_| StoreError::InvalidLocation)?;
                let bytes = data.to_vec();
                // Determine local path and write file
                let rel_path = PathBuf::from(key);
                let full_path = self.base_path.join(&rel_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)
                        .await
                        .map_err(|_| StoreError::LocalFile)?;
                }
                fs::write(&full_path, &bytes)
                    .await
                    .map_err(|_| StoreError::LocalFile)?;
                Ok(rel_path)
            }
        }
    }

    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError> {
        match target {
            FileLocation::LocalPath(rel) => {
                let path = self.base_path.join(rel);
                fs::remove_file(&path)
                    .await
                    .map_err(|_| StoreError::LocalFile)?;
                Ok(())
            }
            _ => Err(StoreError::InvalidLocation),
        }
    }
}

/// In-memory FIFO task queue.
#[derive(Debug, Clone)]
pub struct InMemoryTaskQueue {
    queue: Arc<Mutex<VecDeque<TaskMessage>>>,
}

impl InMemoryTaskQueue {
    /// Create a new empty InMemoryTaskQueue.
    pub fn new() -> Self {
        InMemoryTaskQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

// #[async_trait]
impl TaskQueueImplementation for InMemoryTaskQueue {
    async fn enqueue(self, task: TaskMessage) -> Result<(), QueueError> {
        let mut q = self.queue.lock().await;
        q.push_back(task);
        Ok(())
    }

    async fn dequeue(self) -> Result<Option<TaskMessage>, QueueError> {
        let mut q = self.queue.lock().await;
        Ok(q.pop_front())
    }
}

/// In-memory metadata/status store.
#[derive(Debug, Clone)]
pub struct InMemoryStatusStore {
    store: Arc<Mutex<HashMap<TaskID, DocStatus>>>,
}

impl InMemoryStatusStore {
    /// Create a new empty InMemoryStatusStore.
    pub fn new() -> Self {
        InMemoryStatusStore {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// #[async_trait]
impl StatusStoreImplementation for InMemoryStatusStore {
    async fn set_doc_status(&self, status: DocStatus) -> Result<(), DocStatusError> {
        let mut m = self.store.lock().await;
        m.insert(status.request_id, status);
        Ok(())
    }

    async fn get_doc_status(&self, id: TaskID) -> Result<DocStatus, DocStatusError> {
        let m = self.store.lock().await;
        if let Some(s) = m.get(&id) {
            Ok(s.clone())
        } else {
            Err(DocStatusError::DocidNotFound)
        }
    }
}
