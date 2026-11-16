//! Multi-source Dijkstra visit, available for any graph, directed or undirected.

use std::collections::BinaryHeap;

use crate::{
    edges::{GraphEdgeStorage, Weighted},
    graph::GraphView,
    vertices::{UnsignedId, VertexId},
    visits::{MinEntry, VertexMarks, VisitError, VisitResult},
};

/// Total edge weight from, and identity of, the source that reaches a vertex at the lowest cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DijkstraReached<I: UnsignedId> {
    pub distance: f64,
    pub source: VertexId<I>,
}

/// One `(vertex, reached)` entry per vertex a multi-source Dijkstra visit reached.
pub type DijkstraReaching<I> = Vec<(VertexId<I>, DijkstraReached<I>)>;

/// Dijkstra expansion from several start vertices at once: every vertex is attributed to
/// whichever source reaches it at the lowest total edge weight, ties broken by the lower vertex
/// id, rather than by running one Dijkstra per source and merging the results.
#[derive(Debug, Clone, Copy)]
pub struct MultiSourceDijkstra;

impl MultiSourceDijkstra {
    /// Runs the visit; the result lists every reached vertex in the order its shortest
    /// distance was finalized. Fails if any `sources` vertex is missing from `view`.
    pub fn visit<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
        sources: impl IntoIterator<Item = VertexId<S::Id>>,
    ) -> VisitResult<DijkstraReaching<S::Id>>
    where
        S::Edge: Weighted,
    {
        let mut marks: VertexMarks<Option<DijkstraReached<S::Id>>> = VertexMarks::new(None);
        let mut heap = BinaryHeap::new();

        for source in sources {
            if view.vertex(source).is_none() {
                return Err(VisitError::VertexNotFound);
            }
            if marks.get(source).is_some() {
                continue;
            }
            marks.set(
                source,
                Some(DijkstraReached {
                    distance: 0.0,
                    source,
                }),
            );
            heap.push(MinEntry {
                priority: 0.0,
                vertex: source,
            });
        }

        let mut order = Vec::new();
        while let Some(MinEntry {
            priority: distance,
            vertex,
        }) = heap.pop()
        {
            let current = marks
                .get(vertex)
                .expect("queued vertices are always marked");
            if distance > current.distance {
                continue; // stale entry, superseded by a shorter path found since
            }
            let source = current.source;
            order.push((vertex, current));

            for edge in view.edges().of(vertex) {
                let next_distance = distance + edge.data.weight();
                let is_shorter = match marks.get(edge.to) {
                    None => true,
                    Some(known) => next_distance < known.distance,
                };
                if is_shorter {
                    marks.set(
                        edge.to,
                        Some(DijkstraReached {
                            distance: next_distance,
                            source,
                        }),
                    );
                    heap.push(MinEntry {
                        priority: next_distance,
                        vertex: edge.to,
                    });
                }
            }
        }

        Ok(order)
    }
}
