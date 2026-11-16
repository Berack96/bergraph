use crate::{
    VertexId,
    edges::{EdgeView, GraphEdgeStorage, GraphEdgeStorageDirected},
    views::{EdgeFilter, VertexFilter},
};

pub struct EdgeFilteredView<'a, V, S: GraphEdgeStorage> {
    edge_storage: &'a S,
    filter_edges: Option<EdgeFilter<'a, S::Edge>>,
    vertices: &'a [V],
    filter_vertices: Option<VertexFilter<'a, V>>,
}

impl<'a, V: 'static, S: GraphEdgeStorage> EdgeFilteredView<'a, V, S> {
    pub(crate) fn new(
        edge_storage: &'a S,
        filter_edges: Option<EdgeFilter<'a, S::Edge>>,
        vertices: &'a [V],
        filter_vertices: Option<VertexFilter<'a, V>>,
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
        self.vertices.get(id.id()).is_some_and(|v| {
            self.filter_vertices
                .as_deref()
                .is_none_or(|filter| filter(v))
        })
    }

    fn edge_in_view(&self, edge: &EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool {
        self.filter_edges
            .as_deref()
            .is_none_or(|filter| filter(edge.data))
            && self.vertex_in_view(edge.from)
            && self.vertex_in_view(edge.to)
    }
}

impl<'a, V: 'static, S: GraphEdgeStorage> GraphEdgeStorage for EdgeFilteredView<'a, V, S> {
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
        if !self.vertex_in_view(*from) || !self.vertex_in_view(*to) {
            return None;
        }
        self.edge_storage.get(from, to).filter(|data| {
            self.filter_edges
                .as_deref()
                .is_none_or(|filter| filter(data))
        })
    }

    fn has_edge(&self, from: &VertexId<S::Id>, to: &VertexId<S::Id>) -> bool {
        self.get(from, to).is_some()
    }
}

impl<'a, V: 'static, S: GraphEdgeStorageDirected> GraphEdgeStorageDirected
    for EdgeFilteredView<'a, V, S>
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
