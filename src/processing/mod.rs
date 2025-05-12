use markdownify::{docx, pdf};

use markdownify::{docx, pdf};
use std::path::Path;

fn process_pdf_path(path: Path) -> Result<String, Box<dyn std::error::Error>> {
    let markdown = pdf::pdf_convert(&path)?;
    return Ok(markdown);
}
