use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type TaskID = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileLocation {
    S3Uri(String),
    LocalPath(String),
}
#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema, Clone, Copy)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct DocStatus {
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
    "/v1/status/".to_string() + &id.to_string()
}

pub fn make_default_response(id: TaskID) -> DocStatus {
    DocStatus {
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
