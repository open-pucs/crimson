use std::collections::HashMap;

use axum::Json;
use axum::extract::Multipart;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
#[derive(ToSchema, Serialize, Deserialize, Debug, PartialEq)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}

// PDF Processing Data.
#[derive(ToSchema, Deserialize, Serialize)]
struct PdfProcessingInfo {
    id: Uuid, // Figure out what a proper type for a uuid in rust is.
    process_stage: ProcessingStage,
}

/// Ingest a PDF file via multipart/form-data
#[utoipa::path(
    post,
    path = "/api/ingest",
    request_body(content = DocIngestParams, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "PDF successfully ingested", body = String)
    ),
     tag = super::DOCS_TAG
)]
async fn pdf_ingest(mut multipart: Multipart) -> Json<PdfProcessingInfo> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(name) = field.name() {
            if name == "file" {
                let _bytes = field.bytes().await.expect("should be bytes for pdf field");
                break;
            }
        }
    }


    let task_id = Uuid::new_v4()
    Json(PdfProcessingInfo {
        id: Uuid::new_v4(),
        process_stage: ProcessingStage::Waiting,
    })
}

/// expose the Customer OpenAPI to parent module
pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(pdf_ingest))
}

/// Parameters for ingesting a document.
#[derive(Deserialize, ToSchema, Debug)]
pub struct DocIngestParams {
    /// File to process, sent as binary.
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
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

/// Response when a document ingestion is initiated.
#[derive(Serialize, ToSchema, Debug)]
pub struct DocIngestResponse {
    /// Unique request ID for polling status.
    pub request_id: Uuid,
    /// URL to poll for processing status.
    pub request_check_url: String,
}

/// Response when checking document processing status or final result.
#[derive(Serialize, ToSchema, Debug)]
pub struct DocStatusResponse {
    /// Unique request ID.
    pub request_id: Uuid,
    /// Optional URL to poll for processing (none if already complete).
    pub request_check_url: Option<String>,
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

// use axum::extract::{Multipart, Path};
// use axum::Json;
// use serde::Deserialize;
// use utoipa_axum::router::OpenApiRouter;
// use utoipa_axum::routes;
// use uuid::Uuid;
//
// use crate::doc_processing::{check_document_status, ingest_document};
// use crate::docs_api::{DocIngestParams, DocIngestResponse, DocStatusResponse};
//
// /// Ingest a document via multipart/form-data
// #[utoipa::path(
//     post,
//     path = "/api/v1/docs",
//     request_body(content = DocIngestParams, content_type = "multipart/form-data"),
//     responses((status = 200, description = "Document ingestion initiated", body = DocIngestResponse)),
//     tag = crate::DOCS_TAG
// )]
// async fn doc_ingest(mut multipart: Multipart) -> Json<DocIngestResponse> {
//     // Parse multipart fields
//     let mut file: Option<Vec<u8>> = None;
//     let mut langs: Option<String> = None;
//     let mut force_ocr: Option<bool> = None;
//     let mut paginate: Option<bool> = None;
//     let mut output_format: Option<String> = None;
//     let mut use_llm: Option<bool> = None;
//     let mut strip_existing_ocr: Option<bool> = None;
//     let mut disable_image_extraction: Option<bool> = None;
//     let mut max_pages: Option<u32> = None;
//
//     while let Some(field) = multipart.next_field().await.unwrap() {
//         let name = field.name().unwrap_or_default();
//         match name {
//             "file" => file = Some(field.bytes().await.unwrap().to_vec()),
//             "langs" => langs = Some(field.text().await.unwrap()),
//             "force_ocr" => force_ocr = field.text().await.unwrap().parse().ok(),
//             "paginate" => paginate = field.text().await.unwrap().parse().ok(),
//             "output_format" => output_format = Some(field.text().await.unwrap()),
//             "use_llm" => use_llm = field.text().await.unwrap().parse().ok(),
//             "strip_existing_ocr" => strip_existing_ocr = field.text().await.unwrap().parse().ok(),
//             "disable_image_extraction" => {
//                 disable_image_extraction = field.text().await.unwrap().parse().ok()
//             }
//             "max_pages" => max_pages = field.text().await.unwrap().parse().ok(),
//             _ => {}
//         }
//     }
//
//     let params = DocIngestParams {
//         file: file.unwrap_or_default(),
//         langs,
//         force_ocr,
//         paginate,
//         output_format: output_format.and_then(|s| s.parse().ok()),
//         use_llm,
//         strip_existing_ocr,
//         disable_image_extraction,
//         max_pages,
//     };
//
//     Json(ingest_document(params).await)
// }
//
// /// Check document processing status or get result
// #[utoipa::path(
//     get,
//     path = "/api/v1/docs/{request_id}",
//     params(("request_id" = Uuid, description = "Unique request ID")),
//     responses((status = 200, description = "Document processing status", body = DocStatusResponse)),
//     tag = crate::DOCS_TAG
// )]
// async fn check_status(Path(request_id): Path<Uuid>) -> Json<DocStatusResponse> {
//     Json(check_document_status(request_id).await)
// }
//
// /// Docs module router
// pub fn router() -> OpenApiRouter {
//     OpenApiRouter::new().routes(routes!(doc_ingest, check_status))
// }
