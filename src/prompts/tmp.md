Given all the interfaces that currently exist in src/types/mod.rs

Could you go ahead and implement the following in memory data structures:

### 2.2 Local Filesystem + In-Memory Backend

- **FileStore**: Stores files under a configured `base_path` on disk.  
- **TaskQueue**: In-memory `VecDeque<TaskMessage>` protected by `tokio::sync::Mutex`.  
- **MetadataStore**: Either in-memory `HashMap<TaskID, ...>` with `Mutex` .


And implement the following traits on them from src/types/mod.rs:

```rs
/// Abstract file storage (upload/download/delete).
pub trait FileStore {
    async fn upload(&self, data: &[u8], dest: &FileLocation) -> Result<(), StoreError>;
    async fn download(&self, src: &FileLocation) -> Result<Vec<u8>, StoreError>;
    async fn delete(&self, target: &FileLocation) -> Result<(), StoreError>;
}

/// Abstract FIFO task queue for enqueuing and dequeuing tasks.
pub trait TaskQueue {
    async fn enqueue(&self, task: TaskMessage) -> Result<(), QueueError>;
    async fn dequeue(&self) -> Result<Option<TaskMessage>, QueueError>;
}

/// Metadata store for tracking processing stage and other data.
pub trait StatusStore {
    async fn set_doc_status(&self, status: DocStatus) -> Result<(), DocStatusError>;
    async fn get_doc_status(&self, id: TaskID) -> Result<DocStatus, DocStatusError>;
}

```
While experimenting throw your code in src/logic/local_store.rs 

Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes.

Also you can look up documentation for popular rust libraries like tokio, serde and axum by using the context7 tool, its support on less popular libraries is limited unfortunately. Whenever you are stuck with some inscrutable errors, it can be helpful to look up examples to see how the code should be structured.

