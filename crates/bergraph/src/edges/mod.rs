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

/// Edge data usable as an edge weight by the weighted visits.
pub trait Weighted {
    /// Weight of the edge.
    fn weight(&self) -> f64;
}

/// Read side of every edge storage, independent of whether it is also capable of directed
/// navigation ([`GraphEdgeStorageDirected`]) or of writes ([`GraphEdgeStorageMut`]).
#[cfg_attr(feature = "serde", serde_feature)]
pub trait GraphEdgeStorage {
    /// Data stored on each edge.
    type Edge: 'static;
    /// Vertex id type.
    type Id: UnsignedId;
    /// Identifies an edge in the [`EdgeView`]s: an [`EdgeId`] on storages with dense edge ids, `()` otherwise.
    type EdgeKey: Copy;

    /// Every edge of the storage.
    fn all(&self) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    /// Number of edges.
    fn count(&self) -> usize;

    /// Edges leaving `id`: on an undirected storage, every edge touching it.
    fn of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    /// Number of edges [`of`](Self::of) `id`.
    fn count_of(&self, id: VertexId<Self::Id>) -> usize;

    /// Data of the edge `from -> to`, if any.
    fn get(&self, from: &VertexId<Self::Id>, to: &VertexId<Self::Id>) -> Option<&Self::Edge>;
    /// Whether the edge `from -> to` exists.
    fn has_edge(&self, from: &VertexId<Self::Id>, to: &VertexId<Self::Id>) -> bool;
}

/// Write side of an edge storage: `add_edge`/`remove_edge` act on both endpoints, while the bulk
/// `with_edges`/`add_all`/`remove_all` store one arc per tuple, exactly as given.
pub trait GraphEdgeStorageMut: GraphEdgeStorage {
    /// Empty storage with room for `capacity` vertices.
    fn with_capacity(capacity: usize) -> Self;
    /// Storage holding exactly `edges`, one arc per entry.
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

    /// Adds one arc leaving `from` per `(to, data)` entry.
    fn add_all(
        &mut self,
        from: VertexId<Self::Id>,
        edges: impl IntoIterator<Item = (VertexId<Self::Id>, Self::Edge)>,
    );
    /// Removes every arc leaving `from`, returning their `(to, data)`.
    fn remove_all(&mut self, from: VertexId<Self::Id>) -> Vec<(VertexId<Self::Id>, Self::Edge)>;
}

/// Extends [`GraphEdgeStorage`] with direction-aware navigation.
pub trait GraphEdgeStorageDirected: GraphEdgeStorage {
    /// Edges leaving `id`.
    fn children_of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;
    /// Edges reaching `id`.
    fn parents_of(
        &self,
        id: VertexId<Self::Id>,
    ) -> impl Iterator<Item = EdgeView<'_, Self::Edge, Self::Id, Self::EdgeKey>>;

    /// Number of edges reaching `id`.
    fn count_incoming(&self, id: VertexId<Self::Id>) -> usize;
    /// Number of edges leaving `id`.
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
    /// Edge id of the dense index `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id + 1` does not fit the id type `I`.
    pub fn new(id: usize) -> Self {
        Self(I::to_nz(id))
    }

    /// The dense index of this edge.
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
/// Owned edge, as taken by [`with_edges`](GraphEdgeStorageMut::with_edges).
pub struct Edge<E, I: UnsignedId> {
    /// Source vertex.
    pub from: VertexId<I>,
    /// Target vertex.
    pub to: VertexId<I>,
    /// Edge data.
    pub data: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde_feature)]
/// Stored half of an edge: its target and data, the source being the vertex it is stored under.
pub struct EdgeSimple<E, I: UnsignedId> {
    to: VertexId<I>,
    data: E,
}

// Not `#[serde_feature]`: `data` borrows (`&'a E`), and a borrowed view can't implement
// `Deserialize` (nothing to borrow from during deserialization). `Edge<E, I>` is the
// serializable, owned counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Borrowed view of a stored edge, as yielded by the storages.
pub struct EdgeView<'a, E, I: UnsignedId, K = ()> {
    /// Source vertex.
    pub from: VertexId<I>,
    /// Target vertex.
    pub to: VertexId<I>,
    /// Edge data.
    pub data: &'a E,
    /// The storage's [`EdgeKey`](GraphEdgeStorage::EdgeKey) of this edge.
    pub key: K,
}
impl<'a, E, I: UnsignedId, K> EdgeView<'a, E, I, K> {
    /// View of the edge `from -> to` with its `data` and `key`.
    pub fn new(from: VertexId<I>, to: VertexId<I>, data: &'a E, key: K) -> Self {
        Self {
            from,
            to,
            data,
            key,
        }
    }
}
