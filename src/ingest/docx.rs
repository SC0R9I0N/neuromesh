//! DOCX parser — stub for the MVP.
//!
//! Planned implementation: unzip the OOXML container, parse `word/document.xml`
//! with a lightweight XML reader (`quick-xml`), and map Heading styles to
//! `Section` levels.

use std::path::Path;

use anyhow::Result;

use super::ParsedDocument;

pub fn parse(path: &Path) -> Result<ParsedDocument> {
    anyhow::bail!(
        "DOCX ingestion is not implemented yet ({}). Planned for a future milestone.",
        path.display()
    )
}
