# NeuroMesh – Local Knowledge Graph “Brain” Application
A fully offline, native Rust desktop application that ingests notes (txt, md, pdf, docx, LaTeX, etc.), auto-categorizes them, and visualizes them as an interactive knowledge graph. Designed for maximum performance, minimal footprint, and clean modular architecture.

## 1. Project Overview
Name: NeuroMesh
Type: Native, local-first desktop application (no browser, no Electron)
Primary Goals:
- Ingest new and existing notes in multiple formats
- Parse and normalize content into a unified internal representation
- Automatically categorize and connect concepts into a knowledge graph
- Render an interactive “brain map” GUI
- Keep everything local, fast, and lightweight

Design Priorities:
- Performance (Rust, GPU-accelerated rendering)
- Lightweight (no web engine, small binary)
- Offline (all processing on device)
- Extensible (clear module boundaries)

## 2. Technical Decisions (Optimized for Performance + Lightweight Footprint)

2.1 Language & Runtime
- Rust (native binary, no VM, no browser)

2.2 UI Framework
- egui (via eframe)
Chosen for:
- GPU acceleration
- Immediate-mode simplicity
- Cross-platform
- Lightweight footprint
- Easy custom graph rendering

2.3 Rendering Engine
- egui Painter for MVP
- Upgrade path to wgpu if graph sizes exceed ~5k nodes
This gives the best performance-to-complexity ratio.

2.4 Storage Layer
- SQLite + rusqlite
Reasons:
- Embedded, single-file DB
- Fast reads/writes
- No server
- Lightweight and reliable

2.5 NLP / Categorization Engine
Phase 1 (MVP): Rule-based only
- Fast
- Zero dependencies
- No Python runtime
- No heavy models

Phase 2 (Optional): Python via PyO3
- spaCy
- sentence-transformers
- BERTopic

2.6 Graph Engine
- In-memory adjacency lists
- Backed by SQLite persistence
- Custom Rust force-directed layout (Fruchterman–Reingold variant)
- Optimized for medium-sized graphs (1k–5k nodes)

## 3. Architecture Overview

UI Layer (egui/eframe)
Graph Visualization (layout + rendering)
Knowledge Graph Store (SQLite + Rust models)
NLP & Categorization (rule-based, optional ML)
File Ingestion Pipeline (parsers per format)
Local Filesystem

## 4. Module Structure

src/
  ui/
    mod.rs
    graph_view.rs
    node_panel.rs
    file_browser.rs
  graph/
    mod.rs
    layout.rs
    renderer.rs
    node.rs
    edge.rs
  storage/
    mod.rs
    sqlite.rs
    models.rs
  ingest/
    mod.rs
    pdf.rs
    docx.rs
    markdown.rs
    latex.rs
    text.rs
  nlp/
    mod.rs
    rules.rs
    keywords.rs
    structure.rs
    python_bridge.rs (future)
  app.rs
  main.rs

## 5. Data Model

Node:
id: i64
title: String
content: String
tags: Vec<String>
source_file_id: i64
position: Option<(f32, f32)>

Edge:
id: i64
from_id: i64
to_id: i64
edge_type: Structural | Semantic | TagRelation | Manual
weight: f32

FileRecord:
id: i64
path: String
file_type: Txt | Md | Pdf | Docx | Latex | Other(String)
title: String
imported_at: datetime

SQLite Schema:
files(id, path, file_type, title, imported_at)
nodes(id, title, content, tags, source_file_id, pos_x, pos_y)
edges(id, from_id, to_id, edge_type, weight)

## 6. Core Workflows

6.1 File Ingestion Workflow
- User imports file
- File type detected
- Routed to correct parser
- Extracted into ParsedDocument
- Rule-based NLP creates nodes and edges
- SQLite stores nodes and edges
- Graph layout recomputed
- Graph rendered in UI

6.2 Graph Layout Workflow
- Initialize positions
- Run force-directed iterations
- Store final positions
- Render via egui Painter

6.3 Node Interaction Workflow
- Click node opens detail panel
- Edit tags/content updates SQLite
- Graph updates accordingly

## 7. Performance Considerations

Why egui + SQLite:
- No browser
- No Electron
- GPU acceleration
- Small binary
- Fast local DB

Graph Size Targets:
- MVP: 1k–5k nodes
- Future: wgpu renderer for 10k+ nodes

NLP Performance:
- Rule-based = instant
- Optional Python ML = async, cached models

## 8. Non-Goals (MVP)
- No cloud sync
- No collaboration
- No web UI
- No remote AI APIs

## 9. Future Extensions
- Semantic search
- Tag-based subgraphs
- Timeline view
- Export graph formats
- Encrypted sync
- Plugin system

## 10. Summary for Claude
You are building NeuroMesh, a Rust + egui native desktop app that:
- Ingests notes from multiple formats
- Parses them into structured sections
- Converts sections into nodes and edges stored in SQLite
- Uses a custom Rust force-directed layout
- Renders an interactive graph via egui Painter
- Provides a UI with file import, graph canvas, and node detail panel
- Starts with rule-based categorization for performance and simplicity
- Has a clean module structure for future ML and wgpu upgrades

Use this document as the authoritative blueprint for implementation.
