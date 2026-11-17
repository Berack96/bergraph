//! Struct central to the library, representing a graph and its associated data structures.

#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::{
    edges::{EdgeView, GraphEdgeStorage, GraphEdgeStorageMut},
    vertices::VertexId,
    views::GraphFilteredView,
};

/// Read access to a graph's vertices and edges, implemented by [`Graph`] and its filtered views.
pub trait GraphView<V: 'static, S: GraphEdgeStorage> {
    /// Data of vertex `id`, `None` if it is not in the view.
    fn vertex(&self, id: VertexId<S::Id>) -> Option<&V>;
    /// Number of vertices.
    fn len(&self) -> usize;
    /// Whether the view has no vertices.
    fn is_empty(&self) -> bool;
    /// Edge storage of the view.
    fn edges(&self) -> &S;

    /// Ids of every vertex the view exposes, i.e. exactly those for which
    /// [`vertex`](Self::vertex) returns `Some`. Used by whole-graph visits (connected
    /// components, SCC, ...) that need to enumerate vertices rather than start from one.
    fn ids(&self) -> impl Iterator<Item = VertexId<S::Id>>;

    /// One past the highest vertex id of the underlying graph: every id the view exposes is below it.
    fn id_bound(&self) -> usize;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde_feature)]
/// Vertices with their data `V`, connected by the edges of the storage `S`.
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

    /// Adds a vertex with no edges and returns its id.
    ///
    /// # Panics
    ///
    /// Panics if the new vertex's id does not fit the id type.
    pub fn add_vertex(&mut self, vertex: V) -> VertexId<S::Id> {
        self.vertices.push(vertex);
        VertexId::new(self.vertices.len() - 1)
    }

    /// Mutable access to the edge storage.
    pub fn edges_mut(&mut self) -> &mut S {
        &mut self.edge_storage
    }

    /// Mutable data of vertex `id`, `None` if it is absent.
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

impl<V, S: Default> Default for Graph<V, S> {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            edge_storage: S::default(),
        }
    }
}

/// Graph over the collected vertices, with no edges yet.
impl<V, S: GraphEdgeStorageMut> FromIterator<V> for Graph<V, S> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let vertices: Vec<V> = iter.into_iter().collect();
        let size = vertices.len();
        Self::new(vertices, S::with_capacity(size))
    }
}

/// Appends vertices, with no edges, the same as repeated [`add_vertex`](Graph::add_vertex).
impl<V, S> Extend<V> for Graph<V, S> {
    fn extend<T: IntoIterator<Item = V>>(&mut self, iter: T) {
        self.vertices.extend(iter);
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

    fn id_bound(&self) -> usize {
        self.vertices.len()
    }
}
