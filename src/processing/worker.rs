use std::time::Duration;

use tokio::time::sleep;

use crate::logic::{get_file_task_from_queue, get_local_store, update_task_data};
use crate::processing::cheaply_process_pdf_path;
use crate::types::{DocStatus, FileStoreImplementation, ProcessingStage};
use tracing::{error, info, warn};

/// Start the worker that continuously processes PDF tasks from the queue.
pub async fn start_worker() {
    loop {
        // Try to get a task from the queue
        match get_file_task_from_queue().await {
            Some(status) => {
                let result = process_pdf_from_status(status).await;
                if let Err(err) = result {
                    error!("Encountered error while processing pdf: {}", err);
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
    info!("Updated document to processing stage.");

    let store = get_local_store();
    let download_result = store
        .file_store
        .download_to_file(&status.file_location)
        .await;

    let markdown_res = match download_result {
        Ok(local_path) => {
            // Process PDF to markdown
            info!(
                "Downloaded result successfully, processing pdf at: {}",
                &local_path.to_string_lossy()
            );
            cheaply_process_pdf_path(&local_path)
        }
        Err(err) => {
            status.error =
                Some("Encountered error downloading file: ".to_string() + &err.to_string());
            status.status = ProcessingStage::Errored;
            let _ = update_task_data(status).await;
            return Err("Encountered error downloading file: ".to_string() + &err.to_string());
        }
    };

    // Update status based on processing result
    match markdown_res
        .map_err(|err| "Encountered error processing pdf: ".to_string() + &err.to_string())
    {
        Ok(markdown) => {
            status.markdown = Some(markdown);
            status.status = ProcessingStage::Completed;
            println!("Successfully processed pdf");
            match update_task_data(status).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    Err("Encountered error pushing final data to db: ".to_string()
                        + &err.to_string())
                }
            }
        }
        Err(err_str) => {
            status.error = Some(err_str.clone());
            status.status = ProcessingStage::Errored;
            let _ = update_task_data(status).await;
            Err(err_str)
        }
    }
}
