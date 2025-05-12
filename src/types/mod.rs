use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
mod storage;

pub type TaskID = u64;

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum FileLocation {
    S3Uri(String),
    LocalPath(String),
}
#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Clone, Copy)]
pub enum ProcessingStage {
    Completed,
    Waiting,
    Errored,
    Processing,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct DocStatusResponse {
    request_id: TaskID,
    request_check_url: String,
    markdown: Option<String>,
    status: ProcessingStage,
    success: bool,
    images: Option<HashMap<String, String>>,
    metadata: Option<HashMap<String, String>>,
    error: Option<String>,
}
fn make_request_url(id: TaskID) -> String {
    "/v1/status/".to_string() + &id.to_string()
}

#[derive(Debug, Clone)]
pub struct DocStatus {
    file_location: FileLocation,
    request_id: TaskID,
    markdown: Option<String>,
    status: ProcessingStage,
    images: Option<HashMap<String, String>>,
    metadata: Option<HashMap<String, String>>,
    error: Option<String>,
}

impl From<DocStatus> for DocStatusResponse {
    fn from(input: DocStatus) -> Self {
        // let err_str = self.error.map(|val| val.to_string());
        DocStatusResponse {
            request_id: input.request_id,
            request_check_url: make_request_url(input.request_id),
            markdown: input.markdown,
            status: input.status,
            success: input.status == ProcessingStage::Completed,
            images: input.images,
            metadata: input.metadata,
            error: input.error,
        }
    }
}
pub fn make_new_docstatus(id: TaskID, location: FileLocation) -> DocStatus {
    DocStatus {
        file_location: location,
        request_id: id,
        markdown: None,
        status: ProcessingStage::Waiting,
        metadata: None,
        images: None,
        error: None,
    }
}
