#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

use crate::{
    edges::{EdgeId, EdgeView, GraphEdgeStorage, GraphEdgeStorageDirected, MaybeSerde},
    vertices::{UnsignedId, VertexId},
};

/// Static Compressed Sparse Row storage, built once from a full edge list. No edge data: an
/// edge's position in the packed array is its [`EdgeId`], so callers keep their data indexed by id.
// shortcut: deserialization does not validate the arrays, so inconsistent input panics on use; add a validating `try_from` if graphs come from untrusted sources.
#[cfg_attr(feature = "serde", serde_feature)]
pub struct AdjCsr<VI: UnsignedId = u32, EI: UnsignedId = u32> {
    offsets: Vec<EI>,
    targets: Vec<VertexId<VI>>,
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

impl<VI: UnsignedId, EI: UnsignedId> AdjCsr<VI, EI> {
    /// Directed graph with one edge per `(from, to)` pair; returns the id of each input edge, in order.
    pub fn directed(
        vertex_count: usize,
        edges: &[(VertexId<VI>, VertexId<VI>)],
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError> {
        Self::build(vertex_count, edges, false)
    }

    /// Undirected graph: each pair becomes two twin half-edges, one for a self-loop; returns the
    /// id of each input `from -> to` half-edge, in order.
    pub fn undirected(
        vertex_count: usize,
        edges: &[(VertexId<VI>, VertexId<VI>)],
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError> {
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
        edges: &[(VertexId<VI>, VertexId<VI>)],
        undirected: bool,
    ) -> Result<(Self, Vec<EdgeId<EI>>), CsrError> {
        if VI::try_from_usize(vertex_count).is_none() || vertex_count == usize::MAX {
            return Err(CsrError::TooManyVertices);
        }
        if edges
            .iter()
            .any(|(from, to)| from.id() >= vertex_count || to.id() >= vertex_count)
        {
            return Err(CsrError::VertexOutOfRange);
        }

        let mut offsets = vec![0usize; vertex_count + 1];
        for (from, to) in edges {
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
        let mut place = |from: VertexId<VI>, to: VertexId<VI>| {
            let edge = cursor[from.id()];
            cursor[from.id()] += 1;
            targets[edge] = to;
            edge
        };
        let mut twins = if undirected {
            vec![EdgeId::new(0); edge_count]
        } else {
            Vec::new()
        };
        let mut ids = Vec::with_capacity(edges.len());
        for &(from, to) in edges {
            let forward = place(from, to);
            if undirected {
                let backward = if from == to { forward } else { place(to, from) };
                twins[forward] = EdgeId::new(backward);
                twins[backward] = EdgeId::new(forward);
            }
            ids.push(EdgeId::new(forward));
        }

        let offsets = offsets
            .into_iter()
            .map(|offset| EI::try_from_usize(offset).expect("offsets never exceed the edge count"))
            .collect();
        let csr = Self {
            offsets,
            targets,
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

impl<VI: UnsignedId + MaybeSerde, EI: UnsignedId + MaybeSerde> GraphEdgeStorage for AdjCsr<VI, EI> {
    type Edge = ();
    type Id = VI;
    type EdgeKey = EdgeId<EI>;

    fn all(&self) -> impl Iterator<Item = EdgeView<'_, (), VI, EdgeId<EI>>> {
        (0..self.vertex_count()).flat_map(move |from| self.of(VertexId::new(from)))
    }

    fn count(&self) -> usize {
        self.targets.len()
    }

    fn of(&self, id: VertexId<VI>) -> impl Iterator<Item = EdgeView<'_, (), VI, EdgeId<EI>>> {
        self.range_of(id)
            .map(move |edge| EdgeView::new(id, self.targets[edge], &(), EdgeId::new(edge)))
    }

    fn count_of(&self, id: VertexId<VI>) -> usize {
        self.range_of(id).len()
    }

    fn get(&self, from: &VertexId<VI>, to: &VertexId<VI>) -> Option<&()> {
        self.has_edge(from, to).then_some(&())
    }

    fn has_edge(&self, from: &VertexId<VI>, to: &VertexId<VI>) -> bool {
        self.range_of(*from).any(|edge| self.targets[edge] == *to)
    }
}

impl<VI: UnsignedId + MaybeSerde, EI: UnsignedId + MaybeSerde> GraphEdgeStorageDirected
    for AdjCsr<VI, EI>
{
    fn children_of(
        &self,
        id: VertexId<VI>,
    ) -> impl Iterator<Item = EdgeView<'_, (), VI, EdgeId<EI>>> {
        self.of(id)
    }

    fn parents_of(
        &self,
        id: VertexId<VI>,
    ) -> impl Iterator<Item = EdgeView<'_, (), VI, EdgeId<EI>>> {
        self.all().filter(move |edge| edge.to == id)
    }

    fn count_incoming(&self, id: VertexId<VI>) -> usize {
        self.parents_of(id).count()
    }

    fn count_outgoing(&self, id: VertexId<VI>) -> usize {
        self.count_of(id)
    }
}
