pub mod worker;

use std::path::Path;

use markdownify::pdf;

/// Convert a PDF at the given path to Markdown string.
/// Returns Err(String) on failure.
pub fn cheaply_process_pdf_path(path: &Path) -> Result<String, String> {
    pdf::pdf_convert(path).map_err(|e| e.to_string())
}