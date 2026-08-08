//! File browser panel: import button plus a filterable list of ingested files.

use std::path::PathBuf;

use egui::{RichText, ScrollArea, TextEdit};

use crate::storage::models::FileRecord;

#[derive(Default)]
pub struct FileBrowser {
    filter: String,
}

#[derive(Default)]
pub struct FileBrowserAction {
    /// Paths picked via the import button.
    pub import: Option<Vec<PathBuf>>,
    /// (file id, title) whose remove button was clicked.
    pub delete: Option<(i64, String)>,
}

impl FileBrowser {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        files: &[FileRecord],
        frame: &eframe::Frame,
    ) -> FileBrowserAction {
        let mut action = FileBrowserAction::default();

        ui.heading("Files");
        if ui.button("📂 Import notes…").clicked() {
            action.import = pick_note_files(frame);
        }
        ui.add(TextEdit::singleline(&mut self.filter).hint_text("Filter…"));
        ui.separator();

        ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let needle = self.filter.to_lowercase();
            let mut shown = 0;
            for file in files {
                if !needle.is_empty() && !file.title.to_lowercase().contains(&needle) {
                    continue;
                }
                shown += 1;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} {}", file.file_type.icon(), file.title))
                            .strong(),
                    )
                    .on_hover_text(&file.path);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("🗑")
                            .on_hover_text("Remove from graph (file on disk is kept)")
                            .clicked()
                        {
                            action.delete = Some((file.id, file.title.clone()));
                        }
                    });
                });
                ui.label(RichText::new(&file.imported_at).small().weak());
                ui.separator();
            }
            if files.is_empty() {
                ui.weak("No files imported yet.");
            } else if shown == 0 {
                ui.weak("No files match the filter.");
            }
        });

        action
    }
}

/// Opens the native multi-file picker for supported note formats.
/// Parented to the app window so the dialog is modal and always in front.
pub fn pick_note_files(parent: &eframe::Frame) -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new()
        .set_parent(parent)
        .add_filter(
            "Notes",
            &["txt", "text", "md", "markdown", "pdf", "docx", "tex"],
        )
        .add_filter("All files", &["*"])
        .pick_files()
}
