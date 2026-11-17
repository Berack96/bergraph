//! Struct central to the library, representing a graph and its associated data structures.

#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::{
    edges::{EdgeView, GraphEdgeStorage},
    vertices::VertexId,
    views::GraphFilteredView,
};

pub trait GraphView<V: 'static, S: GraphEdgeStorage> {
    fn vertex(&self, id: VertexId<S::Id>) -> Option<&V>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn edges(&self) -> &S;

    /// Ids of every vertex the view exposes, i.e. exactly those for which
    /// [`vertex`](Self::vertex) returns `Some`. Used by whole-graph visits (connected
    /// components, SCC, ...) that need to enumerate vertices rather than start from one.
    fn ids(&self) -> impl Iterator<Item = VertexId<S::Id>>;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct Graph<V, S> {
    pub(crate) vertices: Vec<V>,
    pub(crate) edge_storage: S,
}

impl<V, S: GraphEdgeStorage> Graph<V, S> {
    /// Graph over `vertices`, vertex `i` being `VertexId::new(i)` in `edge_storage`. Not checked:
    /// edges to ids past `vertices.len()` stay reachable by visits, `vertex()` returns `None` there.
    pub fn new(vertices: Vec<V>, edge_storage: S) -> Self {
        Self {
            vertices,
            edge_storage,
        }
    }

    pub fn add_vertex(&mut self, vertex: V) -> VertexId<S::Id> {
        self.vertices.push(vertex);
        VertexId::new(self.vertices.len() - 1)
    }

    pub fn edges_mut(&mut self) -> &mut S {
        &mut self.edge_storage
    }

    pub fn vertex_mut(&mut self, id: VertexId<S::Id>) -> Option<&mut V> {
        self.vertices.get_mut(id.id())
    }

    /// View of the whole graph, with no vertex or edge filtered out.
    // The closure types can't be named, so no type alias can shorten this signature.
    #[allow(clippy::type_complexity)]
    pub fn view(
        &self,
    ) -> GraphFilteredView<
        '_,
        V,
        S,
        impl Fn(VertexId<S::Id>, &V) -> bool,
        impl Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
    >
    where
        V: 'static,
    {
        self.filtered_view(|_, _| true, |_| true)
    }

    /// View of the vertices accepted by `filter_vertices` and the edges accepted by
    /// `filter_edges` between them.
    pub fn filtered_view<FV, FE>(
        &self,
        filter_vertices: FV,
        filter_edges: FE,
    ) -> GraphFilteredView<'_, V, S, FV, FE>
    where
        V: 'static,
        FV: Fn(VertexId<S::Id>, &V) -> bool,
        FE: Fn(&EdgeView<'_, S::Edge, S::Id, S::EdgeKey>) -> bool,
    {
        GraphFilteredView::new(self, filter_vertices, filter_edges)
    }
}

impl<V: 'static, S: GraphEdgeStorage> GraphView<V, S> for Graph<V, S> {
    fn len(&self) -> usize {
        self.vertices.len()
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    fn vertex(&self, id: VertexId<S::Id>) -> Option<&V> {
        self.vertices.get(id.id())
    }

    fn edges(&self) -> &S {
        &self.edge_storage
    }

    fn ids(&self) -> impl Iterator<Item = VertexId<S::Id>> {
        (0..self.vertices.len()).map(VertexId::new)
    }
}
