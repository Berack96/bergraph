//! Priority-Flood visit, available for any graph, directed or undirected.

use std::collections::BinaryHeap;

use crate::{
    edges::GraphEdgeStorage,
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, VertexMarks, VisitError, VisitResult},
};

/// Level a vertex is flooded at, and the vertex it was flooded from (`None` on a seed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flooded<I: UnsignedId> {
    /// Level the vertex is flooded at.
    pub level: f64,
    /// Vertex it was flooded from, `None` on a seed.
    pub from: Option<VertexId<I>>,
}

/// One `(vertex, flooded)` entry per vertex a Priority-Flood visit reached.
pub type Flooding<I> = Vec<(VertexId<I>, Flooded<I>)>;

/// Expansion from seed vertices in order of a per-vertex priority, ties broken by the lower
/// vertex id: each vertex is flooded at the higher of its own priority and its flooder's level.
#[derive(Debug, Clone, Copy)]
pub struct PriorityFlood;

impl PriorityFlood {
    /// Runs the visit; lists every reached vertex in the order it was flooded. Fails on a seed
    /// missing from `view`.
    pub fn visit<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
        seeds: impl IntoIterator<Item = VertexId<S::Id>>,
        priority: impl Fn(VertexId<S::Id>) -> f64,
    ) -> VisitResult<Flooding<S::Id>> {
        let mut marks: VertexMarks<Option<Flooded<S::Id>>> = VertexMarks::new(None);
        let mut heap = BinaryHeap::new();

        for seed in seeds {
            if view.vertex(seed).is_none() {
                return Err(VisitError::VertexNotFound);
            }
            if marks.get(seed).is_some() {
                continue;
            }
            let level = priority(seed);
            marks.set(seed, Some(Flooded { level, from: None }));
            heap.push(MinEntry {
                priority: level,
                vertex: seed,
            });
        }

        let mut order = Vec::new();
        while let Some(MinEntry {
            priority: level,
            vertex,
        }) = heap.pop()
        {
            let flooded = marks
                .get(vertex)
                .expect("queued vertices are always marked");
            order.push((vertex, flooded));

            for edge in view.edges().of(vertex) {
                if marks.get(edge.to).is_some() {
                    continue;
                }
                let next_level = priority(edge.to).max(level);
                marks.set(
                    edge.to,
                    Some(Flooded {
                        level: next_level,
                        from: Some(vertex),
                    }),
                );
                heap.push(MinEntry {
                    priority: next_level,
                    vertex: edge.to,
                });
            }
        }

        Ok(order)
    }
}
