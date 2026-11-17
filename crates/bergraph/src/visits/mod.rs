//! Generic traversal (visit) framework, running over any [`GraphView`] rather than a
//! concrete `Graph`.
//!
//! A visit is bound to the kind of graph it supports through the storage bound `S`:
//! a bound of `S: GraphEdgeStorage` accepts any graph, directed or undirected,
//! while `S: GraphEdgeStorageDirected` restricts the visit to graphs whose storage
//! exposes direction-aware navigation ([`children_of`](crate::edges::GraphEdgeStorageDirected::children_of) /
//! [`parents_of`](crate::edges::GraphEdgeStorageDirected::parents_of)).

mod astar;
mod bellman_ford;
mod bfs;
mod bfs_to_any;
mod biconnected;
mod bipartite;
mod bridges;
mod centrality;
mod connected_components;
mod cycle;
mod dfs;
mod dijkstra;
mod eccentricity;
mod eulerian;
mod max_flow;
mod mst;
mod multi_source_bfs;
mod multi_source_dijkstra;
mod priority_flood;
mod scc;
mod simple_paths;
mod topological_sort;
mod transitive_closure;

pub use astar::AStar;
pub use bellman_ford::BellmanFord;
pub use bfs::Bfs;
pub use bfs_to_any::BfsToAny;
pub use biconnected::{BiconnectedComponents, BiconnectedEdges};
pub use bipartite::{Bipartite, Bipartition};
pub use bridges::{ArticulationPoints, BridgeEdges, Bridges};
pub use centrality::{BetweennessCentrality, Centrality};
pub use connected_components::{Components, ConnectedComponents};
pub use cycle::CycleDetection;
pub use dfs::Dfs;
pub use dijkstra::Dijkstra;
pub use eccentricity::{Eccentricities, Eccentricity};
pub use eulerian::EulerianTrail;
pub use max_flow::{MaxFlow, MinCutEdges};
pub use mst::{MinimumSpanningTree, MstEdges};
pub use multi_source_bfs::{MultiSourceBfs, Reached, Reaching};
pub use multi_source_dijkstra::{DijkstraReached, DijkstraReaching, MultiSourceDijkstra};
pub use priority_flood::{Flooded, Flooding, PriorityFlood};
pub use scc::{SccComponents, StronglyConnectedComponents};
pub use simple_paths::AllSimplePaths;
pub use topological_sort::TopologicalSort;
pub use transitive_closure::{Reachability, TransitiveClosure};

use std::{cmp::Ordering, collections::BinaryHeap, fmt};

use crate::vertices::{UnsignedId, VertexId};

/// Result of a visit.
pub type VisitResult<T> = Result<T, VisitError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Why a visit could not run.
pub enum VisitError {
    /// A vertex the visit needs is not in the view.
    VertexNotFound,
    /// A negative-weight cycle is reachable from the start vertex, so no shortest path is
    /// well-defined: its cost can be driven arbitrarily low by looping through it.
    NegativeCycle,
}

impl fmt::Display for VisitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::VertexNotFound => "vertex not found in the graph view",
            Self::NegativeCycle => "negative-weight cycle reachable from the start vertex",
        })
    }
}

impl std::error::Error for VisitError {}

/// Visited/unvisited marker shared by the visits that don't need extra per-vertex data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    Unvisited,
    Visited,
}

/// Per-vertex marking storage shared by visits, backed by a densely-indexed growable array.
/// Each visit picks its own mark type `M` (a visited/unvisited enum, a distance, ...).
#[derive(Debug)]
pub struct VertexMarks<M> {
    marks: Vec<M>,
    default: M,
}

impl<M: Clone> VertexMarks<M> {
    pub fn new(default: M) -> Self {
        Self {
            marks: Vec::new(),
            default,
        }
    }

    pub fn get<I: UnsignedId>(&self, id: VertexId<I>) -> &M {
        self.marks.get(id.id()).unwrap_or(&self.default)
    }

    pub fn set<I: UnsignedId>(&mut self, id: VertexId<I>, mark: M) {
        let idx = id.id();
        if idx >= self.marks.len() {
            self.marks.resize(idx + 1, self.default.clone());
        }
        self.marks[idx] = mark;
    }

    pub fn get_mut<I: UnsignedId>(&mut self, id: VertexId<I>) -> &mut M {
        let idx = id.id();
        if idx >= self.marks.len() {
            self.marks.resize(idx + 1, self.default.clone());
        }
        &mut self.marks[idx]
    }
}

/// Min-heap entry of the priority searches: smallest `priority` first (by `total_cmp`), ties
/// broken by the lower vertex id, so the pop order never depends on the heap's internals.
#[derive(Debug)]
pub(crate) struct MinEntry<I: UnsignedId> {
    pub(crate) priority: f64,
    pub(crate) vertex: VertexId<I>,
}

impl<I: UnsignedId> PartialEq for MinEntry<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<I: UnsignedId> Eq for MinEntry<I> {}
impl<I: UnsignedId> PartialOrd for MinEntry<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<I: UnsignedId> Ord for MinEntry<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed: BinaryHeap is a max-heap.
        other
            .priority
            .total_cmp(&self.priority)
            .then_with(|| other.vertex.cmp(&self.vertex))
    }
}

/// Per-vertex arrays and heap of the shortest-path searches, reusable across `visit_with` calls:
/// each search resets only the vertices the previous one reached, not the whole graph.
#[derive(Debug)]
pub struct SearchBuffers<I: UnsignedId> {
    distance: VertexMarks<Option<f64>>,
    // The predecessor on a path search, the source on a multi-source search.
    previous: VertexMarks<Option<VertexId<I>>>,
    heap: BinaryHeap<MinEntry<I>>,
    reached: Vec<VertexId<I>>,
}

impl<I: UnsignedId> Default for SearchBuffers<I> {
    fn default() -> Self {
        Self {
            distance: VertexMarks::new(None),
            previous: VertexMarks::new(None),
            heap: BinaryHeap::new(),
            reached: Vec::new(),
        }
    }
}

impl<I: UnsignedId> SearchBuffers<I> {
    fn clear(&mut self) {
        for vertex in self.reached.drain(..) {
            self.distance.set(vertex, None);
            self.previous.set(vertex, None);
        }
        self.heap.clear();
    }

    /// Records `distance` and `previous` for `vertex` and queues it at `priority`.
    fn reach(
        &mut self,
        vertex: VertexId<I>,
        distance: f64,
        previous: Option<VertexId<I>>,
        priority: f64,
    ) {
        if self.distance.get(vertex).is_none() {
            self.reached.push(vertex);
        }
        self.distance.set(vertex, Some(distance));
        self.previous.set(vertex, previous);
        self.heap.push(MinEntry { priority, vertex });
    }
}

/// Rebuilds the vertex sequence from `target` back to its start, by walking the `previous`
/// pointers a shortest-path visit set while relaxing edges, then reversing them into order.
pub(crate) fn reconstruct_path<I: UnsignedId>(
    target: VertexId<I>,
    previous: &VertexMarks<Option<VertexId<I>>>,
) -> Vec<VertexId<I>> {
    let mut path = vec![target];
    while let Some(prev) = *previous.get(*path.last().unwrap()) {
        path.push(prev);
    }
    path.reverse();
    path
}
