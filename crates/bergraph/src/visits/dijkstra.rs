//! Dijkstra shortest-path visit, available for any graph, directed or undirected.

use std::collections::BinaryHeap;

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, VertexMarks, VisitError, VisitResult},
};

#[derive(Debug, Clone, Copy)]
enum Mark {
    Unvisited,
    Distance(f64),
}

/// Shortest path from a start vertex to `target`: the vertex sequence and its total weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dijkstra<I: UnsignedId> {
    target: VertexId<I>,
}

impl<I: UnsignedId> Dijkstra<I> {
    pub fn new(target: VertexId<I>) -> Self {
        Self { target }
    }

    /// Runs the visit from `start`, `cost` giving the non-negative weight of each edge.
    pub fn visit<V: 'static, S: GraphEdgeStorage<Id = I>, G: GraphView<V, S>>(
        &self,
        view: &G,
        start: VertexId<I>,
        cost: impl Fn(&EdgeView<'_, S::Edge, I, S::EdgeKey>) -> f64,
    ) -> VisitResult<Option<(Vec<VertexId<I>>, f64)>> {
        if view.vertex(start).is_none() || view.vertex(self.target).is_none() {
            return Err(VisitError::VertexNotFound);
        }

        let mut marks = VertexMarks::new(Mark::Unvisited);
        let mut previous: VertexMarks<Option<VertexId<S::Id>>> = VertexMarks::new(None);
        let mut heap = BinaryHeap::new();

        marks.set(start, Mark::Distance(0.0));
        heap.push(MinEntry {
            priority: 0.0,
            vertex: start,
        });

        while let Some(MinEntry {
            priority: distance,
            vertex,
        }) = heap.pop()
        {
            if vertex == self.target {
                break;
            }
            if matches!(marks.get(vertex), Mark::Distance(known) if *known < distance) {
                continue;
            }

            for edge in view.edges().of(vertex) {
                let next_distance = distance + cost(&edge);
                let is_shorter = match marks.get(edge.to) {
                    Mark::Unvisited => true,
                    Mark::Distance(known) => next_distance < *known,
                };
                if is_shorter {
                    marks.set(edge.to, Mark::Distance(next_distance));
                    previous.set(edge.to, Some(vertex));
                    heap.push(MinEntry {
                        priority: next_distance,
                        vertex: edge.to,
                    });
                }
            }
        }

        let Mark::Distance(total) = *marks.get(self.target) else {
            return Ok(None);
        };

        let mut path = vec![self.target];
        while let Some(prev) = *previous.get(*path.last().unwrap()) {
            path.push(prev);
        }
        path.reverse();

        Ok(Some((path, total)))
    }
}
