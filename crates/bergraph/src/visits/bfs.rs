//! Breadth-first visit, available for any graph, directed or undirected.

use std::collections::VecDeque;

use crate::{
    edges::GraphEdgeStorage,
    graph::GraphView,
    vertices::VertexId,
    visits::{Mark, VertexMarks, VisitError, VisitResult},
};

/// Breadth-first traversal from a start vertex, returning the vertices in visit order.
#[derive(Debug, Clone, Copy)]
pub struct Bfs;

impl Bfs {
    /// Vertices reachable from `start`, in breadth-first order.
    ///
    /// # Errors
    ///
    /// [`VisitError::VertexNotFound`] if `start` is not in `view`.
    pub fn visit<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
        start: VertexId<S::Id>,
    ) -> VisitResult<Vec<VertexId<S::Id>>> {
        if view.vertex(start).is_none() {
            return Err(VisitError::VertexNotFound);
        }

        let mut marks = VertexMarks::new(Mark::Unvisited);
        let mut order = Vec::new();
        let mut queue = VecDeque::new();

        marks.set(start, Mark::Visited);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            order.push(current);
            for edge in view.edges().of(current) {
                if *marks.get(edge.to) == Mark::Unvisited {
                    marks.set(edge.to, Mark::Visited);
                    queue.push_back(edge.to);
                }
            }
        }

        Ok(order)
    }
}
