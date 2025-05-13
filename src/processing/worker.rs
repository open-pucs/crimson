use std::time::Duration;

use tokio::time::sleep;

use crate::logic::{get_file_task_from_queue, get_local_store, update_task_data};
use crate::processing::cheaply_process_pdf_path;
use crate::types::{FileStoreImplementation, ProcessingStage};

/// Start the worker that continuously processes PDF tasks from the queue.
pub async fn start_worker() {
    loop {
        // Try to get a task from the queue
        match get_file_task_from_queue().await {
            Some(mut status) => {
                let task_id = status.request_id;
                // Update status to Processing
                status.status = ProcessingStage::Processing;
                if let Err(err) = update_task_data(status.clone()).await {
                    eprintln!(
                        "Failed to set status to Processing for task {}: {}",
                        task_id, err
                    );
                    continue;
                }

                if let Err(err) = update_task_data(final_status).await {
                    eprintln!("Failed to update status for task {}: {}", task_id, err);
                }
            }
            None => {
                // No tasks available, sleep briefly
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}


async fn process_pdf_from_status(status: DocStatus) -> Result<(), Box<dyn Error>> {
                // Download the file
                let store = get_local_store();
                let download_result = store
                    .file_store
                    .download_to_file(&status.file_location)
                    .await;

                let markdown_res = match download_result {
                    Ok(local_path) => {
                        // Process PDF to markdown
                        cheaply_process_pdf_path(&local_path)
                    }
                    Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
                };

                // Update status based on processing result
                let mut final_status = status.clone();
                match markdown_res {
                    Ok(markdown) => {
                        final_status.markdown = Some(markdown);
                        final_status.status = ProcessingStage::Completed;
                    }
                    Err(err) => {
                        final_status.error = Some(err.to_string());
                        final_status.status = ProcessingStage::Errored;
                    }
                }
}
