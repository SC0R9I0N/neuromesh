//! File ingestion pipeline: detects the file type and routes to the matching
//! parser, producing a normalized `ParsedDocument` for the NLP stage.

pub mod docx;
pub mod latex;
pub mod markdown;
pub mod pdf;
pub mod text;

use std::path::Path;

use anyhow::Result;

use crate::storage::models::FileType;

/// Unified internal representation every parser produces.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub file_type: FileType,
    pub sections: Vec<Section>,
}

/// One coherent chunk of a document (a heading + its body, or a text chunk).
#[derive(Debug, Clone)]
pub struct Section {
    pub heading: String,
    /// Hierarchy depth (1 = top level). Drives structural edges.
    pub level: u8,
    pub content: String,
}

/// Detects the file type and dispatches to the right parser.
/// Unknown extensions fall back to the plain-text parser.
pub fn parse_file(path: &Path) -> Result<ParsedDocument> {
    let file_type = FileType::from_path(path);
    let mut doc = match &file_type {
        FileType::Txt => text::parse(path)?,
        FileType::Md => markdown::parse(path)?,
        FileType::Pdf => pdf::parse(path)?,
        FileType::Docx => docx::parse(path)?,
        FileType::Latex => latex::parse(path)?,
        FileType::Other(_) => text::parse(path)?,
    };
    doc.file_type = file_type;
    Ok(doc)
}

pub(crate) fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}
