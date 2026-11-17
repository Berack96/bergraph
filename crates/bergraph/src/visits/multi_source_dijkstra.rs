//! Multi-source Dijkstra visit, available for any graph, directed or undirected.

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, SearchBuffers, VisitError, VisitResult},
};

/// Total edge weight from, and identity of, the source that reaches a vertex at the lowest cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DijkstraReached<I: UnsignedId> {
    /// Total edge weight from the source.
    pub distance: f64,
    /// The source reaching the vertex at the lowest cost.
    pub source: VertexId<I>,
}

/// One `(vertex, reached)` entry per vertex a multi-source Dijkstra visit reached.
pub type DijkstraReaching<I> = Vec<(VertexId<I>, DijkstraReached<I>)>;

/// Dijkstra expansion from several start vertices at once: every vertex is attributed to
/// whichever source reaches it at the lowest total edge weight, a tie going to the path whose
/// last step comes from the vertex finalized first (lower id on equal distance). With a single
/// source it gives the distances from it, e.g. the vertices within a radius when bounded.
#[derive(Debug, Clone, Copy)]
pub struct MultiSourceDijkstra {
    max_distance: f64,
}

impl MultiSourceDijkstra {
    /// Expansion that leaves unreached every vertex farther than `max_distance` from all
    /// sources; `f64::INFINITY` for no bound.
    pub fn new(max_distance: f64) -> Self {
        Self { max_distance }
    }

    /// Runs the visit, `cost` giving the non-negative weight of each edge; lists every reached
    /// vertex in the order its distance was finalized.
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if a source is not in `view`.
    pub fn visit<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
        sources: impl IntoIterator<Item = VertexId<S::Id>>,
        cost: impl Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> f64,
    ) -> VisitResult<DijkstraReaching<S::Id>> {
        self.visit_with(&mut SearchBuffers::default(), view, sources, cost)
    }

    /// Same as [`visit`](Self::visit), reusing `buffers` instead of allocating new ones.
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if a source is not in `view`.
    pub fn visit_with<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        buffers: &mut SearchBuffers<S::Id>,
        view: &G,
        sources: impl IntoIterator<Item = VertexId<S::Id>>,
        cost: impl Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> f64,
    ) -> VisitResult<DijkstraReaching<S::Id>> {
        buffers.clear();
        for source in sources {
            if view.vertex(source).is_none() {
                return Err(VisitError::VertexNotFound);
            }
            if buffers.distance.get(source).is_none() {
                buffers.reach(source, 0.0, Some(source), 0.0);
            }
        }

        let mut order = Vec::new();
        while let Some(MinEntry {
            priority: distance,
            vertex,
        }) = buffers.heap.pop()
        {
            if buffers
                .distance
                .get(vertex)
                .is_some_and(|known| known < distance)
            {
                continue; // stale entry, superseded by a shorter path found since
            }
            let source = buffers
                .previous
                .get(vertex)
                .expect("reached vertices always have a source");
            order.push((vertex, DijkstraReached { distance, source }));

            for edge in view.edges().of(vertex) {
                let next_distance = distance + cost(&edge);
                if next_distance > self.max_distance {
                    continue;
                }
                if buffers
                    .distance
                    .get(edge.to)
                    .is_none_or(|known| next_distance < known)
                {
                    buffers.reach(edge.to, next_distance, Some(source), next_distance);
                }
            }
        }

        Ok(order)
    }
}
