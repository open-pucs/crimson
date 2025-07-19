use std::time::Duration;

use anyhow::{anyhow, bail};
use tokio::time::sleep;

use crate::logic::{get_file_task_from_queue, get_local_store, update_task_data};
use crate::processing::{cheaply_process_pdf_path, process_marker_pdf};
use crate::types::{DocStatus, FileStoreImplementation, MarkdownConversionMethod, ProcessingStage};
use tracing::{error, info};

/// Start the worker that continuously processes PDF tasks from the queue.
pub async fn start_worker() {
    info!("Starting pdf processing worker.");
    loop {
        // Try to get a task from the queue
        // This is multithreaded, so I assume only one instance is enough to keep itself busy,
        // might want to add a semaphore if it needs more work though.
        match get_file_task_from_queue().await {
            Some(status) => {
                let result = process_pdf_from_status(status).await;
                if let Err(err) = result {
                    error!("Encountered error while processing pdf: {}", err);
                }
            }
            None => {
                // No tasks available, sleep briefly
                info!("No tasks detected, sleeping for 2 seconds.");
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn process_pdf_from_status(mut status: DocStatus) -> anyhow::Result<()> {
    async fn task_errored(mut status: DocStatus, err: anyhow::Error) -> anyhow::Error {
        status.error = Some("Encountered error: ".to_string() + &err.to_string());
        status.status = ProcessingStage::Errored;
        let _ = update_task_data(status).await;
        err
    }
    // Download the file
    let task_id = status.request_id;
    status.status = ProcessingStage::Processing;
    if let Err(err) = update_task_data(status.clone()).await {
        bail!("Failed to set status to Processing for task {task_id}: {err}",);
    }
    info!(task_id, "Updated document to processing stage.");

    let store = get_local_store();
    let download_result = store
        .file_store
        .download_to_file(&status.file_location)
        .await;
    if let Err(err) = download_result {
        return Err(task_errored(status, err.into()).await);
    }
    let local_path = download_result.unwrap();

    // Process PDF to markdown
    info!(
        local_path=%local_path.to_string_lossy(),
        "Downloaded result successfully, processing pdf on locally",
    );
    let markdown_res = match status.conversion_method {
        MarkdownConversionMethod::Simple => cheaply_process_pdf_path(&local_path),
        MarkdownConversionMethod::Marker => process_marker_pdf(&local_path).await,
    };

    // Update status based on processing result
    match markdown_res {
        Ok(markdown) => {
            status.markdown = Some(markdown);
            status.status = ProcessingStage::Completed;
            info!(task_id, "Successfully processed pdf");
            match update_task_data(status).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    bail!(
                        "Encountered error pushing final data to db: ".to_string()
                            + &err.to_string()
                    )
                }
            }
        }
        Err(err) => {
            tracing::error!(%err,task_id,"Encountered error processing pdf");
            Err(task_errored(status, anyhow!("Encountered error processing pdf: {err}")).await)
        }
    }
}
