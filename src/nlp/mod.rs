//! NLP & categorization — Phase 1: pure rule-based, zero dependencies, instant.
//! Phase 2 (optional, future): Python bridge via PyO3 in `python_bridge.rs`.

pub mod keywords;
pub mod python_bridge;
pub mod rules;
pub mod structure;

pub use structure::build_document_graph as analyze;
