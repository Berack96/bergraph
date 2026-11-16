mod adj_list;
mod csr;
mod fixed;
mod matrix;

pub use adj_list::AdjList;
pub use csr::{AdjCsr, CsrError};
pub use fixed::AdjListFixed;
pub use matrix::AdjMatrix;

#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::vertices::{UnsignedId, VertexId};

/// No-op bound when `serde` is disabled, `Serialize + DeserializeOwned` when it's enabled.
/// A concrete storage's `impl GraphEdgeStorage` must satisfy `Edge`/`Id`'s own bound (see
/// below); Rust has no way to `#[cfg]` a single predicate inside one `where` clause, so this
/// lets that requirement be written once, unconditionally, instead of duplicating the impl.
#[cfg(feature = "serde")]
pub(crate) trait MaybeSerde: serde::Serialize + serde::de::DeserializeOwned {}
#[cfg(feature = "serde")]
impl<T: serde::Serialize + serde::de::DeserializeOwned> MaybeSerde for T {}

#[cfg(not(feature = "serde"))]
pub(crate) trait MaybeSerde {}
#[cfg(not(feature = "serde"))]
impl<T: ?Sized> MaybeSerde for T {}

/// Edge data usable as a Dijkstra edge weight.
/// `weight` must be non-negative: `Dijkstra` does not terminate on negative-weight cycles.
pub trait Weighted {
    fn weight(&self) -> f64;
}

/// Read side of every edge storage, independent of whether it is also capable of directed
/// navigation ([`GraphEdgeStorageDirected`]) or of writes ([`GraphEdgeStorageMut`]).
#[cfg_attr(feature = "serde", serde_feature)]
pub trait GraphEdgeStorage {
    type Edge: 'static;
    type Id: UnsignedId;
    /// Identifies an edge in the [`EdgeView`]s: an [`EdgeId`] on storages with dense edge ids, `()` otherwise.
    type EdgeKey: Copy;

    fn all(&self) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    fn count(&self) -> usize;

    fn of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    fn count_of(&self, id: VertexId<Self::Id>) -> usize;

    fn get(&self, from: &VertexId<Self::Id>, to: &VertexId<Self::Id>) -> Option<&Self::Edge>;
    fn has_edge(&self, from: &VertexId<Self::Id>, to: &VertexId<Self::Id>) -> bool;
}

/// Write side of an edge storage. Every write here is undirected: `add_edge`/`remove_edge`
/// always act on both endpoints, so generic code written against this trait alone can never
/// end up with an asymmetric edge by accident — only
/// [`GraphEdgeStorageDirectedMut::add_edge_directed`] can create one, on a storage that opts
/// into it. `with_edges`/`add_all`/`remove_all` are the literal, non-mirrored bulk-load
/// counterparts (one arc per tuple, exactly as given).
pub trait GraphEdgeStorageMut: GraphEdgeStorage {
    fn with_capacity(capacity: usize) -> Self;
    fn with_edges(edges: Vec<Edge<Self::Edge, Self::Id>>) -> Self;

    /// Inserts an undirected edge: stored on both endpoints, so it is reachable from either
    /// side regardless of navigation direction. A self-loop (`from == to`) is stored once.
    /// Requires `Self::Edge: Clone` to populate both sides; storages whose edge data isn't
    /// `Clone` can still be built one direction at a time via
    /// [`GraphEdgeStorageDirectedMut::add_edge_directed`].
    fn add_edge(&mut self, from: VertexId<Self::Id>, to: VertexId<Self::Id>, data: Self::Edge)
    where
        Self::Edge: Clone;
    /// Removes an undirected edge, the mirror of [`add_edge`](Self::add_edge): clears both
    /// endpoints (best-effort on each side) and returns the data removed from the
    /// `from -> to` side.
    fn remove_edge(
        &mut self,
        from: VertexId<Self::Id>,
        to: VertexId<Self::Id>,
    ) -> Option<Self::Edge>;

    fn add_all(
        &mut self,
        from: VertexId<Self::Id>,
        edges: impl IntoIterator<Item = (VertexId<Self::Id>, Self::Edge)>,
    );
    fn remove_all(&mut self, from: VertexId<Self::Id>) -> Vec<(VertexId<Self::Id>, Self::Edge)>;
}

/// Extends [`GraphEdgeStorage`] with direction-aware navigation.
pub trait GraphEdgeStorageDirected: GraphEdgeStorage {
    fn children_of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    fn parents_of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;

    fn count_incoming(&self, id: VertexId<Self::Id>) -> usize;
    fn count_outgoing(&self, id: VertexId<Self::Id>) -> usize;
}

/// Single-direction writes. Implementing this trait is an opt-in promise that
/// `add_edge_directed`/`remove_edge_directed` touch only the given direction, while the
/// inherited `add_edge`/`remove_edge` must still keep behaving as fully undirected.
pub trait GraphEdgeStorageDirectedMut: GraphEdgeStorageDirected + GraphEdgeStorageMut {
    /// Inserts a single directed arc `from -> to`; the reverse direction is left untouched.
    fn add_edge_directed(
        &mut self,
        from: VertexId<Self::Id>,
        to: VertexId<Self::Id>,
        data: Self::Edge,
    );
    /// Removes a single directed arc `from -> to`; the reverse direction, if present, is left untouched.
    fn remove_edge_directed(
        &mut self,
        from: VertexId<Self::Id>,
        to: VertexId<Self::Id>,
    ) -> Option<Self::Edge>;
}

/// Dense index of an edge, `0..edge_count`, for callers that keep edge data in their own
/// arrays. Stored as `id + 1` in a `NonZero`, so `Option<EdgeId>` costs no extra memory.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct EdgeId<I: UnsignedId>(I::NonZero);

impl<I: UnsignedId> EdgeId<I> {
    pub fn new(id: usize) -> Self {
        Self(I::to_nz(id))
    }

    pub fn id(&self) -> usize {
        I::from_nz(self.0)
    }
}

// Not derived, for the same reason as `VertexId`'s: ordered by the dense index `id()` returns.
impl<I: UnsignedId> PartialOrd for EdgeId<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: UnsignedId> Ord for EdgeId<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct Edge<E, I: UnsignedId> {
    pub from: VertexId<I>,
    pub to: VertexId<I>,
    pub data: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct EdgeSimple<E, I: UnsignedId> {
    to: VertexId<I>,
    data: E,
}

// Not `#[serde_feature]`: `data` borrows (`&'a E`), and a borrowed view can't implement
// `Deserialize` (nothing to borrow from during deserialization). `Edge<E, I>` is the
// serializable, owned counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeView<'a, E, I: UnsignedId, K = ()> {
    pub from: VertexId<I>,
    pub to: VertexId<I>,
    pub data: &'a E,
    /// The storage's [`EdgeKey`](GraphEdgeStorage::EdgeKey) of this edge.
    pub key: K,
}
impl<'a, E, I: UnsignedId, K> EdgeView<'a, E, I, K> {
    pub fn new(from: VertexId<I>, to: VertexId<I>, data: &'a E, key: K) -> Self {
        Self {
            from,
            to,
            data,
            key,
        }
    }
}
