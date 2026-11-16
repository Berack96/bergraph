//! A graph tool library for building memory efficient and fast graphs.

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

pub mod graph_edges {
    pub use super::edges::{
        AdjCsr, AdjList, AdjListFixed, AdjMatrix, CsrError, Edge, EdgeSimple, GraphEdgeStorage,
        GraphEdgeStorageDirected, GraphEdgeStorageDirectedMut, GraphEdgeStorageMut, Weighted,
    };
}

pub mod graph_view {
    pub use super::edges::EdgeView;
    pub use super::graph::GraphView;
    pub use super::views::{EdgeFilteredView, GraphFilteredView};
}

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
