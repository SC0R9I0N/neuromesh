//! Render-ready edge representation.

use crate::storage::models::EdgeType;

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: i64,
    pub to: i64,
    pub edge_type: EdgeType,
    pub weight: f32,
}
