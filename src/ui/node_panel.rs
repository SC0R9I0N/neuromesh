//! Node detail panel: view/edit the selected node and navigate its connections.

use egui::{RichText, ScrollArea, TextEdit};

use crate::storage::models::{FileRecord, Node};

#[derive(Default)]
pub struct NodePanel {
    current: Option<i64>,
    title_buf: String,
    tags_buf: String,
    content_buf: String,
}

#[derive(Default)]
pub struct NodePanelAction {
    /// Set when the user clicked Save: the node with edited fields applied.
    pub save: Option<Node>,
    /// Set when the user clicked a connected node's link.
    pub navigate: Option<i64>,
    /// True when the user asked to delete this node.
    pub delete: bool,
}

impl NodePanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        node: &Node,
        source_file: Option<&FileRecord>,
        connections: &[(i64, String)],
    ) -> NodePanelAction {
        // Reload edit buffers whenever the selection changes.
        if self.current != Some(node.id) {
            self.current = Some(node.id);
            self.title_buf = node.title.clone();
            self.tags_buf = node.tags.join(", ");
            self.content_buf = node.content.clone();
        }

        let mut action = NodePanelAction::default();

        ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.heading("Node");
            if let Some(file) = source_file {
                ui.label(
                    RichText::new(format!("{} {}", file.file_type.icon(), file.title)).weak(),
                );
            }
            ui.separator();

            ui.label("Title");
            ui.add(TextEdit::singleline(&mut self.title_buf).desired_width(f32::INFINITY));

            ui.label("Tags (comma-separated)");
            ui.add(TextEdit::singleline(&mut self.tags_buf).desired_width(f32::INFINITY));

            ui.label("Content");
            ui.add(
                TextEdit::multiline(&mut self.content_buf)
                    .desired_width(f32::INFINITY)
                    .desired_rows(14),
            );

            if ui.button("💾 Save changes").clicked() {
                action.save = Some(Node {
                    id: node.id,
                    title: self.title_buf.trim().to_string(),
                    content: self.content_buf.clone(),
                    tags: self
                        .tags_buf
                        .split(',')
                        .map(str::trim)
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                        .collect(),
                    source_file_id: node.source_file_id,
                    position: node.position,
                });
            }

            ui.separator();
            ui.label(RichText::new(format!("Connections ({})", connections.len())).strong());
            for (id, title) in connections {
                if ui.link(title).clicked() {
                    action.navigate = Some(*id);
                }
            }
            if connections.is_empty() {
                ui.weak("No connections.");
            }

            ui.separator();
            if ui.button("🗑 Delete node").clicked() {
                action.delete = true;
            }
        });

        action
    }
}
