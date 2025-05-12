use std::collections::HashMap;

use aide::axum::routing::{get, post};
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::Json;
use axum::extract::{Multipart, Path};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
// use utoipa::ToSchema;
// use utoipa_axum::router::OpenApiRouter;
// use utoipa_axum::routes;
// use uuid::Uuid;
use rand::prelude::*;

type TaskID = u64;

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}

/// Ingest a PDF file via multipart/form-data
// #[utoipa::path(
//     post,
//     path = "/ingest",
//     request_body(content = DocIngestParams, content_type = "multipart/form-data"),
//     responses(
//         (status = 200, description = "PDF successfully ingested", body = DocStatusResponse)
//     ),
//     tag = super::DOCS_TAG
// )]
async fn pdf_ingest(mut multipart: Multipart) -> Json<DocStatusResponse> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(name) = field.name() {
            if name == "file" {
                let _bytes = field.bytes().await.expect("should be bytes for pdf field");
                break;
            }
        }
    }

    let task_id: TaskID = {
        let mut f = File::open("/dev/urandom").expect("Failed to open /dev/urandom");
        let mut bytes = [0u8; 8];
        f.read_exact(&mut bytes)
            .expect("Failed to read random bytes");
        u64::from_ne_bytes(bytes)
    };
    // let task_id = Uuid::new_v4();
    // TODO: send to processing queue

    Json(make_default_response(task_id))
}

/// Get PDF status by task_id
// #[utoipa::path(
//     get,
//     path = "/status/{task_id}",
//     responses(
//         (status = 200, description = "Got PDF Status", body = DocStatusResponse)
//     ),
//     params(
//         ("task_id" = Uuid, Path, description = "Task ID for PDF."),
//     ),
//     tag = super::DOCS_TAG
// )]
async fn pdf_get_status(Path(task_id): Path<TaskID>) -> impl IntoApiResponse {
    Json(make_default_response(task_id))
}

/// Docs module router
pub fn router() -> ApiRouter {
    ApiRouter::new()
        // .api_route("/ingest", post(pdf_ingest))
        .api_route("/status/{task_id}", get(pdf_get_status))
        .api_route("/ingest", post(pdf_ingest))
}

/// Parameters for ingesting a document.
#[derive(Deserialize, Debug, JsonSchema)]
pub struct DocIngestParams {
    /// File to process, sent as binary.
    // #[schema(format = Binary, content_media_type = "application/octet-stream")]
    pub file: Vec<u8>,
    /// Optional comma-separated list of languages for OCR (e.g., "en,fr").
    pub langs: Option<String>,
    /// Force OCR on every page.
    pub force_ocr: Option<bool>,
    /// Paginate output with page delimiters.
    pub paginate: Option<bool>,
    /// Disable image extraction.
    pub disable_image_extraction: Option<bool>,
    /// Maximum number of pages to process from the start.
    pub max_pages: Option<u32>,
}

/// Response when checking document processing status or final result.
#[derive(Serialize, Debug, JsonSchema)]
pub struct DocStatusResponse {
    /// Unique request ID.
    pub request_id: TaskID,
    /// Optional URL to poll for processing (none if already complete).
    pub request_check_url: String,
    /// Markdown output (if requested).
    pub markdown: Option<String>,
    /// Current processing stage.
    pub status: ProcessingStage,
    /// Indicates if processing was successful.
    pub success: bool,
    /// Map of image filenames to base64-encoded data.
    pub images: Option<HashMap<String, String>>,
    /// Metadata about the processed document.
    pub metadata: Option<HashMap<String, String>>,
    /// Error message if processing failed.
    pub error: Option<String>,
    /// Number of pages processed.
    pub page_count: Option<u32>,
}

fn make_request_url(id: TaskID) -> String {
    "/test".to_string() + &id.to_string()
}

fn make_default_response(id: TaskID) -> DocStatusResponse {
    DocStatusResponse {
        request_id: id,
        request_check_url: make_request_url(id),
        markdown: None,
        status: ProcessingStage::Waiting,
        success: false,
        metadata: None,
        images: None,
        error: None,
        page_count: None,
    }
}

// The following Utoipa-based documentation has been commented out:
// use axum::extract::{Multipart, Path};
// use utoipa_axum::router::OpenApiRouter;
// use utoipa_axum::routes;
// ...
