//! Undirected connected-components visit, available for any graph, directed or undirected.

use crate::{
    edges::GraphEdgeStorage,
    graph::GraphView,
    vertices::VertexId,
    visits::{Mark, VertexMarks},
};

/// One entry per connected component, each holding its member vertices.
pub type Components<I> = Vec<Vec<VertexId<I>>>;

/// Partitions every vertex the view exposes into connected components, treating
/// [`GraphEdgeStorage::of`] as an undirected adjacency: on a graph built one direction at a
/// time via [`add_edge_directed`](crate::edges::GraphEdgeStorageDirectedMut::add_edge_directed),
/// this follows outgoing edges only, same as [`Bfs`](super::Bfs). To restrict which vertices
/// or edges count, run this over a [`GraphFilteredView`](crate::graph_view::GraphFilteredView)
/// instead of filtering inside the visit.
#[derive(Debug, Clone, Copy)]
pub struct ConnectedComponents;

impl ConnectedComponents {
    /// Runs the visit over the whole `view`.
    pub fn visit<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
    ) -> Components<S::Id> {
        let mut marks = VertexMarks::new(Mark::Unvisited);
        let mut components = Vec::new();

        for start in view.ids() {
            if *marks.get(start) == Mark::Visited {
                continue;
            }

            let mut component = Vec::new();
            let mut stack = vec![start];
            marks.set(start, Mark::Visited);

            while let Some(current) = stack.pop() {
                component.push(current);
                for edge in view.edges().of(current) {
                    if *marks.get(edge.to) == Mark::Unvisited {
                        marks.set(edge.to, Mark::Visited);
                        stack.push(edge.to);
                    }
                }
            }
            components.push(component);
        }

        components
    }

    /// Labels each vertex the view exposes with the first vertex of its component, in `ids`
    /// order; indexed by vertex id, `None` or past the end outside the view.
    pub fn labels<V: 'static, S: GraphEdgeStorage, G: GraphView<V, S>>(
        &self,
        view: &G,
    ) -> Vec<Option<VertexId<S::Id>>> {
        let mut labels = VertexMarks::new(None);

        for start in view.ids() {
            if labels.get(start).is_some() {
                continue;
            }
            let mut stack = vec![start];
            labels.set(start, Some(start));
            while let Some(current) = stack.pop() {
                for edge in view.edges().of(current) {
                    if labels.get(edge.to).is_none() {
                        labels.set(edge.to, Some(start));
                        stack.push(edge.to);
                    }
                }
            }
        }

        labels.marks
    }
}
