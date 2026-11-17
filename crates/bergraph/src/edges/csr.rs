#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::{
    edges::{EdgeId, EdgeView, GraphEdgeStorage, GraphEdgeStorageDirected, MaybeSerde},
    vertices::{UnsignedId, VertexId},
};

/// Static Compressed Sparse Row storage, built once from a full edge list. An edge's position in
/// the packed arrays is its [`EdgeId`]; its data `E` is stored there too, free when `E = ()`.
// shortcut: deserialization does not validate the arrays, so inconsistent input panics on use; add a validating `try_from` if graphs come from untrusted sources.
#[derive(Debug)]
#[cfg_attr(feature = "serde", serde_feature)]
pub struct AdjCsr<E = (), VI: UnsignedId = u32, EI: UnsignedId = u32> {
    offsets: Vec<EI>,
    targets: Vec<VertexId<VI>>,
    data: Vec<E>,
    twins: Vec<EdgeId<EI>>,
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

impl<E, VI: UnsignedId, EI: UnsignedId> AdjCsr<E, VI, EI> {
    /// Directed graph with one edge per `(from, to, data)`; returns the id of each input edge,
    /// in order.
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
        let csr = Self {
            offsets,
            targets,
            data,
            twins,
        };
        Ok((csr, ids))
    }

    fn range_of(&self, id: VertexId<VI>) -> std::ops::Range<usize> {
        match (self.offsets.get(id.id()), self.offsets.get(id.id() + 1)) {
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

    fn parents_of(
        &self,
        id: VertexId<VI>,
    ) -> impl Iterator<Item = EdgeView<'_, E, VI, EdgeId<EI>>> {
        self.all().filter(move |edge| edge.to == id)
    }

    fn count_incoming(&self, id: VertexId<VI>) -> usize {
        self.parents_of(id).count()
    }

    fn count_outgoing(&self, id: VertexId<VI>) -> usize {
        self.count_of(id)
    }
}
