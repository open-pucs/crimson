pub mod worker;

use std::path::Path;

use markdownify::pdf;

/// Convert a PDF at the given path to Markdown string.
/// Returns Err(String) on failure.
pub fn cheaply_process_pdf_path(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Check if path exists
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("File access error: {} (path: {:?})", e, path))?;

    // Verify it's a regular file
    if !metadata.is_file() {
        return Err(format!("Path is not a file: {:?}", path).into());
    }

    // // Check file extension
    // if path.extension().and_then(|s| s.to_str()) != Some("pdf") {
    //     return Err(format!("Invalid file extension for path: {:?}", path).into());
    // }

    pdf::pdf_convert(path)
}
