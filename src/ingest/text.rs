//! Plain-text parser: splits on blank lines and groups paragraphs into
//! ~700-character sections so long files become several graph nodes instead of one.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{file_stem, ParsedDocument, Section};
use crate::storage::models::FileType;

const CHUNK_TARGET_BYTES: usize = 700;

pub fn parse(path: &Path) -> Result<ParsedDocument> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let raw = raw.replace("\r\n", "\n");
    let stem = file_stem(path);

    let mut sections = Vec::new();
    let mut buf = String::new();
    for para in raw.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        if !buf.is_empty() {
            buf.push_str("\n\n");
        }
        buf.push_str(para);
        if buf.len() >= CHUNK_TARGET_BYTES {
            sections.push(make_section(&buf));
            buf.clear();
        }
    }
    if !buf.is_empty() {
        sections.push(make_section(&buf));
    }
    if sections.is_empty() {
        sections.push(Section {
            heading: stem.clone(),
            level: 1,
            content: String::new(),
        });
    }

    Ok(ParsedDocument {
        title: stem,
        file_type: FileType::Txt,
        sections,
    })
}

fn make_section(content: &str) -> Section {
    let first_line = content.lines().next().unwrap_or("").trim();
    let heading = if first_line.chars().count() > 60 {
        let mut h: String = first_line.chars().take(57).collect();
        h.push('…');
        h
    } else {
        first_line.to_string()
    };
    Section {
        heading,
        level: 1,
        content: content.to_string(),
    }
}
