//! In-app note creation: a floating editor window. Saving writes a real .md
//! file (user picks the location) and runs it through the normal import
//! pipeline, so in-app notes behave exactly like imported ones.

use egui::{Align2, TextEdit, Vec2};

#[derive(Default)]
pub struct NoteEditor {
    pub title: String,
    pub content: String,
}

#[derive(PartialEq, Eq)]
pub enum NoteEditorAction {
    None,
    Cancel,
    Save,
}

impl NoteEditor {
    pub fn show(&mut self, ctx: &egui::Context) -> NoteEditorAction {
        let mut action = NoteEditorAction::None;
        egui::Window::new("New note")
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Title");
                ui.add(TextEdit::singleline(&mut self.title).desired_width(f32::INFINITY));
                ui.label("Content (Markdown — headings become graph nodes)");
                ui.add(
                    TextEdit::multiline(&mut self.content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(16),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("💾 Save & import").clicked() {
                        action = NoteEditorAction::Save;
                    }
                    if ui.button("Cancel").clicked() {
                        action = NoteEditorAction::Cancel;
                    }
                });
            });
        action
    }
}
