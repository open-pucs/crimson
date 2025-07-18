use aide::axum::ApiRouter;
use aide::axum::routing::{get, post};
use axum::Json;
use axum::extract::{Multipart, Path};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::logic::{get_task_data_from_id, ingest_file_to_queue};
use crate::types::{
    DocStatus, DocStatusResponse, FileLocation, MarkdownConversionMethod, S3Location, StoreError,
    TaskID,
};

async fn pdf_ingest(mut multipart: Multipart) -> Result<Json<DocStatusResponse>, String> {
    let task_id: TaskID = make_task_id();
    while let Some(field) = multipart.next_field().await.unwrap()
        && let Some(name) = field.name()
    {
        if name == "file" {
            let _bytes = field.bytes().await.expect("should be bytes for pdf field");
            let file_path = "/tmp/save/to/this/path".to_string() + &task_id.to_string();
            let file_location = FileLocation::LocalPath(file_path.into());
            let task_status = DocStatus::new_from_id_loc(
                task_id,
                file_location,
                MarkdownConversionMethod::default(),
            );
            ingest_file_to_queue(task_status.clone()).await;
            return Ok(Json(task_status.into()));
        }
    }
    Err("Multipart was improperely formatted".into())
}
async fn pdf_ingest_s3(
    Json(ingest_params): Json<DocIngestParamsS3>,
) -> Result<Json<DocStatusResponse>, String> {
    let task_id: TaskID = make_task_id();
    let s3_result: Result<S3Location, StoreError> = ingest_params.s3_uri.try_into();
    let file_location = FileLocation::S3Location(s3_result.map_err(|err| err.to_string())?);
    let conversion_method = ingest_params.conversion_method.unwrap_or_default();
    let task_status = DocStatus::new_from_id_loc(task_id, file_location, conversion_method);
    ingest_file_to_queue(task_status.clone()).await;
    Ok(Json(task_status.into()))
}

async fn pdf_ingest_debug_local_path(
    Json(ingest_params): Json<DocIngestParamsDebugLocalPath>,
) -> Result<Json<DocStatusResponse>, String> {
    let task_id: TaskID = make_task_id();
    let file_location = FileLocation::LocalPath(ingest_params.local_path);
    let conversion_method = ingest_params.conversion_method.unwrap_or_default();
    let task_status = DocStatus::new_from_id_loc(task_id, file_location, conversion_method);
    ingest_file_to_queue(task_status.clone()).await;
    Ok(Json(task_status.into()))
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
struct TaskIDParams {
    task_id: TaskID,
}

async fn pdf_get_status(
    Path(TaskIDParams { task_id }): Path<TaskIDParams>,
) -> Json<DocStatusResponse> {
    Json(get_task_data_from_id(task_id).await.unwrap().into())
}

/// Docs module router
pub fn router() -> ApiRouter {
    ApiRouter::new()
        // .api_route("/ingest", post(pdf_ingest))
        .api_route("/status/{task_id}", get(pdf_get_status))
        .api_route("/ingest/upload", post(pdf_ingest))
        .api_route("/ingest/s3", post(pdf_ingest_s3))
        .api_route(
            "/ingest/debug_local_path",
            post(pdf_ingest_debug_local_path),
        )
}

/// Parameters for ingesting a document.
// #[derive(Deserialize, Debug, JsonSchema)]
// struct DocIngestParams {
//     /// File to process, sent as binary.
//     // #[schema(format = Binary, content_media_type = "application/octet-stream")]
//     pub file: Vec<u8>,
//     /// Optional comma-separated list of languages for OCR (e.g., "en,fr").
//     pub langs: Option<String>,
//     /// Force OCR on every page.
//     pub force_ocr: Option<bool>,
//     /// Paginate output with page delimiters.
//     pub paginate: Option<bool>,
//     /// Disable image extraction.
//     pub disable_image_extraction: Option<bool>,
//     /// Maximum number of pages to process from the start.
//     pub max_pages: Option<u32>,
// }

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct DocIngestParamsS3 {
    /// File to process, sent as binary.
    // #[schema(format = Binary, content_media_type = "application/octet-stream")]
    pub s3_uri: String,
    /// Optional comma-separated list of languages for OCR (e.g., "en,fr").
    pub langs: Option<String>,

    /// What method do you want to use to convert the markdown
    pub conversion_method: Option<MarkdownConversionMethod>,

    /// Force OCR on every page.
    pub force_ocr: Option<bool>,
    /// Paginate output with page delimiters.
    pub paginate: Option<bool>,
    /// Disable image extraction.
    pub disable_image_extraction: Option<bool>,
    /// Maximum number of pages to process from the start.
    pub max_pages: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct DocIngestParamsDebugLocalPath {
    /// File to process, sent as binary.
    // #[schema(format = Binary, content_media_type = "application/octet-stream")]
    pub local_path: PathBuf,
    /// Optional comma-separated list of languages for OCR (e.g., "en,fr").
    pub langs: Option<String>,
    /// Force OCR on every page.
    pub force_ocr: Option<bool>,
    /// What method do you want to use to convert the markdown
    pub conversion_method: Option<MarkdownConversionMethod>,
    /// Paginate output with page delimiters.
    pub paginate: Option<bool>,
    /// Disable image extraction.
    pub disable_image_extraction: Option<bool>,
    /// Maximum number of pages to process from the start.
    pub max_pages: Option<u32>,
}
// Logic For document processing.

fn make_task_id() -> TaskID {
    let mut f = File::open("/dev/urandom").expect("Failed to open /dev/urandom");
    let mut bytes = [0u8; 8];
    f.read_exact(&mut bytes)
        .expect("Failed to read random bytes");
    u64::from_ne_bytes(bytes)
}
