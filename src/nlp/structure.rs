//! Turns a `ParsedDocument` into a small graph fragment: a document root node,
//! one node per section, structural edges following the heading hierarchy, and
//! semantic edges between sections that share vocabulary.
//!
//! Node references are indices into `DocumentGraph::nodes`; the caller assigns
//! real database ids when persisting.

use crate::ingest::ParsedDocument;
use crate::storage::models::EdgeType;

use super::{keywords, rules};

const ROOT_TAGS: usize = 5;
const SECTION_KEYWORDS: usize = 8;
const SEMANTIC_MIN_SHARED: usize = 2;
const ROOT_SUMMARY_CHARS: usize = 400;

#[derive(Debug, Clone)]
pub struct DocNode {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DocEdge {
    /// Index into `DocumentGraph::nodes`.
    pub from: usize,
    /// Index into `DocumentGraph::nodes`.
    pub to: usize,
    pub edge_type: EdgeType,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct DocumentGraph {
    pub nodes: Vec<DocNode>,
    pub edges: Vec<DocEdge>,
}

pub fn build_document_graph(doc: &ParsedDocument) -> DocumentGraph {
    let full_text: String = doc
        .sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut nodes = vec![DocNode {
        title: doc.title.clone(),
        content: full_text.chars().take(ROOT_SUMMARY_CHARS).collect(),
        tags: keywords::extract_keywords(&full_text, ROOT_TAGS),
    }];
    let mut edges: Vec<DocEdge> = Vec::new();

    // (heading level, node index) stack; the root sits at level 0.
    let mut stack: Vec<(u8, usize)> = vec![(0, 0)];
    let mut section_keywords: Vec<(usize, Vec<String>)> = Vec::new();

    for section in &doc.sections {
        let idx = nodes.len();
        let tags = rules::derive_tags(&section.heading, &section.content);
        nodes.push(DocNode {
            title: if section.heading.is_empty() {
                format!("Section {idx}")
            } else {
                section.heading.clone()
            },
            content: section.content.clone(),
            tags,
        });

        while stack.len() > 1 && stack.last().unwrap().0 >= section.level {
            stack.pop();
        }
        let parent = stack.last().unwrap().1;
        edges.push(DocEdge {
            from: parent,
            to: idx,
            edge_type: EdgeType::Structural,
            weight: 1.0,
        });
        stack.push((section.level, idx));

        let kw_text = format!("{} {}", section.heading, section.content);
        section_keywords.push((idx, keywords::extract_keywords(&kw_text, SECTION_KEYWORDS)));
    }

    // Semantic edges between sections sharing enough vocabulary.
    for i in 0..section_keywords.len() {
        for j in (i + 1)..section_keywords.len() {
            let (ai, a_kw) = &section_keywords[i];
            let (bi, b_kw) = &section_keywords[j];
            let shared = a_kw.iter().filter(|k| b_kw.contains(k)).count();
            if shared >= SEMANTIC_MIN_SHARED && !connected(&edges, *ai, *bi) {
                edges.push(DocEdge {
                    from: *ai,
                    to: *bi,
                    edge_type: EdgeType::Semantic,
                    weight: shared as f32 * 0.5,
                });
            }
        }
    }

    DocumentGraph { nodes, edges }
}

fn connected(edges: &[DocEdge], a: usize, b: usize) -> bool {
    edges
        .iter()
        .any(|e| (e.from == a && e.to == b) || (e.from == b && e.to == a))
}
