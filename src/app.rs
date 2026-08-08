//! Application state and workflow orchestration: wires ingestion → NLP →
//! storage → graph → UI, per the core workflows in CLAUDE.md.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::graph::layout::ForceLayout;
use crate::graph::Graph;
use crate::ingest;
use crate::nlp;
use crate::storage::models::{Edge, EdgeType, FileRecord, Node};
use crate::storage::sqlite::Storage;
use crate::ui::file_browser::{pick_note_files, FileBrowser};
use crate::ui::graph_view::GraphView;
use crate::ui::node_panel::{NodePanel, NodePanelAction};
use crate::ui::note_editor::{NoteEditor, NoteEditorAction};

/// Minimum shared tags for a cross-document TagRelation edge.
const TAG_RELATION_MIN_SHARED: usize = 2;

enum ImportOutcome {
    Imported { nodes: usize },
    Skipped,
}

/// One line of the post-import report popup.
enum ReportLine {
    Ok(String),
    Skip(String),
    Fail(String),
}

/// A destructive action awaiting user confirmation.
enum PendingDelete {
    File { id: i64, title: String },
    Node { id: i64, title: String },
}

/// Replaces characters Windows forbids in file names.
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) { '-' } else { c })
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() {
        "Untitled".to_string()
    } else {
        cleaned
    }
}

/// Database location: `%APPDATA%\NeuroMesh\neuromesh.db`. The app may be
/// installed under Program Files, which is not writable, so the working
/// directory is only a fallback when APPDATA is unset.
fn db_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("NeuroMesh");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("neuromesh.db")
}

pub struct NeuroMeshApp {
    storage: Storage,
    /// Full node records by id (the graph keeps only render-relevant fields).
    nodes: HashMap<i64, Node>,
    files: Vec<FileRecord>,
    graph: Graph,
    layout: ForceLayout,
    layout_active: bool,
    selected: Option<i64>,
    graph_view: GraphView,
    node_panel: NodePanel,
    file_browser: FileBrowser,
    status: String,
    /// Per-file results of the most recent import, shown in a popup until dismissed.
    import_report: Option<Vec<ReportLine>>,
    /// Deletion awaiting confirmation in a popup.
    pending_delete: Option<PendingDelete>,
    /// Open "New note" editor window, if any.
    note_editor: Option<NoteEditor>,
}

impl NeuroMeshApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let storage = Storage::open(&db_path())?;
        let node_list = storage.load_nodes()?;
        let edges = storage.load_edges()?;
        let files = storage.load_files()?;

        let graph = Graph::from_models(&node_list, &edges, None);
        let mut layout = ForceLayout::default();
        // Skip the settle animation when every node already has a stored position.
        let all_placed = !node_list.is_empty() && node_list.iter().all(|n| n.position.is_some());
        if !all_placed {
            layout.reset(graph.nodes.len());
        }

        Ok(Self {
            storage,
            nodes: node_list.into_iter().map(|n| (n.id, n)).collect(),
            files,
            graph,
            layout,
            layout_active: !all_placed,
            selected: None,
            graph_view: GraphView::default(),
            node_panel: NodePanel::default(),
            file_browser: FileBrowser::default(),
            status: "Ready. Import notes (or drop files onto the window) to get started.".to_string(),
            import_report: None,
            pending_delete: None,
            note_editor: None,
        })
    }

    // ---- import workflow -------------------------------------------------

    fn import_files(&mut self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }
        let mut report: Vec<ReportLine> = Vec::new();
        let mut imported = 0usize;

        for path in paths {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            match self.import_one(&path) {
                Ok(ImportOutcome::Imported { nodes }) => {
                    imported += 1;
                    report.push(ReportLine::Ok(format!("{name} — {nodes} node(s) added")));
                }
                Ok(ImportOutcome::Skipped) => {
                    report.push(ReportLine::Skip(format!("{name} — already imported")));
                }
                Err(e) => report.push(ReportLine::Fail(format!("{name} — {e:#}"))),
            }
        }

        if imported > 0 {
            if let Err(e) = self.reload_from_storage() {
                report.push(ReportLine::Fail(format!("reloading graph failed — {e:#}")));
            }
        }

        self.status = format!(
            "Imported {imported} file(s) — {} nodes, {} edges total",
            self.graph.nodes.len(),
            self.graph.edges.len()
        );
        self.import_report = Some(report);
    }

    /// Full ingestion workflow for one file.
    fn import_one(&mut self, path: &Path) -> Result<ImportOutcome> {
        let path_str = path.to_string_lossy().to_string();
        if self.storage.file_id_for_path(&path_str)?.is_some() {
            return Ok(ImportOutcome::Skipped);
        }

        // Parse → categorize → persist.
        let doc = ingest::parse_file(path)?;
        let doc_graph = nlp::analyze(&doc);
        let file_id = self.storage.insert_file(&path_str, &doc.file_type, &doc.title)?;

        let mut ids = Vec::with_capacity(doc_graph.nodes.len());
        for dn in &doc_graph.nodes {
            let id = self.storage.insert_node(&Node {
                id: 0,
                title: dn.title.clone(),
                content: dn.content.clone(),
                tags: dn.tags.clone(),
                source_file_id: file_id,
                position: None,
            })?;
            ids.push(id);
        }
        for de in &doc_graph.edges {
            self.storage.insert_edge(&Edge {
                id: 0,
                from_id: ids[de.from],
                to_id: ids[de.to],
                edge_type: de.edge_type,
                weight: de.weight,
            })?;
        }

        // Cross-document connections: link new nodes to existing ones sharing tags.
        for (i, dn) in doc_graph.nodes.iter().enumerate() {
            for existing in self.nodes.values() {
                let shared = dn.tags.iter().filter(|t| existing.tags.contains(t)).count();
                if shared >= TAG_RELATION_MIN_SHARED {
                    self.storage.insert_edge(&Edge {
                        id: 0,
                        from_id: ids[i],
                        to_id: existing.id,
                        edge_type: EdgeType::TagRelation,
                        weight: shared as f32 * 0.5,
                    })?;
                }
            }
        }

        Ok(ImportOutcome::Imported {
            nodes: doc_graph.nodes.len(),
        })
    }

    fn reload_from_storage(&mut self) -> Result<()> {
        let node_list = self.storage.load_nodes()?;
        let edges = self.storage.load_edges()?;
        self.files = self.storage.load_files()?;
        self.graph = Graph::from_models(&node_list, &edges, Some(&self.graph));
        self.nodes = node_list.into_iter().map(|n| (n.id, n)).collect();
        self.reheat_layout();
        Ok(())
    }

    // ---- layout persistence ----------------------------------------------

    fn reheat_layout(&mut self) {
        self.layout.reset(self.graph.nodes.len());
        self.layout_active = true;
    }

    fn persist_all_positions(&mut self) {
        let positions: Vec<(i64, f32, f32)> = self
            .graph
            .nodes
            .iter()
            .map(|n| (n.id, n.pos.x, n.pos.y))
            .collect();
        if let Err(e) = self.storage.save_positions(&positions) {
            self.status = format!("Failed to save layout: {e:#}");
            return;
        }
        for (id, x, y) in positions {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.position = Some((x, y));
            }
        }
    }

    fn persist_node_position(&mut self, id: i64) {
        let Some((x, y)) = self.graph.node(id).map(|n| (n.pos.x, n.pos.y)) else {
            return;
        };
        if let Err(e) = self.storage.save_positions(&[(id, x, y)]) {
            self.status = format!("Failed to save position: {e:#}");
        } else if let Some(n) = self.nodes.get_mut(&id) {
            n.position = Some((x, y));
        }
    }

    // ---- deletion --------------------------------------------------------

    fn perform_delete(&mut self, pending: PendingDelete) {
        let result = match &pending {
            PendingDelete::File { id, title } => self
                .storage
                .delete_file(*id)
                .map(|_| format!("Removed '{title}' and its nodes from the graph.")),
            PendingDelete::Node { id, title } => {
                self.selected = None;
                self.storage
                    .delete_node(*id)
                    .map(|_| format!("Deleted node '{title}'."))
            }
        };
        match result {
            Ok(msg) => {
                if let Err(e) = self.reload_from_storage() {
                    self.status = format!("Deleted, but reloading the graph failed: {e:#}");
                } else {
                    self.status = msg;
                }
            }
            Err(e) => self.status = format!("Delete failed: {e:#}"),
        }
    }

    // ---- in-app note creation --------------------------------------------

    /// Saves the open editor's note as a real .md file (user picks where) and
    /// imports it through the normal pipeline.
    fn save_new_note(&mut self, frame: &eframe::Frame) {
        let Some(editor) = &self.note_editor else {
            return;
        };
        let title = if editor.title.trim().is_empty() {
            "Untitled".to_string()
        } else {
            editor.title.trim().to_string()
        };
        let Some(path) = rfd::FileDialog::new()
            .set_parent(frame)
            .set_file_name(format!("{}.md", sanitize_filename(&title)))
            .add_filter("Markdown", &["md"])
            .save_file()
        else {
            return; // user cancelled the save dialog; keep the editor open
        };
        let mut text = String::new();
        // Prepend the typed title as the document H1 unless the content
        // already opens with its own level-1 heading.
        if !editor.content.trim_start().starts_with("# ") {
            text.push_str(&format!("# {title}\n\n"));
        }
        text.push_str(&editor.content);
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.note_editor = None;
                self.import_files(vec![path]);
            }
            Err(e) => self.status = format!("Failed to save note: {e:#}"),
        }
    }

    // ---- node editing ----------------------------------------------------

    fn apply_node_edit(&mut self, updated: Node) {
        if let Err(e) = self.storage.update_node(&updated) {
            self.status = format!("Failed to save node: {e:#}");
            return;
        }
        if let Some(gn) = self.graph.node_mut(updated.id) {
            gn.title = updated.title.clone();
            gn.tags = updated.tags.clone();
        }
        self.nodes.insert(updated.id, updated);
        self.status = "Node saved.".to_string();
    }
}

impl eframe::App for NeuroMeshApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Files dropped onto the window import directly — no dialog needed.
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            self.import_files(dropped);
        }

        // Advance the force layout; persist positions once it settles.
        if self.layout_active {
            if self.layout.step(&mut self.graph) {
                ctx.request_repaint();
            } else {
                self.layout_active = false;
                self.persist_all_positions();
            }
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧠 NeuroMesh");
                ui.separator();
                if ui.button("📂 Import notes…").clicked() {
                    if let Some(paths) = pick_note_files(frame) {
                        self.import_files(paths);
                    }
                }
                if ui.button("📝 New note…").clicked() {
                    self.note_editor = Some(NoteEditor::default());
                }
                if ui.button("↻ Re-run layout").clicked() {
                    self.reheat_layout();
                }
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{} files · {} nodes · {} edges{}",
                        self.files.len(),
                        self.graph.nodes.len(),
                        self.graph.edges.len(),
                        if self.layout_active { " · layout…" } else { "" }
                    ));
                });
            });
        });

        egui::SidePanel::left("file_browser")
            .default_width(230.0)
            .resizable(true)
            .show(ctx, |ui| {
                let action = self.file_browser.show(ui, &self.files, frame);
                if let Some(paths) = action.import {
                    self.import_files(paths);
                }
                if let Some((id, title)) = action.delete {
                    self.pending_delete = Some(PendingDelete::File { id, title });
                }
            });

        if let Some(id) = self.selected {
            match self.nodes.get(&id).cloned() {
                Some(node) => {
                    let source = self
                        .files
                        .iter()
                        .find(|f| f.id == node.source_file_id)
                        .cloned();
                    let connections: Vec<(i64, String)> = self
                        .graph
                        .neighbors(id)
                        .into_iter()
                        .filter_map(|nid| self.graph.node(nid).map(|n| (nid, n.title.clone())))
                        .collect();

                    let mut action = NodePanelAction::default();
                    egui::SidePanel::right("node_panel")
                        .default_width(320.0)
                        .resizable(true)
                        .show(ctx, |ui| {
                            action = self.node_panel.show(ui, &node, source.as_ref(), &connections);
                        });
                    if let Some(updated) = action.save {
                        self.apply_node_edit(updated);
                    }
                    if let Some(nav) = action.navigate {
                        self.selected = Some(nav);
                    }
                    if action.delete {
                        self.pending_delete = Some(PendingDelete::Node {
                            id,
                            title: node.title.clone(),
                        });
                    }
                }
                None => self.selected = None,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let out = self.graph_view.show(ui, &mut self.graph, &mut self.selected);
            if let Some(id) = out.released_node {
                self.persist_node_position(id);
            }
        });

        // Per-file results of the last import — visible until dismissed, so
        // failures (e.g. unsupported formats) are never silent.
        if self.import_report.is_some() {
            let mut dismissed = false;
            egui::Window::new("Import results")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    for line in self.import_report.as_ref().unwrap() {
                        match line {
                            ReportLine::Ok(text) => {
                                ui.colored_label(egui::Color32::from_rgb(130, 210, 150), text)
                            }
                            ReportLine::Skip(text) => {
                                ui.colored_label(egui::Color32::from_gray(160), text)
                            }
                            ReportLine::Fail(text) => {
                                ui.colored_label(egui::Color32::from_rgb(240, 130, 130), text)
                            }
                        };
                    }
                    ui.separator();
                    if ui.button("OK").clicked() {
                        dismissed = true;
                    }
                });
            if dismissed {
                self.import_report = None;
            }
        }

        // Confirmation popup for pending deletions.
        if self.pending_delete.is_some() {
            let message = match self.pending_delete.as_ref().unwrap() {
                PendingDelete::File { title, .. } => format!(
                    "Remove \"{title}\" and all of its nodes from the graph?\n\
                     The file on disk is not deleted."
                ),
                PendingDelete::Node { title, .. } => {
                    format!("Delete node \"{title}\" and its connections?")
                }
            };
            let mut confirmed = false;
            let mut cancelled = false;
            egui::Window::new("Confirm deletion")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(message);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancelled = true;
                        }
                    });
                });
            if confirmed {
                if let Some(pending) = self.pending_delete.take() {
                    self.perform_delete(pending);
                }
            } else if cancelled {
                self.pending_delete = None;
            }
        }

        // "New note" editor window.
        if self.note_editor.is_some() {
            let action = self.note_editor.as_mut().unwrap().show(ctx);
            match action {
                NoteEditorAction::Save => self.save_new_note(frame),
                NoteEditorAction::Cancel => self.note_editor = None,
                NoteEditorAction::None => {}
            }
        }

        // Full-window overlay while files are being dragged over the app.
        let hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if hovering_files {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drop_overlay"),
            ));
            let rect = ctx.screen_rect();
            painter.rect_filled(
                rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(25, 45, 90, 140),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop notes to import",
                egui::FontId::proportional(28.0),
                egui::Color32::WHITE,
            );
        }
    }
}
