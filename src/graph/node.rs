//! Render-ready node representation used by the layout and renderer.

use egui::Pos2;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: i64,
    pub title: String,
    pub tags: Vec<String>,
    /// Position in world (layout) coordinates.
    pub pos: Pos2,
    /// True while the user is dragging this node; the layout leaves it alone.
    pub pinned: bool,
    /// Number of incident edges; drives the rendered radius.
    pub degree: u32,
}

impl GraphNode {
    /// World-space radius: hubs render larger, capped so they never dominate.
    pub fn radius(&self) -> f32 {
        (7.0 + (self.degree as f32).sqrt() * 2.5).min(20.0)
    }
}
