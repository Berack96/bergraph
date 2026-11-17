//! Dijkstra shortest-path visit, available for any graph, directed or undirected.

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, SearchBuffers, VertexMarks, VisitError, VisitResult, reconstruct_path},
};

/// Shortest paths from a start vertex to every vertex it reaches, from [`Dijkstra::visit_all`].
#[derive(Debug)]
pub struct ShortestPathTree<I: UnsignedId> {
    distance: VertexMarks<Option<f64>>,
    previous: VertexMarks<Option<VertexId<I>>>,
}

impl<I: UnsignedId> ShortestPathTree<I> {
    /// Total weight of the shortest path to `vertex`, `None` if the start does not reach it.
    pub fn distance(&self, vertex: VertexId<I>) -> Option<f64> {
        *self.distance.get(vertex)
    }

    /// Vertex sequence of the shortest path from the start to `vertex`, `None` if unreached.
    pub fn path_to(&self, vertex: VertexId<I>) -> Option<Vec<VertexId<I>>> {
        self.distance(vertex)
            .map(|_| reconstruct_path(vertex, &self.previous))
    }
}

/// Shortest path from a start vertex to `target`: the vertex sequence and its total weight; see
/// [`visit_all`](Self::visit_all) for the paths to every vertex.
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
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if `start` or the target is not in `view`.
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

        search(buffers, view, start, Some(self.target), cost);

        let Some(total) = *buffers.distance.get(self.target) else {
            return Ok(None);
        };
        Ok(Some((
            reconstruct_path(self.target, &buffers.previous),
            total,
        )))
    }

    /// Shortest paths from `start` to every vertex it reaches, `cost` giving the non-negative
    /// weight of each edge; for distances only, see [`MultiSourceDijkstra`](super::MultiSourceDijkstra).
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if `start` is not in `view`.
    pub fn visit_all<V: 'static, S: GraphEdgeStorage<Id = I>, G: GraphView<V, S>>(
        view: &G,
        start: VertexId<I>,
        cost: impl Fn(&EdgeView<'_, S::Edge, I, S::EdgeKey>) -> f64,
    ) -> VisitResult<ShortestPathTree<I>> {
        if view.vertex(start).is_none() {
            return Err(VisitError::VertexNotFound);
        }
        let mut buffers = SearchBuffers::default();
        search(&mut buffers, view, start, None, cost);
        Ok(ShortestPathTree {
            distance: buffers.distance,
            previous: buffers.previous,
        })
    }
}

/// Settles vertices from `start` in distance order into `buffers`, until `target` if given.
fn search<I: UnsignedId, V: 'static, S: GraphEdgeStorage<Id = I>, G: GraphView<V, S>>(
    buffers: &mut SearchBuffers<I>,
    view: &G,
    start: VertexId<I>,
    target: Option<VertexId<I>>,
    cost: impl Fn(&EdgeView<'_, S::Edge, I, S::EdgeKey>) -> f64,
) {
    buffers.clear();
    buffers.reach(start, 0.0, None, 0.0);

    while let Some(MinEntry {
        priority: distance,
        vertex,
    }) = buffers.heap.pop()
    {
        if Some(vertex) == target {
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
}
