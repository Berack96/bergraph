//! A* shortest-path visit, available for any graph, directed or undirected: Dijkstra guided
//! by a heuristic estimate of the remaining distance to `target`.

use std::fmt::{self, Debug, Formatter};

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, SearchBuffers, VisitError, VisitResult, reconstruct_path},
};

/// Shortest path from a start vertex to `target`, guided by `heuristic`: an estimate of the
/// remaining distance from a vertex to `target`. `heuristic` must be pure and never overestimate
/// the true remaining distance (be admissible), or the returned path may not be the shortest.
pub struct AStar<I: UnsignedId, H> {
    target: VertexId<I>,
    heuristic: H,
}

impl<I: UnsignedId, H> Debug for AStar<I, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AStar")
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl<I: UnsignedId, H> AStar<I, H>
where
    H: Fn(VertexId<I>) -> f64,
{
    pub fn new(target: VertexId<I>, heuristic: H) -> Self {
        Self { target, heuristic }
    }

    /// Runs the visit from `start`, `cost` giving the non-negative weight of each edge.
    pub fn visit<V: 'static, S: GraphEdgeStorage<Id = I>, G: GraphView<V, S>>(
        &self,
        view: &G,
        start: VertexId<I>,
        cost: impl Fn(&EdgeView<'_, S::Edge, I, S::EdgeKey>) -> f64,
    ) -> VisitResult<Option<(Vec<VertexId<I>>, f64)>> {
        self.visit_with(&mut SearchBuffers::default(), view, start, cost)
    }

    /// Same as [`visit`](Self::visit), reusing `buffers` instead of allocating new ones.
    pub fn visit_with<V: 'static, S: GraphEdgeStorage<Id = I>, G: GraphView<V, S>>(
        &self,
        buffers: &mut SearchBuffers<I>,
        view: &G,
        start: VertexId<I>,
        cost: impl Fn(&EdgeView<'_, S::Edge, I, S::EdgeKey>) -> f64,
    ) -> VisitResult<Option<(Vec<VertexId<I>>, f64)>> {
        if view.vertex(start).is_none() || view.vertex(self.target).is_none() {
            return Err(VisitError::VertexNotFound);
        }

        buffers.clear();
        buffers.reach(start, 0.0, None, (self.heuristic)(start));

        while let Some(MinEntry { priority, vertex }) = buffers.heap.pop() {
            if vertex == self.target {
                break;
            }
            let distance = buffers
                .distance
                .get(vertex)
                .expect("queued vertices are always reached");
            // Stale entry: a shorter path to `vertex` was queued since, at a lower priority.
            // shortcut: recomputes the heuristic on every pop; keep the distance in the heap
            // entry if heuristics get expensive.
            if priority > distance + (self.heuristic)(vertex) {
                continue;
            }

            for edge in view.edges().of(vertex) {
                let next_distance = distance + cost(&edge);
                if buffers
                    .distance
                    .get(edge.to)
                    .is_none_or(|known| next_distance < known)
                {
                    let priority = next_distance + (self.heuristic)(edge.to);
                    buffers.reach(edge.to, next_distance, Some(vertex), priority);
                }
            }
        }

        let Some(total) = *buffers.distance.get(self.target) else {
            return Ok(None);
        };
        Ok(Some((
            reconstruct_path(self.target, &buffers.previous),
            total,
        )))
    }
}
