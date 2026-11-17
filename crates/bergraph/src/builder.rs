use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    edges::{
        AdjCsr, AdjList, AdjListFixed, AdjMatrix, CsrError, EdgeId, GraphEdgeStorageMut, MaybeSerde,
    },
    graph::Graph,
    vertices::{UnsignedId, VertexId},
};

/// Builds a [`Graph`], choosing its vertex id type `I` and its edge storage `S`, an [`AdjList`] by
/// default.
pub struct GraphBuilder<V, E, I = u32, S = AdjList<E, I>> {
    vertices: Option<Vec<V>>,
    _marker: PhantomData<(V, E, I, S)>,
}

impl<V: Debug, E, I, S> Debug for GraphBuilder<V, E, I, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphBuilder")
            .field("vertices", &self.vertices)
            .finish_non_exhaustive()
    }
}

// MaybeSerde is intentionally crate-internal (see its doc comment); it only shapes what
// E/I must satisfy, it isn't part of the public API surface.
#[allow(private_bounds)]
impl<V, E: MaybeSerde> GraphBuilder<V, E> {
    /// Builder with `u32` vertex ids and an [`AdjList`] storage.
    pub fn new() -> Self {
        Self {
            vertices: None,
            _marker: PhantomData,
        }
    }
}

// `new()` generalized over `I`, for callers who need a narrower id type than the u32 default
// (e.g. to match an existing u16 index) instead of the type inference `new()` relies on.
#[allow(private_bounds)]
impl<V, E: MaybeSerde, I: UnsignedId + MaybeSerde> GraphBuilder<V, E, I, AdjList<E, I>> {
    /// Builder with vertex id type `I` and an [`AdjList`] storage.
    pub fn new_with_id() -> Self {
        Self {
            vertices: None,
            _marker: PhantomData,
        }
    }
}

#[allow(private_bounds)]
impl<V, E: MaybeSerde> Default for GraphBuilder<V, E> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(private_bounds)]
impl<V, E: MaybeSerde, I, S> GraphBuilder<V, E, I, S>
where
    I: UnsignedId + MaybeSerde,
    S: GraphEdgeStorageMut<Edge = E, Id = I>,
{
    /// Switches to the fixed-capacity adjacency list, with at most `N` outgoing edges per vertex.
    pub fn into_adjlist_fixed_max_degree<const N: usize>(
        self,
    ) -> GraphBuilder<V, E, I, AdjListFixed<E, I, N>> {
        GraphBuilder {
            vertices: self.vertices,
            _marker: PhantomData,
        }
    }

    /// Switch explicitly back to the dynamic adjacency list.
    pub fn into_adjlist(self) -> GraphBuilder<V, E, I, AdjList<E, I>> {
        GraphBuilder {
            vertices: self.vertices,
            _marker: PhantomData,
        }
    }

    /// Switch to the dense adjacency matrix storage.
    pub fn into_matrix(self) -> GraphBuilder<V, E, I, AdjMatrix<E, I>> {
        GraphBuilder {
            vertices: self.vertices,
            _marker: PhantomData,
        }
    }

    /// Starts the graph with `vertices`, vertex `i` getting `VertexId::new(i)`.
    pub fn with_vertices(mut self, vertices: Vec<V>) -> Self {
        self.vertices = Some(vertices);
        self
    }

    /// Builds the graph: the given vertices and an empty storage sized for them.
    pub fn build(self) -> Graph<V, S> {
        let vertices = self.vertices.unwrap_or_default();
        let size = vertices.len();
        Graph::new(vertices, S::with_capacity(size))
    }
}

#[allow(private_bounds)]
impl<V, E, I, S> GraphBuilder<V, E, I, S>
where
    E: Clone + 'static + MaybeSerde,
    I: UnsignedId + MaybeSerde,
{
    /// Builds the graph on a directed [`AdjCsr`] over the given vertices, as
    /// [`AdjCsr::directed`] with the vertex count taken from them; returns each input edge's id.
    ///
    /// # Errors
    ///
    /// The [`CsrError`] of [`AdjCsr::directed`].
    #[allow(clippy::type_complexity)]
    pub fn build_csr_directed<EI: UnsignedId + MaybeSerde>(
        self,
        edges: &[(VertexId<I>, VertexId<I>, E)],
    ) -> Result<(Graph<V, AdjCsr<E, I, EI>>, Vec<EdgeId<EI>>), CsrError> {
        let vertices = self.vertices.unwrap_or_default();
        let (csr, ids) = AdjCsr::directed(vertices.len(), edges)?;
        Ok((Graph::new(vertices, csr), ids))
    }

    /// Builds the graph on an undirected [`AdjCsr`] over the given vertices, as
    /// [`AdjCsr::undirected`] with the vertex count taken from them; returns each input edge's id.
    ///
    /// # Errors
    ///
    /// The [`CsrError`] of [`AdjCsr::undirected`].
    #[allow(clippy::type_complexity)]
    pub fn build_csr_undirected<EI: UnsignedId + MaybeSerde>(
        self,
        edges: &[(VertexId<I>, VertexId<I>, E)],
    ) -> Result<(Graph<V, AdjCsr<E, I, EI>>, Vec<EdgeId<EI>>), CsrError> {
        let vertices = self.vertices.unwrap_or_default();
        let (csr, ids) = AdjCsr::undirected(vertices.len(), edges)?;
        Ok((Graph::new(vertices, csr), ids))
    }
}
