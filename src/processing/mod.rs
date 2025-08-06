pub mod worker;

use std::path::Path;

use anyhow::{anyhow, bail};
use markdownify::pdf;

use crate::types::MarkdownConversionMethod;

pub async fn process_pdf(
    local_path: &str,
    method: &MarkdownConversionMethod,
) -> anyhow::Result<String> {
    match method {
        MarkdownConversionMethod::Simple => cheaply_process_pdf_path(local_path.as_ref()),
        MarkdownConversionMethod::Marker => process_marker_pdf(local_path.as_ref()).await,
        MarkdownConversionMethod::OlmOcr => olmocr_deepinfra_process(local_path).await,
    }
}

pub async fn olmocr_deepinfra_process(local_path: &str) -> anyhow::Result<String> {
    todo!()
}

/// Convert a PDF at the given path to Markdown string.
/// Returns Err(String) on failure.
pub fn cheaply_process_pdf_path(path: &Path) -> anyhow::Result<String> {
    // Check if path exists
    let metadata = std::fs::metadata(path)
        .map_err(|e| anyhow!("File access error: {} (path: {:?})", e, path))?;

    // Verify it's a regular file
    if !metadata.is_file() {
        bail!("Path is not a file: {:?}", path);
    }

    // // Check file extension
    // if path.extension().and_then(|s| s.to_str()) != Some("pdf") {
    //     return Err(format!("Invalid file extension for path: {:?}", path).into());
    // }
    let pdf_result = pdf::pdf_convert(path);
    match pdf_result {
        Ok(path) => Ok(path),
        Err(err) => bail!("Encountered markdownify error: {err}"),
    }
}

pub async fn process_marker_pdf(_path: &Path) -> anyhow::Result<String> {
    bail!("Marker PDF Processing Not Implemented")
}
