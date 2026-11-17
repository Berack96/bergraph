use std::fmt;

#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::{
    edges::{EdgeId, EdgeView, GraphEdgeStorage, GraphEdgeStorageDirected, MaybeSerde},
    vertices::{UnsignedId, VertexId},
};

/// Static Compressed Sparse Row storage, built once from a full edge list. An edge's position in
/// the packed arrays is its [`EdgeId`]; its data `E` is stored there too, free when `E = ()`.
// shortcut: deserialization does not validate the arrays, so inconsistent input panics on use; add a validating `try_from` if graphs come from untrusted sources.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct AdjCsr<E = (), VI: UnsignedId = u32, EI: UnsignedId = u32> {
    offsets: Vec<EI>,
    targets: Vec<VertexId<VI>>,
    data: Vec<E>,
    twins: Vec<EdgeId<EI>>,
    // Directed only: the incoming edges of each vertex, in edge id order, indexed like `offsets`.
    in_offsets: Vec<EI>,
    in_edges: Vec<EdgeId<EI>>,
}

/// Why an edge list cannot become an [`AdjCsr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrError {
    /// The vertex count does not fit the vertex id type.
    TooManyVertices,
    /// The edge count does not fit the edge index type.
    TooManyEdges,
    /// An edge endpoint is not below the vertex count.
    VertexOutOfRange,
}

impl fmt::Display for CsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooManyVertices => "vertex count does not fit the vertex id type",
            Self::TooManyEdges => "edge count does not fit the edge index type",
            Self::VertexOutOfRange => "edge endpoint is not below the vertex count",
        })
    }
}

impl std::error::Error for CsrError {}

impl<E, VI: UnsignedId, EI: UnsignedId> AdjCsr<E, VI, EI> {
    /// Directed graph with one edge per `(from, to, data)`; returns the id of each input edge,
    /// in order.
    ///
    /// # Errors
    ///
    /// [`CsrError::TooManyVertices`] or [`CsrError::TooManyEdges`] if a count does not fit its
    /// index type, [`CsrError::VertexOutOfRange`] if an endpoint is not below `vertex_count`.
    pub fn directed(
        vertex_count: usize,
        edges: &[(VertexId<VI>, VertexId<VI>, E)],
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError>
    where
        E: Clone,
    {
        Self::build(vertex_count, edges, false)
    }

    /// Undirected graph: each edge becomes two twin half-edges with a copy of its data each, one
    /// for a self-loop; returns the id of each input `from -> to` half-edge, in order.
    ///
    /// # Errors
    ///
    /// [`CsrError::TooManyVertices`] or [`CsrError::TooManyEdges`] if a count does not fit its
    /// index type, [`CsrError::VertexOutOfRange`] if an endpoint is not below `vertex_count`.
    pub fn undirected(
        vertex_count: usize,
        edges: &[(VertexId<VI>, VertexId<VI>, E)],
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError>
    where
        E: Clone,
    {
        Self::build(vertex_count, edges, true)
    }

    fn vertex_count(&self) -> usize {
        self.offsets.len() - 1
    }

    /// The reverse half-edge of `edge` in an undirected graph, `None` in a directed one.
    pub fn twin(&self, edge: EdgeId<EI>) -> Option<EdgeId<EI>> {
        self.twins.get(edge.id()).copied()
    }

    fn build(
        vertex_count: usize,
        edges: &[(VertexId<VI>, VertexId<VI>, E)],
        undirected: bool,
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError>
    where
        E: Clone,
    {
        if VI::try_from_usize(vertex_count).is_none() || vertex_count == usize::MAX {
            return Err(CsrError::TooManyVertices);
        }
        if edges
            .iter()
            .any(|(from, to, _)| from.id() >= vertex_count || to.id() >= vertex_count)
        {
            return Err(CsrError::VertexOutOfRange);
        }

        let mut offsets = vec![0usize; vertex_count + 1];
        for (from, to, _) in edges {
            offsets[from.id() + 1] += 1;
            if undirected && from != to {
                offsets[to.id() + 1] += 1;
            }
        }
        for i in 1..offsets.len() {
            offsets[i] += offsets[i - 1];
        }
        let edge_count = offsets[vertex_count];
        EI::try_from_usize(edge_count).ok_or(CsrError::TooManyEdges)?;

        let mut cursor = offsets.clone();
        let mut targets = vec![VertexId::new(0); edge_count];
        // Input index of each half-edge, to copy its data into place once every edge is placed.
        let mut inputs = vec![0; edge_count];
        let mut place = |from: VertexId<VI>, to: VertexId<VI>, input: usize| {
            let edge = cursor[from.id()];
            cursor[from.id()] += 1;
            targets[edge] = to;
            inputs[edge] = input;
            edge
        };
        let mut twins = if undirected {
            vec![EdgeId::new(0); edge_count]
        } else {
            Vec::new()
        };
        let mut ids = Vec::with_capacity(edges.len());
        for (input, &(from, to, _)) in edges.iter().enumerate() {
            let forward = place(from, to, input);
            if undirected {
                let backward = if from == to {
                    forward
                } else {
                    place(to, from, input)
                };
                twins[forward] = EdgeId::new(backward);
                twins[backward] = EdgeId::new(forward);
            }
            ids.push(EdgeId::new(forward));
        }

        let offsets = offsets
            .into_iter()
            .map(|offset| EI::try_from_usize(offset).expect("offsets never exceed the edge count"))
            .collect();
        let data = inputs.iter().map(|&input| edges[input].2.clone()).collect();
        let (in_offsets, in_edges) = if undirected {
            (Vec::new(), Vec::new())
        } else {
            Self::reverse_index(vertex_count, &targets)
        };
        let csr = Self {
            offsets,
            targets,
            data,
            twins,
            in_offsets,
            in_edges,
        };
        Ok((csr, ids))
    }

    /// Counting sort of the edge ids by target: the incoming edges of each vertex, in id order.
    fn reverse_index(vertex_count: usize, targets: &[VertexId<VI>]) -> (Vec<EI>, Vec<EdgeId<EI>>) {
        let mut offsets = vec![0usize; vertex_count + 1];
        for to in targets {
            offsets[to.id() + 1] += 1;
        }
        for i in 1..offsets.len() {
            offsets[i] += offsets[i - 1];
        }
        let mut cursor = offsets.clone();
        let mut edges = vec![EdgeId::new(0); targets.len()];
        for (edge, to) in targets.iter().enumerate() {
            edges[cursor[to.id()]] = EdgeId::new(edge);
            cursor[to.id()] += 1;
        }
        let offsets = offsets
            .into_iter()
            .map(|offset| EI::try_from_usize(offset).expect("offsets never exceed the edge count"))
            .collect();
        (offsets, edges)
    }

    /// The vertex whose outgoing range holds `edge`.
    fn source_of(&self, edge: usize) -> VertexId<VI> {
        VertexId::new(
            self.offsets
                .partition_point(|offset| offset.to_usize() <= edge)
                - 1,
        )
    }

    fn range_of(&self, id: VertexId<VI>) -> std::ops::Range<usize> {
        Self::range_in(&self.offsets, id)
    }

    fn range_in(offsets: &[EI], id: VertexId<VI>) -> std::ops::Range<usize> {
        match (offsets.get(id.id()), offsets.get(id.id() + 1)) {
            (Some(start), Some(end)) => start.to_usize()..end.to_usize(),
            _ => 0..0,
        }
    }
}

impl<E: 'static + MaybeSerde, VI: UnsignedId + MaybeSerde, EI: UnsignedId + MaybeSerde>
    GraphEdgeStorage for AdjCsr<E, VI, EI>
{
    type Edge = E;
    type Id = VI;
    type EdgeKey = EdgeId<EI>;

    fn all(&self) -> impl Iterator<Item = EdgeView<'_, E, VI, EdgeId<EI>>> {
        (0..self.vertex_count()).flat_map(move |from| self.of(VertexId::new(from)))
    }

    fn count(&self) -> usize {
        self.targets.len()
    }

    fn of(&self, id: VertexId<VI>) -> impl Iterator<Item = EdgeView<'_, E, VI, EdgeId<EI>>> {
        self.range_of(id).map(move |edge| {
            EdgeView::new(id, self.targets[edge], &self.data[edge], EdgeId::new(edge))
        })
    }

    fn count_of(&self, id: VertexId<VI>) -> usize {
        self.range_of(id).len()
    }

    fn get(&self, from: &VertexId<VI>, to: &VertexId<VI>) -> Option<&E> {
        self.range_of(*from)
            .find(|&edge| self.targets[edge] == *to)
            .map(|edge| &self.data[edge])
    }

    fn has_edge(&self, from: &VertexId<VI>, to: &VertexId<VI>) -> bool {
        self.range_of(*from).any(|edge| self.targets[edge] == *to)
    }
}

impl<E: 'static + MaybeSerde, VI: UnsignedId + MaybeSerde, EI: UnsignedId + MaybeSerde>
    GraphEdgeStorageDirected for AdjCsr<E, VI, EI>
{
    fn children_of(
        &self,
        id: VertexId<VI>,
    ) -> impl Iterator<Item = EdgeView<'_, E, VI, EdgeId<EI>>> {
        self.of(id)
    }

    /// The incoming edges of `id`: in edge id order on a directed graph, in the order of
    /// [`of`](GraphEdgeStorage::of) on an undirected one, where they are the twins of its outgoing edges.
    fn parents_of(
        &self,
        id: VertexId<VI>,
    ) -> impl Iterator<Item = EdgeView<'_, E, VI, EdgeId<EI>>> {
        // Exactly one of the two is non-empty: the reverse index is built only without twins.
        let directed = self.in_edges[Self::range_in(&self.in_offsets, id)]
            .iter()
            .map(|edge| (edge.id(), self.source_of(edge.id())));
        let twins = if self.twins.is_empty() {
            0..0
        } else {
            self.range_of(id)
        }
        .map(|edge| (self.twins[edge].id(), self.targets[edge]));
        directed
            .chain(twins)
            .map(move |(edge, from)| EdgeView::new(from, id, &self.data[edge], EdgeId::new(edge)))
    }

    fn count_incoming(&self, id: VertexId<VI>) -> usize {
        if self.twins.is_empty() {
            Self::range_in(&self.in_offsets, id).len()
        } else {
            self.count_of(id)
        }
    }

    fn count_outgoing(&self, id: VertexId<VI>) -> usize {
        self.count_of(id)
    }
}
