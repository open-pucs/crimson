Given all the interfaces that currently exist in src/types/mod.rs

and this local file store defined here 

### 2.2 Local Filesystem + In-Memory Backend

- **FileStore**: Stores files under a configured `base_path` on disk.  
- **TaskQueue**: In-memory `VecDeque<TaskMessage>` protected by `tokio::sync::Mutex`.  
- **MetadataStore**: Either in-memory `HashMap<TaskID, ...>` with `Mutex` .

All of this already got implemented in logic/local_store.rs

At the start of the program in main, could you go ahead and make a local store that exists as a static variable, that could then be accessible from the accesser functions in mod.rs?




Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes.

Also you can look up documentation for popular rust libraries like tokio, serde and axum by using the context7 tool, its support on less popular libraries is limited unfortunately. Whenever you are stuck with some inscrutable errors, it can be helpful to look up examples to see how the code should be structured.

