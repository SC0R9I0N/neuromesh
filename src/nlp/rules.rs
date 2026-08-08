//! Rule-based tagging: cheap content heuristics plus top keywords as topic tags.

use super::keywords;

const MAX_TAGS: usize = 6;
const TOPIC_KEYWORDS: usize = 4;

/// Derives tags for one section: category rules first (code/math/task/reference/
/// question), then top keywords as topic tags. Order matters — the first tag
/// drives the node color in the graph view.
pub fn derive_tags(heading: &str, content: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    let lower = content.to_lowercase();

    if content.contains("```") {
        tags.push("code".to_string());
    }
    if content.contains("\\begin{") || content.contains('$') {
        tags.push("math".to_string());
    }
    if lower.contains("todo") || lower.contains("fixme") || content.contains("- [ ]") {
        tags.push("task".to_string());
    }
    if lower.contains("http://") || lower.contains("https://") {
        tags.push("reference".to_string());
    }
    if heading.trim_end().ends_with('?') {
        tags.push("question".to_string());
    }

    let combined = format!("{heading} {content}");
    for kw in keywords::extract_keywords(&combined, TOPIC_KEYWORDS) {
        if !tags.contains(&kw) {
            tags.push(kw);
        }
    }
    tags.truncate(MAX_TAGS);
    tags
}
