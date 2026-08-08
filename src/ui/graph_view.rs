//! Interactive graph canvas: pan (drag background), zoom (scroll, anchored at
//! the cursor), select (click), and reposition nodes (drag).

use egui::{Pos2, Rect, Sense};

use crate::graph::renderer::{self, Camera};
use crate::graph::Graph;

#[derive(Default)]
pub struct GraphView {
    pub camera: Camera,
    drag_node: Option<i64>,
}

pub struct GraphViewOutput {
    /// Set when the user finished dragging a node; the app persists its position.
    pub released_node: Option<i64>,
}

impl GraphView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut Graph,
        selected: &mut Option<i64>,
    ) -> GraphViewOutput {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        // Scroll-to-zoom, anchored at the cursor so the point under it stays put.
        if let Some(hover) = response.hover_pos() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let before = self.camera.screen_to_world(hover, rect);
                self.camera.zoom = (self.camera.zoom * (1.0 + scroll * 0.0015)).clamp(0.05, 6.0);
                let after = self.camera.screen_to_world(hover, rect);
                self.camera.center += before - after;
            }
        }

        let hovered = response
            .hover_pos()
            .and_then(|p| hit_node(graph, &self.camera, rect, p));

        if response.drag_started() {
            self.drag_node = response
                .interact_pointer_pos()
                .and_then(|p| hit_node(graph, &self.camera, rect, p));
            if let Some(id) = self.drag_node {
                if let Some(n) = graph.node_mut(id) {
                    n.pinned = true;
                }
            }
        }

        if response.dragged() {
            let delta = response.drag_delta() / self.camera.zoom;
            match self.drag_node {
                Some(id) => {
                    if let Some(n) = graph.node_mut(id) {
                        n.pos += delta;
                    }
                }
                None => self.camera.center -= delta,
            }
        }

        let mut released_node = None;
        if response.drag_stopped() {
            if let Some(id) = self.drag_node.take() {
                if let Some(n) = graph.node_mut(id) {
                    n.pinned = false;
                }
                released_node = Some(id);
            }
        }

        if response.clicked() {
            *selected = hovered;
        }

        renderer::draw_graph(&painter, rect, &self.camera, graph, *selected, hovered);

        GraphViewOutput { released_node }
    }
}

/// Returns the id of the closest node whose circle contains `p`, if any.
fn hit_node(graph: &Graph, camera: &Camera, rect: Rect, p: Pos2) -> Option<i64> {
    let mut best: Option<(i64, f32)> = None;
    for n in &graph.nodes {
        let sp = camera.world_to_screen(n.pos, rect);
        let r = renderer::node_screen_radius(n, camera.zoom) + 3.0;
        let d = sp.distance(p);
        if d <= r && best.map_or(true, |(_, bd)| d < bd) {
            best = Some((n.id, d));
        }
    }
    best.map(|(id, _)| id)
}
