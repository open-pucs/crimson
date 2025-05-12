use std::{collections::{HashMap, VecDeque}, path::{Path, PathBuf}, sync::Arc};
use tokio::fs;
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::types::{
    DocStatus,
    DocStatusError,
    FileLocation,
    QueueError,
    StoreError,
    TaskID,
    TaskMessage,
    FileStore as FileStoreTrait,
    StatusStore as StatusStoreTrait,
    TaskQueue as TaskQueueTrait,
};

/// Local filesystem-based implementation of FileStore.
#[derive(Debug, Clone)]
pub struct LocalFileStore {
    base_path: PathBuf,
}

impl LocalFileStore {
    /// Create a new LocalFileStore with the given base directory.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        LocalFileStore {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl FileStoreTrait for LocalFileStore {
    async fn upload(&self, data: &[u8], dest: &FileLocation) -> Result<(), StoreError> {
        match dest {
            FileLocation::LocalPath(rel) => {
                let path = self.base_path.join(rel);
                if let Some(dir) = path.parent() {
                    fs::create_dir_all(dir).await.map_err(|_| StoreError::InvalidLocation)?;
                }
                fs::write(&path, data).await.map_err(|_| StoreError::InvalidLocation)?;
                Ok(())
            }
            _ => Err(StoreError::InvalidLocation),
        }
    }

    async fn download(&self, src: &FileLocation) -> Result<Vec<u8>, StoreError> {
        match src {
            FileLocation::LocalPath(rel) => {
                let path = self.base_path.join(rel);
                let data = fs::read(&path).await.map_err(|_| StoreError::InvalidLocation)?;
                Ok(data)
            }
            _ => Err(StoreError::InvalidLocation),
        }
    }

    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError> {
        match target {
            FileLocation::LocalPath(rel) => {
                let path = self.base_path.join(rel);
                fs::remove_file(&path).await.map_err(|_| StoreError::InvalidLocation)?;
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

#[async_trait]
impl TaskQueueTrait for InMemoryTaskQueue {
    async fn enqueue(&self, task: TaskMessage) -> Result<(), QueueError> {
        let mut q = self.queue.lock().await;
        q.push_back(task);
        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<TaskMessage>, QueueError> {
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

#[async_trait]
impl StatusStoreTrait for InMemoryStatusStore {
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