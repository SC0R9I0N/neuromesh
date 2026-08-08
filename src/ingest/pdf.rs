//! PDF parser — stub for the MVP.
//!
//! Planned implementation: extract text with a pure-Rust crate (`pdf-extract` or
//! `lopdf`), then reuse the plain-text chunking from `text.rs` on the result.

use std::path::Path;

use anyhow::Result;

use super::ParsedDocument;

pub fn parse(path: &Path) -> Result<ParsedDocument> {
    anyhow::bail!(
        "PDF ingestion is not implemented yet ({}). Planned for a future milestone.",
        path.display()
    )
}
