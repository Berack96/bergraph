//! Dijkstra shortest-path visit, available for any graph, directed or undirected.

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, SearchBuffers, VisitError, VisitResult, reconstruct_path},
};

/// Shortest path from a start vertex to `target`: the vertex sequence and its total weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dijkstra<I: UnsignedId> {
    target: VertexId<I>,
}

impl<I: UnsignedId> Dijkstra<I> {
    /// Search for the shortest path to `target`.
    pub fn new(target: VertexId<I>) -> Self {
        Self { target }
    }

    /// Runs the visit from `start`, `cost` giving the non-negative weight of each edge.
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if `start` or the target is not in `view`.
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
        buffers.reach(start, 0.0, None, 0.0);

        while let Some(MinEntry {
            priority: distance,
            vertex,
        }) = buffers.heap.pop()
        {
            if vertex == self.target {
                break;
            }
            if buffers
                .distance
                .get(vertex)
                .is_some_and(|known| known < distance)
            {
                continue;
            }

            for edge in view.edges().of(vertex) {
                let next_distance = distance + cost(&edge);
                if buffers
                    .distance
                    .get(edge.to)
                    .is_none_or(|known| next_distance < known)
                {
                    buffers.reach(edge.to, next_distance, Some(vertex), next_distance);
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
