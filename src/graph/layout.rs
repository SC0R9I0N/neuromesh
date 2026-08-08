//! Custom force-directed layout — a Fruchterman–Reingold variant with simulated
//! annealing (temperature cap + cooling) and a weak gravity term that keeps
//! disconnected components from drifting apart.
//!
//! Complexity is O(n²) per step for repulsion, which is fine for the MVP target
//! of 1k–5k nodes. Upgrade path: Barnes–Hut quadtree + wgpu rendering for 10k+.

use egui::Vec2;

use super::Graph;

const MIN_TEMP: f32 = 0.5;
const COOLING: f32 = 0.96;
const GRAVITY: f32 = 0.02;
const SETTLE_MOVEMENT: f32 = 0.05;

pub struct ForceLayout {
    /// Max displacement per step; cools each iteration (simulated annealing).
    pub temperature: f32,
    /// Ideal edge length in world units.
    pub k: f32,
}

impl Default for ForceLayout {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            k: 120.0,
        }
    }
}

impl ForceLayout {
    /// Re-heats the simulation, scaling the ideal edge length to the node count.
    pub fn reset(&mut self, node_count: usize) {
        self.temperature = 250.0;
        let n = node_count.max(1) as f32;
        self.k = (0.9 * (900_000.0 / n).sqrt()).clamp(60.0, 200.0);
    }

    /// Advances the simulation one step. Returns `false` once settled.
    pub fn step(&mut self, graph: &mut Graph) -> bool {
        let n = graph.nodes.len();
        if n < 2 || self.temperature <= MIN_TEMP {
            return false;
        }
        let k = self.k;
        let mut disp = vec![Vec2::ZERO; n];

        // Repulsion between every pair of nodes.
        for i in 0..n {
            for j in (i + 1)..n {
                let delta = graph.nodes[i].pos - graph.nodes[j].pos;
                let dist = delta.length().max(0.01);
                let force = k * k / dist;
                let dir = delta / dist;
                disp[i] += dir * force;
                disp[j] -= dir * force;
            }
        }

        // Attraction along edges, scaled by edge weight.
        for e in &graph.edges {
            let (Some(a), Some(b)) = (graph.index_of(e.from), graph.index_of(e.to)) else {
                continue;
            };
            let delta = graph.nodes[a].pos - graph.nodes[b].pos;
            let dist = delta.length().max(0.01);
            let force = dist * dist / k * e.weight.clamp(0.3, 3.0);
            let dir = delta / dist;
            disp[a] -= dir * force;
            disp[b] += dir * force;
        }

        // Weak gravity toward the origin keeps disconnected documents nearby.
        for (i, node) in graph.nodes.iter().enumerate() {
            disp[i] -= node.pos.to_vec2() * GRAVITY;
        }

        // Apply displacements, capped by the current temperature.
        let mut max_move: f32 = 0.0;
        for (i, node) in graph.nodes.iter_mut().enumerate() {
            if node.pinned {
                continue;
            }
            let len = disp[i].length();
            if len > 0.0 {
                let step = disp[i] / len * len.min(self.temperature);
                node.pos += step;
                max_move = max_move.max(step.length());
            }
        }

        self.temperature *= COOLING;
        self.temperature > MIN_TEMP && max_move > SETTLE_MOVEMENT
    }
}
