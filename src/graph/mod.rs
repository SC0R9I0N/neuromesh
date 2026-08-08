//! Graph engine: in-memory adjacency-list graph backed by SQLite persistence.

pub mod edge;
pub mod layout;
pub mod node;
pub mod renderer;

use std::collections::HashMap;

use egui::{pos2, Pos2};

pub use edge::GraphEdge;
pub use node::GraphNode;

use crate::storage::models::{Edge, Node};

/// The in-memory, render-ready view of the knowledge graph.
#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// node id → index into `nodes`
    index: HashMap<i64, usize>,
    /// adjacency lists, parallel to `nodes` (indices into `nodes`)
    adjacency: Vec<Vec<usize>>,
}

impl Graph {
    /// Builds a graph from storage models. Position priority: the live position in
    /// `prev` (so an in-progress layout is not thrown away on re-import), then the
    /// persisted position, then a deterministic spiral placement for new nodes.
    pub fn from_models(nodes: &[Node], edges: &[Edge], prev: Option<&Graph>) -> Self {
        let mut g = Graph {
            nodes: Vec::with_capacity(nodes.len()),
            edges: Vec::with_capacity(edges.len()),
            index: HashMap::with_capacity(nodes.len()),
            adjacency: Vec::new(),
        };
        for (i, n) in nodes.iter().enumerate() {
            let pos = prev
                .and_then(|p| p.node(n.id))
                .map(|gn| gn.pos)
                .or_else(|| n.position.map(|(x, y)| pos2(x, y)))
                .unwrap_or_else(|| spiral_position(i));
            g.index.insert(n.id, i);
            g.nodes.push(GraphNode {
                id: n.id,
                title: n.title.clone(),
                tags: n.tags.clone(),
                pos,
                pinned: false,
                degree: 0,
            });
        }
        for e in edges {
            if g.index.contains_key(&e.from_id) && g.index.contains_key(&e.to_id) {
                g.edges.push(GraphEdge {
                    from: e.from_id,
                    to: e.to_id,
                    edge_type: e.edge_type,
                    weight: e.weight,
                });
            }
        }
        g.rebuild_adjacency();
        g
    }

    fn rebuild_adjacency(&mut self) {
        self.adjacency = vec![Vec::new(); self.nodes.len()];
        for e in &self.edges {
            let a = self.index[&e.from];
            let b = self.index[&e.to];
            self.adjacency[a].push(b);
            self.adjacency[b].push(a);
        }
        let degrees: Vec<u32> = self.adjacency.iter().map(|a| a.len() as u32).collect();
        for (node, degree) in self.nodes.iter_mut().zip(degrees) {
            node.degree = degree;
        }
    }

    pub fn index_of(&self, id: i64) -> Option<usize> {
        self.index.get(&id).copied()
    }

    pub fn node(&self, id: i64) -> Option<&GraphNode> {
        self.index_of(id).map(|i| &self.nodes[i])
    }

    pub fn node_mut(&mut self, id: i64) -> Option<&mut GraphNode> {
        let i = self.index_of(id)?;
        Some(&mut self.nodes[i])
    }

    /// Ids of all nodes sharing an edge with `id`.
    pub fn neighbors(&self, id: i64) -> Vec<i64> {
        self.index_of(id)
            .map(|i| {
                self.adjacency[i]
                    .iter()
                    .map(|&j| self.nodes[j].id)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Deterministic golden-angle spiral so new nodes start spread out instead of stacked.
fn spiral_position(i: usize) -> Pos2 {
    let n = i as f32;
    let angle = n * 2.399_963; // golden angle in radians
    let radius = 45.0 * (n + 1.0).sqrt();
    pos2(radius * angle.cos(), radius * angle.sin())
}
