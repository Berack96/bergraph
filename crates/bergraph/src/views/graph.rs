use std::fmt::{self, Debug, Formatter};

use crate::{
    Graph, VertexId,
    edges::{EdgeView, GraphEdgeStorage},
    graph::GraphView,
    views::edges::EdgeFilteredView,
};

/// Graph seen through a vertex predicate `FV` and an edge predicate `FE`; `Send`/`Sync` whenever
/// the predicates and the graph are.
pub struct GraphFilteredView<'a, V, S: GraphEdgeStorage, FV, FE> {
    graph: &'a Graph<V, S>,
    edges: EdgeFilteredView<'a, V, S, FV, FE>,
}

impl<V: Debug, S: GraphEdgeStorage + Debug, FV, FE> Debug for GraphFilteredView<'_, V, S, FV, FE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphFilteredView")
            .field("graph", &self.graph)
            .finish_non_exhaustive()
    }
}

impl<'a, V: 'static, S: GraphEdgeStorage, FV, FE> GraphFilteredView<'a, V, S, FV, FE>
where
    FV: Fn(&V) -> bool,
    FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
{
    pub fn new(graph: &'a Graph<V, S>, filter_vertices: FV, filter_edges: FE) -> Self {
        Self {
            graph,
            edges: EdgeFilteredView::new(
                &graph.edge_storage,
                filter_edges,
                &graph.vertices,
                filter_vertices,
            ),
        }
    }
}

impl<'a, V: 'static, S: GraphEdgeStorage, FV, FE> GraphView<V, EdgeFilteredView<'a, V, S, FV, FE>>
    for GraphFilteredView<'a, V, S, FV, FE>
where
    FV: Fn(&V) -> bool,
    FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
{
    fn vertex(&self, id: VertexId<S::Id>) -> Option<&V> {
        self.graph
            .vertex(id)
            .filter(|_| self.edges.vertex_in_view(id))
    }

    fn len(&self) -> usize {
        self.ids().count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn edges(&self) -> &EdgeFilteredView<'a, V, S, FV, FE> {
        &self.edges
    }

    fn ids(&self) -> impl Iterator<Item = VertexId<S::Id>> {
        (0..self.graph.vertices.len())
            .map(VertexId::new)
            .filter(move |&id| self.vertex(id).is_some())
    }
}
