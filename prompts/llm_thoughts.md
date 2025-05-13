# LLM Implementation Thoughts

- Goal:
  - Implement component 3: worker task to process queued PDF tasks.
  - Worker should poll the task queue, update status to Processing, download file, process PDF to markdown, update status to Completed or Errored.
  - Must integrate with existing logic/local_store and processing functions.
  - Initialize worker in main function.

- Steps:
  1. Expose `cheaply_process_pdf_path` as `pub` in processing/mod.rs for use by worker.
  2. Create new module `src/processing/worker.rs` defining async `start_worker` function with infinite loop:
     - Dequeue task: use `logic::get_file_task_from_queue`.
     - Update status to Processing using `logic::update_task_data`.
     - Download document: `get_local_store().file_store.download_to_file`.
     - Convert to markdown: `crate::processing::cheaply_process_pdf_path`.
     - Update DocStatus fields and set proper ProcessingStage.
     - Persist updated status.
     - On no tasks, sleep for a short duration.
  3. Modify `src/processing/mod.rs` to `pub mod worker` and `pub fn cheaply_process_pdf_path`.
  4. Adjust `src/main.rs` to `use processing::worker` and spawn `worker::start_worker()` using `tokio::spawn` before starting the HTTP server.
  5. Run `cargo check` to verify correctness and fix any errors.

- Potential Issues:
  - Thread safety: Using Tokio runtime, logic functions are already async and thread-safe.
  - Download path correctness: `LocalFileStore.download_to_file` returns a `LocalPath`; passing directly to processing.
  - Error handling: Ensure errors during download or processing mark status as Errored and include error messages.

- Next Steps:
  - Implement code modifications.
  - Test with mock tasks.
  - Iterate based on `cargo check` feedback.
