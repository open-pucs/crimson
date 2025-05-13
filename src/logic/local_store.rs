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
    s3_config: (),
}

impl LocalFileStore {
    /// Create a new LocalFileStore with the given base directory.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        LocalFileStore {
            base_path: base_path.as_ref().to_path_buf(),
            s3_config: (),
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
            FileLocation::S3Uri(s3_uri) => {
                // Parse S3 URI of form s3://bucket/key
                let uri = s3_uri
                    .strip_prefix("s3://")
                    .ok_or(StoreError::InvalidLocation)?;
                let mut parts = uri.splitn(2, '/');
                let bucket = parts.next().unwrap();
                let key = parts.next().ok_or(StoreError::InvalidLocation)?;
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
                    .map_err(|_| StoreError::InvalidLocation)?;
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
