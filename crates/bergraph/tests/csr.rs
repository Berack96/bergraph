use bergraph::{
    EdgeId, Graph, VertexId,
    graph_edges::{AdjCsr, CsrError, GraphEdgeStorage, GraphEdgeStorageDirected},
    graph_view::GraphView,
    graph_visit::{Bfs, Dijkstra, ViewVisit},
};

fn v(id: usize) -> VertexId<u32> {
    VertexId::new(id)
}

fn neighbors<S: GraphEdgeStorage<Edge = (), EdgeKey = EdgeId<u32>>>(
    csr: &S,
    vertex: usize,
) -> Vec<(usize, usize)> {
    csr.of(VertexId::new(vertex))
        .map(|edge| (edge.to.id(), edge.key.id()))
        .collect()
}

#[test]
fn builds_a_directed_graph_with_edge_ids_grouped_by_source() {
    let (csr, ids) =
        AdjCsr::<u32, u32>::directed(3, &[(v(1), v(2)), (v(0), v(1)), (v(1), v(0))]).unwrap();

    assert_eq!(csr.count(), 3);
    assert_eq!(ids, vec![EdgeId::new(1), EdgeId::new(0), EdgeId::new(2)]);
    assert_eq!(neighbors(&csr, 0), vec![(1, 0)]);
    assert_eq!(neighbors(&csr, 1), vec![(2, 1), (0, 2)]);
    assert_eq!(neighbors(&csr, 2), vec![]);
    assert!(csr.has_edge(&v(1), &v(0)));
    assert_eq!(csr.get(&v(2), &v(0)), None);
    assert_eq!(csr.twin(EdgeId::new(0)), None);
}

#[test]
fn navigates_parents_and_children() {
    let (csr, _) = AdjCsr::<u32, u32>::directed(3, &[(v(0), v(2)), (v(1), v(2))]).unwrap();

    let parents: Vec<_> = csr.parents_of(v(2)).map(|edge| edge.from.id()).collect();
    assert_eq!(parents, vec![0, 1]);
    assert_eq!(csr.count_incoming(v(2)), 2);
    assert_eq!(csr.count_outgoing(v(0)), 1);
    assert_eq!(csr.children_of(v(2)).count(), 0);
}

#[test]
fn builds_an_undirected_graph_with_twin_half_edges() {
    let (csr, ids) =
        AdjCsr::<u32, u32>::undirected(3, &[(v(0), v(1)), (v(1), v(2)), (v(2), v(2))]).unwrap();

    assert_eq!(csr.count(), 5);
    for (input, (from, to)) in ids.iter().zip([(0, 1), (1, 2), (2, 2)]) {
        assert!(neighbors(&csr, from).contains(&(to, input.id())));
        let twin = csr.twin(*input).unwrap();
        assert!(neighbors(&csr, to).contains(&(from, twin.id())));
        assert_eq!(csr.twin(twin), Some(*input));
    }
    // The self-loop is a single half-edge, its own twin.
    assert_eq!(csr.twin(ids[2]), Some(ids[2]));
}

#[test]
fn uses_separate_vertex_and_edge_index_types() {
    let (csr, ids) =
        AdjCsr::<u16, u32>::directed(2, &[(VertexId::new(0), VertexId::new(1))]).unwrap();

    let edge = csr.of(VertexId::new(0)).next().unwrap();
    assert_eq!((edge.to.id(), edge.key), (1, ids[0]));
}

#[test]
fn has_no_edges_on_a_vertex_outside_the_graph() {
    let (csr, _) = AdjCsr::<u32, u32>::directed(1, &[(v(0), v(0))]).unwrap();

    assert_eq!(csr.of(v(1)).count(), 0);
    assert_eq!(csr.count_of(v(1)), 0);
    assert!(!csr.has_edge(&v(1), &v(0)));
}

#[test]
fn lists_all_edges_in_edge_id_order() {
    let (csr, _) =
        AdjCsr::<u32, u32>::undirected(3, &[(v(2), v(0)), (v(1), v(0)), (v(1), v(1))]).unwrap();

    let keys: Vec<_> = csr.all().map(|edge| edge.key.id()).collect();
    assert_eq!(keys, (0..csr.count()).collect::<Vec<_>>());
    let parents: Vec<_> = csr.parents_of(v(0)).map(|edge| edge.from.id()).collect();
    assert_eq!(parents, vec![1, 2]);
}

#[test]
fn builds_an_empty_graph() {
    let (csr, ids) = AdjCsr::<u32, u32>::directed(0, &[]).unwrap();

    assert_eq!(csr.count(), 0);
    assert_eq!(csr.all().count(), 0);
    assert!(ids.is_empty());
}

#[test]
fn runs_the_visits_on_a_graph_over_a_csr() {
    let (csr, _) = AdjCsr::<u32, u32>::undirected(3, &[(v(0), v(1)), (v(1), v(2))]).unwrap();
    let graph = Graph::new(vec!["a", "b", "c"], csr);

    assert_eq!(graph.len(), 3);
    assert_eq!(Bfs.visit(&graph, v(2)).unwrap(), vec![v(2), v(1), v(0)]);
    let view = graph.filtered_view(|name: &&str| *name != "b", |_| true);
    assert_eq!(Bfs.visit(&view, v(2)).unwrap(), vec![v(2)]);
}

#[test]
fn runs_dijkstra_with_costs_indexed_by_edge_id() {
    let (csr, ids) =
        AdjCsr::<u32, u32>::directed(3, &[(v(0), v(1)), (v(1), v(2)), (v(0), v(2))]).unwrap();
    let graph = Graph::new(vec!["a", "b", "c"], csr);
    let mut costs = vec![0.0; graph.edges().count()];
    for (id, cost) in ids.iter().zip([1.0, 1.0, 5.0]) {
        costs[id.id()] = cost;
    }

    let shortest = Dijkstra::new(v(2)).visit(&graph, v(0), |edge| costs[edge.key.id()]);
    assert_eq!(shortest.unwrap(), Some((vec![v(0), v(1), v(2)], 2.0)));
}

#[test]
fn filters_csr_edges_by_edge_id() {
    let (csr, ids) = AdjCsr::<u32, u32>::undirected(3, &[(v(0), v(1)), (v(1), v(2))]).unwrap();
    let graph = Graph::new(vec!["a", "b", "c"], csr);
    let closed = graph.edges().twin(ids[1]).unwrap();

    let view = graph.filtered_view(|_| true, |edge| edge.key != ids[1] && edge.key != closed);
    assert_eq!(Bfs.visit(&view, v(0)).unwrap(), vec![v(0), v(1)]);
    assert!(!view.edges().has_edge(&v(1), &v(2)));
}

#[test]
fn builds_edges_on_the_highest_vertex_id() {
    let (csr, _) =
        AdjCsr::<u8, u32>::directed(255, &[(VertexId::new(254), VertexId::new(0))]).unwrap();

    let edge = csr.of(VertexId::new(254)).next().unwrap();
    assert_eq!(edge.to.id(), 0);
}

#[test]
fn rejects_a_vertex_count_above_the_vertex_id_range() {
    assert_eq!(AdjCsr::<u8, u32>::directed(255, &[]).err(), None);
    assert_eq!(
        AdjCsr::<u8, u32>::directed(256, &[]).err(),
        Some(CsrError::TooManyVertices)
    );
}

#[test]
fn counts_an_undirected_self_loop_as_one_half_edge() {
    let edges = vec![(v(0), v(0)); 255];

    assert_eq!(AdjCsr::<u32, u8>::undirected(1, &edges).err(), None);
}

#[test]
fn rejects_an_edge_count_above_the_edge_index_range() {
    let edges = vec![(v(0), v(1)); 255];
    assert_eq!(AdjCsr::<u32, u8>::directed(2, &edges).err(), None);

    let edges = vec![(v(0), v(1)); 128];
    assert_eq!(
        AdjCsr::<u32, u8>::undirected(2, &edges).err(),
        Some(CsrError::TooManyEdges)
    );
}

#[test]
fn rejects_an_endpoint_outside_the_vertices() {
    assert_eq!(
        AdjCsr::<u32, u32>::directed(2, &[(v(0), v(2))]).err(),
        Some(CsrError::VertexOutOfRange)
    );
}
