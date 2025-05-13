use std::time::Duration;

use tokio::time::sleep;

use crate::logic::{get_file_task_from_queue, get_local_store, update_task_data};
use crate::processing::cheaply_process_pdf_path;
use crate::types::{DocStatus, FileStoreImplementation, ProcessingStage};

/// Start the worker that continuously processes PDF tasks from the queue.
pub async fn start_worker() {
    loop {
        // Try to get a task from the queue
        match get_file_task_from_queue().await {
            Some(mut status) => {
                let result = process_pdf_from_status(status).await
                if let Err(err) = result {
                    println!("Encountered error while processing pdf: {}",err)
                }
            }
            None => {
                // No tasks available, sleep briefly
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_pdf_from_status(mut status: DocStatus) -> Result<(), String> {
    // Download the file
    let task_id = status.request_id;
    status.status = ProcessingStage::Processing;
    if let Err(err) = update_task_data(status.clone()).await {
        return Err(format!(
            "Failed to set status to Processing for task {}: {}",
            task_id, err
        ));
    }

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
        Err(err) => {
            status.error = Some(err.to_string());
            status.status = ProcessingStage::Errored;
            update_task_data(status).await;
            return Err(err.to_string())
        }
    };

    // Update status based on processing result
    match markdown_res {
        Ok(markdown) => {
            status.markdown = Some(markdown);
            status.status = ProcessingStage::Completed;
            update_task_data(status).await;
            return Ok(())
        }
        Err(err) => {
            status.error = Some(err.clone());
            status.status = ProcessingStage::Errored;
            update_task_data(status).await;
            return Err(err)
        }
    }
}
