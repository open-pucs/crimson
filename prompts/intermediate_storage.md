# Intermediate Storage Architecture

This document outlines the design and implementation plan for the **Intermediate Storage** component of the PDF batch processing system. This layer provides:

- **File Storage** abstraction (upload, download, delete).
- **Task Queue** abstraction (enqueue, dequeue).
- **Metadata Store** abstraction (status updates, lookups).

It supports two configurable modes:

1. **S3 + Redis Backend** (production, scalable)  
2. **Local Filesystem + In-Memory/Embedded DB Backend** (development, testing)

---

## 1. Core Traits & Types

### 1.1 Types

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use async_trait::async_trait;

/// Unique identifier for a document processing task.
pub type TaskID = u64;

/// Key/value metadata for a document.
pub type DocumentMetadata = HashMap<String, String>;

/// Standard enum to locate files across backends.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FileLocation {
    S3Uri(String),
    LocalPath(String),
}

/// Processing stages for a document.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ProcessingStage {
    Waiting,
    Processing,
    Completed,
    Errored,
}

/// Simplified task message carrying ID and file location.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TaskMessage {
    pub id: TaskID,
    pub location: FileLocation,
}
```

### 1.2 Traits

```rust
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
```

Error types (`StoreError`, `QueueError`, `MetadataError`) can be defined with [`thiserror`](https://crates.io/crates/thiserror).

---

## 2. Backend Implementations

### 2.1 S3 + Redis Backend

- **FileStore**:  Uses [`aws-sdk-s3`](https://crates.io/crates/aws-sdk-s3) (official AWS SDK).  
- **TaskQueue**: Uses [`redis`](https://crates.io/crates/redis) client with a Redis list (`RPUSH`/`LPOP`).  
- **MetadataStore**: Uses Redis hash (`HSET`/`HGETALL`) per `TaskID`.

Configuration example:
```rust
pub struct S3RedisConfig {
    pub bucket: String,
    pub region: String,
    pub redis_url: String,
}
```

#### Crates:
- aws-config = "0.9"
- aws-sdk-s3 = "0.9"
- redis = "0.23"
- tokio = { version = "1", features = ["full"] }
- async-trait = "0.1"
- thiserror = "1.0"

### 2.2 Local Filesystem + In-Memory Backend

- **FileStore**: Stores files under a configured `base_path` on disk.  
- **TaskQueue**: In-memory `VecDeque<TaskMessage>` protected by `tokio::sync::Mutex`.  
- **MetadataStore**: Either in-memory `HashMap<TaskID, ...>` with `Mutex` or embedded DB such as [`sled`](https://crates.io/crates/sled).

Configuration example:
```rust
pub struct LocalConfig {
    pub base_path: PathBuf,
}
```

#### Crates:
- tokio = { version = "1", features = ["full"] }
- sled = "0.34"           # optional for persistence
- async-trait = "0.1"
- thiserror = "1.0"

---

## 3. Module Layout

```
src/
└── storage/
    ├── mod.rs              # public re-exports + config loader
    ├── traits.rs           # FileStore, TaskQueue, MetadataStore
    ├── types.rs            # TaskMessage, FileLocation, ProcessingStage, errors
    ├── backend/
    │   ├── s3_redis.rs     # S3 + Redis implementation
    │   └── local.rs        # Local FS + in-memory (or sled) implementation
    └── tests/              # unit & integration tests
```

---

## 4. Usage Example

```rust
use storage::{Storage, StorageConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = StorageConfig::from_env();
    let storage = Storage::new(cfg).await?;

    // Enqueue a task
    storage.task_queue().enqueue(TaskMessage { id: 1, location: FileLocation::LocalPath("/tmp/doc.pdf".into()) }).await?;

    // Worker dequeues and processes…

    Ok(())
}
```

---

This design cleanly separates concerns, allows easy swapping of backends via configuration, and provides type-safe abstractions for intermediate state, queuing, and file handling.
