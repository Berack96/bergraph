use crate::{
    VertexId,
    edges::{EdgeView, GraphEdgeStorage, GraphEdgeStorageDirected},
};

/// Edge storage seen through a vertex predicate `FV` and an edge predicate `FE`.
pub struct EdgeFilteredView<'a, V, S: GraphEdgeStorage, FV, FE> {
    edge_storage: &'a S,
    filter_edges: FE,
    vertices: &'a [V],
    filter_vertices: FV,
}

impl<'a, V: 'static, S: GraphEdgeStorage, FV, FE> EdgeFilteredView<'a, V, S, FV, FE>
where
    FV: Fn(&V) -> bool,
    FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
{
    pub(crate) fn new(
        edge_storage: &'a S,
        filter_edges: FE,
        vertices: &'a [V],
        filter_vertices: FV,
    ) -> Self {
        Self {
            edge_storage,
            filter_edges,
            vertices,
            filter_vertices,
        }
    }

    /// Whether `id` names a vertex both present in `vertices` and accepted by
    /// `filter_vertices`: an edge reaching outside this is no more "in the view" than one
    /// `filter_edges` rejects.
    pub(crate) fn vertex_in_view(&self, id: VertexId<S::Id>) -> bool {
        self.vertices
            .get(id.id())
            .is_some_and(|v| (self.filter_vertices)(v))
    }

    fn edge_in_view(&self, edge: &EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool {
        (self.filter_edges)(edge) && self.vertex_in_view(edge.from) && self.vertex_in_view(edge.to)
    }
}

impl<'a, V: 'static, S: GraphEdgeStorage, FV, FE> GraphEdgeStorage
    for EdgeFilteredView<'a, V, S, FV, FE>
where
    FV: Fn(&V) -> bool,
    FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
{
    type Edge = S::Edge;
    type Id = S::Id;
    type EdgeKey = S::EdgeKey;

    fn all(&self) -> impl Iterator<Item = EdgeView<'_, S::Edge, S::Id, S::EdgeKey>> {
        self.edge_storage
            .all()
            .filter(|edge| self.edge_in_view(edge))
    }

    fn count(&self) -> usize {
        self.all().count()
    }

    fn of(
        &self,
        id: VertexId<S::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, S::Edge, S::Id, S::EdgeKey>> {
        self.edge_storage
            .of(id)
            .filter(|edge| self.edge_in_view(edge))
    }

    fn count_of(&self, id: VertexId<S::Id>) -> usize {
        self.of(id).count()
    }

    fn get(&self, from: &VertexId<S::Id>, to: &VertexId<S::Id>) -> Option<&S::Edge> {
        // shortcut: scans `of(from)` (O(V) on `AdjMatrix`) to give the edge filter its key; switch
        // to a keyed `get` on the storage if filtered lookups get hot.
        self.of(*from)
            .find(|edge| edge.to == *to)
            .map(|edge| edge.data)
    }

    fn has_edge(&self, from: &VertexId<S::Id>, to: &VertexId<S::Id>) -> bool {
        self.get(from, to).is_some()
    }
}

impl<'a, V: 'static, S: GraphEdgeStorageDirected, FV, FE> GraphEdgeStorageDirected
    for EdgeFilteredView<'a, V, S, FV, FE>
where
    FV: Fn(&V) -> bool,
    FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
{
    fn children_of(
        &self,
        id: VertexId<S::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, S::Edge, S::Id, S::EdgeKey>> {
        self.edge_storage
            .children_of(id)
            .filter(|edge| self.edge_in_view(edge))
    }

    fn parents_of(
        &self,
        id: VertexId<S::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, S::Edge, S::Id, S::EdgeKey>> {
        self.edge_storage
            .parents_of(id)
            .filter(|edge| self.edge_in_view(edge))
    }

    fn count_incoming(&self, id: VertexId<S::Id>) -> usize {
        self.parents_of(id).count()
    }

    fn count_outgoing(&self, id: VertexId<S::Id>) -> usize {
        self.children_of(id).count()
    }
}
