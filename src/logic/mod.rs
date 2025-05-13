// logic module grouping local_store and interface functions
mod local_store;

use crate::logic::local_store::{InMemoryStatusStore, InMemoryTaskQueue, LocalFileStore};
use crate::types::{
    DocStatus, DocStatusError, StatusStoreImplementation, StoreError, TaskID, TaskMessage,
    TaskQueueImplementation,
};
use aws_sdk_s3::config::endpoint;
use once_cell::sync::Lazy;

/// A composite store bundling file storage, task queue, and status store.
pub struct LocalStore {
    pub file_store: LocalFileStore,
    pub task_queue: InMemoryTaskQueue,
    pub status_store: InMemoryStatusStore,
}

/// Global static local store instance.
static LOCAL_STORE: Lazy<LocalStore> = Lazy::new(|| {
    // Base path for file storage, can be configured via environment variable.
    LocalStore {
        file_store: LocalFileStore::default(),
        task_queue: InMemoryTaskQueue::new(),
        status_store: InMemoryStatusStore::new(),
    }
});

/// Retrieve a reference to the global local store.
pub fn get_local_store() -> &'static LocalStore {
    &LOCAL_STORE
}

/// Enqueue a new document processing task.
pub async fn ingest_file_to_queue(status: DocStatus) {
    // Store initial status
    let _ = LOCAL_STORE
        .status_store
        .set_doc_status(status.clone())
        .await;
    // Enqueue task for processing
    let message = TaskMessage {
        id: status.request_id,
        location: status.file_location.clone(),
    };
    let _ = LOCAL_STORE
        .task_queue
        .clone()
        .enqueue(message)
        .await
        .expect("Ingest should just work");
}

/// Update an existing task's processing status.
pub async fn update_task_data(status: DocStatus) -> Result<(), DocStatusError> {
    LOCAL_STORE.status_store.set_doc_status(status).await
}

/// Dequeue the next file processing task, returning its DocStatus.
pub async fn get_file_task_from_queue() -> Option<DocStatus> {
    if let Ok(Some(task)) = LOCAL_STORE.task_queue.clone().dequeue().await {
        // Retrieve status for this task
        return Some(
            LOCAL_STORE
                .status_store
                .get_doc_status(task.id)
                .await
                .unwrap_or_else(|err| {
                    panic!(
                        "DocStatus not found for dequeued TaskMessage, this shouldnt be possible: {}",
                        err.to_string()
                    )
                }),
        );
    }
    None
}

/// Retrieve the stored DocStatus for a given task ID.
pub async fn get_task_data_from_id(id: TaskID) -> Result<DocStatus, DocStatusError> {
    LOCAL_STORE.status_store.get_doc_status(id).await
}
