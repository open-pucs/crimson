Currently this api documentation library utoipa is turning out to be a lot more trouble then its worth. Could you comment out (not delete) the existing API documentation and just route everything using vanilla axum?

### 2.2 Local Filesystem + In-Memory Backend

- **FileStore**: Stores files under a configured `base_path` on disk.  
- **TaskQueue**: In-memory `VecDeque<TaskMessage>` protected by `tokio::sync::Mutex`.  
- **MetadataStore**: Either in-memory `HashMap<TaskID, ...>` with `Mutex` .

Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes.

Also you can look up documentation for popular rust libraries like tokio, serde and axum by using the context7 tool, its support on less popular libraries is limited unfortunately..
