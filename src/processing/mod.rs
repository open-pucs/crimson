use std::path::Path;

use markdownify::{docx, pdf};

fn cheaply_process_pdf_path(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let markdown = pdf::pdf_convert(path)?;
    return Ok(markdown);
}
