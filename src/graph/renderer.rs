//! Graph rendering via the egui Painter (MVP renderer; wgpu is the upgrade path
//! once graphs exceed ~5k nodes).

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

use super::{Graph, GraphNode};
use crate::storage::models::EdgeType;

/// Pan/zoom camera mapping world (layout) coordinates to screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// World point shown at the center of the viewport.
    pub center: Pos2,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: Pos2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn world_to_screen(&self, world: Pos2, rect: Rect) -> Pos2 {
        rect.center() + (world - self.center) * self.zoom
    }

    pub fn screen_to_world(&self, screen: Pos2, rect: Rect) -> Pos2 {
        self.center + (screen - rect.center()) / self.zoom
    }
}

pub fn node_screen_radius(node: &GraphNode, zoom: f32) -> f32 {
    node.radius() * zoom.clamp(0.35, 2.0)
}

pub fn draw_graph(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    graph: &Graph,
    selected: Option<i64>,
    hovered: Option<i64>,
) {
    painter.rect_filled(rect, egui::Rounding::ZERO, Color32::from_gray(16));

    if graph.nodes.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Import notes to grow your knowledge graph",
            FontId::proportional(16.0),
            Color32::from_gray(110),
        );
        return;
    }

    // Edges first, under the nodes.
    for e in &graph.edges {
        let (Some(a), Some(b)) = (graph.node(e.from), graph.node(e.to)) else {
            continue;
        };
        let p1 = camera.world_to_screen(a.pos, rect);
        let p2 = camera.world_to_screen(b.pos, rect);
        if !rect.intersects(Rect::from_two_pos(p1, p2)) {
            continue;
        }
        let (color, width) = edge_style(e.edge_type, e.weight);
        painter.line_segment([p1, p2], Stroke::new(width * camera.zoom.clamp(0.3, 1.5), color));
    }

    let show_labels = camera.zoom >= 0.55;
    for n in &graph.nodes {
        let p = camera.world_to_screen(n.pos, rect);
        let r = node_screen_radius(n, camera.zoom);
        if !rect.expand(r + 2.0).contains(p) {
            continue;
        }
        painter.circle_filled(p, r, node_color(n));
        if selected == Some(n.id) {
            painter.circle_stroke(p, r + 2.0, Stroke::new(2.0_f32, Color32::WHITE));
        } else if hovered == Some(n.id) {
            painter.circle_stroke(p, r + 1.5, Stroke::new(1.5_f32, Color32::from_gray(180)));
        }
        if show_labels || selected == Some(n.id) || hovered == Some(n.id) {
            painter.text(
                p + Vec2::new(0.0, r + 3.0),
                Align2::CENTER_TOP,
                truncate(&n.title, 28),
                FontId::proportional(11.0),
                Color32::from_gray(205),
            );
        }
    }
}

fn edge_style(edge_type: EdgeType, weight: f32) -> (Color32, f32) {
    let color = match edge_type {
        EdgeType::Structural => Color32::from_gray(105),
        EdgeType::Semantic => Color32::from_rgb(80, 130, 220),
        EdgeType::TagRelation => Color32::from_rgb(80, 180, 120),
        EdgeType::Manual => Color32::from_rgb(230, 160, 60),
    };
    (color, (0.8 + weight * 0.4).min(3.0))
}

const PALETTE: [Color32; 8] = [
    Color32::from_rgb(102, 153, 255),
    Color32::from_rgb(255, 140, 120),
    Color32::from_rgb(120, 200, 140),
    Color32::from_rgb(220, 170, 90),
    Color32::from_rgb(190, 130, 220),
    Color32::from_rgb(90, 200, 210),
    Color32::from_rgb(235, 120, 170),
    Color32::from_rgb(160, 180, 100),
];

/// Color a node by its first tag so tag families cluster visually.
fn node_color(node: &GraphNode) -> Color32 {
    match node.tags.first() {
        Some(tag) => {
            let hash = tag
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            PALETTE[(hash % PALETTE.len() as u64) as usize]
        }
        None => Color32::from_rgb(140, 150, 165),
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    } else {
        s.to_string()
    }
}
