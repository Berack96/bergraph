//! A graph tool library for building memory efficient and fast graphs.
//!
//! ```
//! use bergraph::{GraphBuilder, graph_edges::GraphEdgeStorageDirectedMut, graph_visit::Dijkstra};
//!
//! let mut graph = GraphBuilder::<&str, f64>::new().build();
//! let a = graph.add_vertex("a");
//! let b = graph.add_vertex("b");
//! graph.edges_mut().add_edge_directed(a, b, 2.5);
//!
//! let shortest = Dijkstra::new(b).visit(&graph, a, |edge| *edge.data).unwrap();
//! assert_eq!(shortest, Some((vec![a, b], 2.5)));
//! ```

mod builder;
mod edges;
mod graph;
mod vertices;
mod views;
mod visits;

pub use builder::GraphBuilder;
pub use edges::EdgeId;
pub use graph::Graph;
pub use vertices::VertexId;

/// Edge storages and their traits.
pub mod graph_edges {
    pub use super::edges::{
        AdjCsr, AdjList, AdjListFixed, AdjMatrix, CsrError, Edge, EdgeSimple, GraphEdgeStorage,
        GraphEdgeStorageDirected, GraphEdgeStorageDirectedMut, GraphEdgeStorageMut, Weighted,
    };
}

/// Read access to graphs, and filtered views.
pub mod graph_view {
    pub use super::edges::EdgeView;
    pub use super::graph::GraphView;
    pub use super::views::{EdgeFilteredView, GraphFilteredView};
}

/// Graph algorithms.
pub mod graph_visit {
    pub use super::visits::{
        AStar, AllSimplePaths, ArticulationPoints, BellmanFord, BetweennessCentrality, Bfs,
        BfsToAny, BiconnectedComponents, BiconnectedEdges, Bipartite, Bipartition, BridgeEdges,
        Bridges, Centrality, Components, ConnectedComponents, CycleDetection, Dfs, Dijkstra,
        DijkstraReached, DijkstraReaching, Eccentricities, Eccentricity, EulerianTrail, Flooded,
        Flooding, MaxFlow, MinCutEdges, MinimumSpanningTree, MstEdges, MultiSourceBfs,
        MultiSourceDijkstra, PriorityFlood, Reachability, Reached, Reaching, SccComponents,
        SearchBuffers, StronglyConnectedComponents, TopologicalSort, TransitiveClosure, VisitError,
        VisitResult,
    };
}
