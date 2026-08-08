#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod graph;
mod ingest;
mod nlp;
mod storage;
mod ui;

use app::NeuroMeshApp;

fn main() -> eframe::Result<()> {
    // 64x64 RGBA dump generated alongside assets/icon.ico — no image decoder needed.
    let icon = egui::IconData {
        rgba: include_bytes!("../assets/icon_64.rgba").to_vec(),
        width: 64,
        height: 64,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "NeuroMesh",
        options,
        Box::new(|cc| Ok(Box::new(NeuroMeshApp::new(cc)?))),
    )
}
