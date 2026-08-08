//! Markdown parser: line-based ATX-heading splitter (no external crate needed).
//! Headings define the section hierarchy; code fences are respected so `# comments`
//! inside fenced blocks are not mistaken for headings.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{file_stem, ParsedDocument, Section};
use crate::storage::models::FileType;

pub fn parse(path: &Path) -> Result<ParsedDocument> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let raw = raw.replace("\r\n", "\n");
    let stem = file_stem(path);

    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;
    let mut preamble = String::new();
    let mut in_fence = false;
    let mut title: Option<String> = None;

    for line in raw.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        let heading = if in_fence { None } else { parse_heading(line) };
        match heading {
            Some((level, text)) => {
                if title.is_none() && level == 1 {
                    title = Some(text.clone());
                }
                if let Some(sec) = current.take() {
                    sections.push(sec);
                }
                current = Some(Section {
                    heading: text,
                    level,
                    content: String::new(),
                });
            }
            None => {
                let target = match &mut current {
                    Some(sec) => &mut sec.content,
                    None => &mut preamble,
                };
                target.push_str(line);
                target.push('\n');
            }
        }
    }
    if let Some(sec) = current.take() {
        sections.push(sec);
    }

    let preamble = preamble.trim();
    if !preamble.is_empty() {
        sections.insert(
            0,
            Section {
                heading: "Preamble".to_string(),
                level: 1,
                content: preamble.to_string(),
            },
        );
    }
    if sections.is_empty() {
        sections.push(Section {
            heading: stem.clone(),
            level: 1,
            content: String::new(),
        });
    }
    for sec in &mut sections {
        sec.content = sec.content.trim().to_string();
    }

    Ok(ParsedDocument {
        title: title.unwrap_or(stem),
        file_type: FileType::Md,
        sections,
    })
}

/// Parses an ATX heading (`## Title`). Returns the level (1–6) and the text.
fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.starts_with(' ') {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some((hashes as u8, text))
}
