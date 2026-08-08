//! Core data models shared across the application, mirroring the SQLite schema.

use std::path::Path;

/// A single concept in the knowledge graph, usually one section of a document.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source_file_id: i64,
    /// Persisted layout position; `None` until the force layout settles once.
    pub position: Option<(f32, f32)>,
}

/// How two nodes are related.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Document hierarchy: parent section → child section.
    Structural,
    /// Shared vocabulary detected by keyword analysis.
    Semantic,
    /// Shared tags across documents.
    TagRelation,
    /// Created by the user.
    Manual,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Structural => "structural",
            EdgeType::Semantic => "semantic",
            EdgeType::TagRelation => "tag_relation",
            EdgeType::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "structural" => EdgeType::Structural,
            "semantic" => EdgeType::Semantic,
            "tag_relation" => EdgeType::TagRelation,
            _ => EdgeType::Manual,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    /// Database id; kept for the data model even though rendering ignores it.
    #[allow(dead_code)]
    pub id: i64,
    pub from_id: i64,
    pub to_id: i64,
    pub edge_type: EdgeType,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Txt,
    Md,
    Pdf,
    Docx,
    Latex,
    Other(String),
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("txt") | Some("text") => FileType::Txt,
            Some("md") | Some("markdown") => FileType::Md,
            Some("pdf") => FileType::Pdf,
            Some("docx") => FileType::Docx,
            Some("tex") | Some("latex") => FileType::Latex,
            Some(other) => FileType::Other(other.to_string()),
            None => FileType::Other(String::new()),
        }
    }

    pub fn to_db_string(&self) -> String {
        match self {
            FileType::Txt => "txt".to_string(),
            FileType::Md => "md".to_string(),
            FileType::Pdf => "pdf".to_string(),
            FileType::Docx => "docx".to_string(),
            FileType::Latex => "latex".to_string(),
            FileType::Other(ext) => format!("other:{ext}"),
        }
    }

    pub fn from_db_string(s: &str) -> Self {
        match s {
            "txt" => FileType::Txt,
            "md" => FileType::Md,
            "pdf" => FileType::Pdf,
            "docx" => FileType::Docx,
            "latex" => FileType::Latex,
            other => FileType::Other(other.strip_prefix("other:").unwrap_or(other).to_string()),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FileType::Txt => "📄",
            FileType::Md => "📝",
            FileType::Pdf => "📕",
            FileType::Docx => "📘",
            FileType::Latex => "📐",
            FileType::Other(_) => "❔",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub file_type: FileType,
    pub title: String,
    pub imported_at: String,
}
