//! LaTeX parser — stub for the MVP.
//!
//! Planned implementation: line-based scan for `\section{…}` / `\subsection{…}`
//! / `\subsubsection{…}` mapped to section levels 1–3, stripping common commands
//! from the body text.

use std::path::Path;

use anyhow::Result;

use super::ParsedDocument;

pub fn parse(path: &Path) -> Result<ParsedDocument> {
    anyhow::bail!(
        "LaTeX ingestion is not implemented yet ({}). Planned for a future milestone.",
        path.display()
    )
}
