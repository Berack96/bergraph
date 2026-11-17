use bergraph::GraphBuilder;
use bergraph::graph_edges::{AdjList, GraphEdgeStorageDirectedMut, GraphEdgeStorageMut, Weighted};
use bergraph::graph_view::GraphView;
use bergraph::graph_visit::{
    AStar, AllSimplePaths, ArticulationPoints, BellmanFord, BetweennessCentrality, Bfs, BfsToAny,
    BiconnectedComponents, Bipartite, Bridges, ConnectedComponents, CycleDetection, Dfs, Dijkstra,
    DijkstraReached, Eccentricity, EulerianTrail, Flooded, MaxFlow, MinimumSpanningTree,
    MultiSourceBfs, MultiSourceDijkstra, PriorityFlood, Reached, SearchBuffers,
    StronglyConnectedComponents, TopologicalSort, TransitiveClosure, VisitError,
};

#[test]
fn bfs_visits_reachable_vertices_in_order() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());

    let order = Bfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a, b, c]);
}

#[test]
fn bfs_reports_missing_start_vertex() {
    let graph = GraphBuilder::<&str, ()>::new().build();
    let missing = bergraph::VertexId::new(0);

    assert_eq!(Bfs.visit(&graph, missing), Err(VisitError::VertexNotFound));
}

#[test]
fn bfs_ignores_disconnected_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let _c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());

    let order = Bfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a, b]);
}

#[test]
fn bfs_handles_self_loop() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    graph.edges_mut().add_edge_directed(a, a, ());

    let order = Bfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a]);
}

#[test]
fn dfs_visits_reachable_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, c, ());

    // Iterative DFS pops the stack LIFO, so children are visited in reverse
    // of the order children_of() yields them: c (pushed last) before b.
    let order = Dfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a, c, b]);
}

#[test]
fn dfs_reports_missing_start_vertex() {
    let graph = GraphBuilder::<&str, ()>::new().build();
    let missing = bergraph::VertexId::new(0);

    assert_eq!(Dfs.visit(&graph, missing), Err(VisitError::VertexNotFound));
}

#[test]
fn dfs_ignores_disconnected_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let _c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());

    let order = Dfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a, b]);
}

#[test]
fn dfs_handles_self_loop() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    graph.edges_mut().add_edge_directed(a, a, ());

    let order = Dfs.visit(&graph, a).unwrap();
    assert_eq!(order, vec![a]);
}

#[test]
fn bfs_reports_missing_start_vertex_when_excluded_by_a_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let view = graph.filtered_view(|_, v: &&str| *v != "a", |_| true);

    assert_eq!(Bfs.visit(&view, a), Err(VisitError::VertexNotFound));
}

#[test]
fn bfs_does_not_reach_a_vertex_excluded_by_the_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let x = graph.add_vertex("x");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, x, ());
    let view = graph.filtered_view(|_, v: &&str| *v != "x", |_| true);

    let order = Bfs.visit(&view, a).unwrap();

    assert_eq!(order, vec![a, b]);
}

#[test]
fn bfs_to_any_finds_the_path_to_the_closest_target() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, d, ());
    graph.edges_mut().add_edge_directed(b, c, ());

    let (path, target) = BfsToAny::new([c, d]).visit(&graph, a).unwrap().unwrap();

    assert_eq!(path, vec![a, d]);
    assert_eq!(target, d);
}

#[test]
fn bfs_to_any_breaks_a_tie_between_equidistant_targets_by_discovery_order() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, c, ());

    let (path, target) = BfsToAny::new([c, b]).visit(&graph, a).unwrap().unwrap();

    assert_eq!(path, vec![a, b]);
    assert_eq!(target, b);
}

#[test]
fn bfs_to_any_returns_the_start_immediately_when_it_is_a_target() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, b, ());

    let (path, target) = BfsToAny::new([a, b]).visit(&graph, a).unwrap().unwrap();

    assert_eq!(path, vec![a]);
    assert_eq!(target, a);
}

#[test]
fn bfs_to_any_returns_none_when_no_target_is_reachable() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");

    assert_eq!(BfsToAny::new([b]).visit(&graph, a).unwrap(), None);
}

#[test]
fn bfs_to_any_reports_missing_start_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let b = graph.add_vertex("b");
    let missing = bergraph::VertexId::new(1);

    assert_eq!(
        BfsToAny::new([b]).visit(&graph, missing),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn bfs_to_any_reports_missing_target_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let missing = bergraph::VertexId::new(1);

    assert_eq!(
        BfsToAny::new([missing]).visit(&graph, a),
        Err(VisitError::VertexNotFound)
    );
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Km(f64);
impl Weighted for Km {
    fn weight(&self) -> f64 {
        self.0
    }
}

#[test]
fn dijkstra_finds_shortest_path() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, Km(5.0));
    graph.edges_mut().add_edge_directed(a, c, Km(1.0));
    graph.edges_mut().add_edge_directed(c, b, Km(1.0));

    let (path, total) = Dijkstra::new(b)
        .visit(&graph, a, |edge| edge.data.0)
        .unwrap()
        .unwrap();
    assert_eq!(path, vec![a, c, b]);
    assert_eq!(total, 2.0);
}

#[test]
fn dijkstra_breaks_distance_ties_on_the_lower_vertex_id() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, c, Km(1.0));
    graph.edges_mut().add_edge_directed(a, b, Km(1.0));
    graph.edges_mut().add_edge_directed(c, d, Km(1.0));
    graph.edges_mut().add_edge_directed(b, d, Km(1.0));

    let (path, _) = Dijkstra::new(d)
        .visit(&graph, a, |edge| edge.data.0)
        .unwrap()
        .unwrap();
    assert_eq!(path, vec![a, b, d]);
}

#[test]
fn dijkstra_returns_none_for_unreachable_target() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");

    assert_eq!(
        Dijkstra::new(b)
            .visit(&graph, a, |edge| edge.data.0)
            .unwrap(),
        None
    );
}

#[test]
fn dijkstra_reports_missing_start_vertex() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let b = graph.add_vertex("b");
    let missing = bergraph::VertexId::new(1);

    assert_eq!(
        Dijkstra::new(b).visit(&graph, missing, |edge| edge.data.0),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn dijkstra_reports_missing_target_vertex() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let missing = bergraph::VertexId::new(1);

    assert_eq!(
        Dijkstra::new(missing).visit(&graph, a, |edge| edge.data.0),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn multi_source_bfs_attributes_each_vertex_to_the_nearest_source() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let s1 = graph.add_vertex("s1");
    let x = graph.add_vertex("x");
    let s2 = graph.add_vertex("s2");
    let far = graph.add_vertex("far");
    graph.edges_mut().add_edge(s1, x, ());
    graph.edges_mut().add_edge(s2, x, ());
    graph.edges_mut().add_edge(s2, far, ());

    let reached = MultiSourceBfs.visit(&graph, [s1, s2]).unwrap();
    let of = |v| reached.iter().find(|(id, _)| *id == v).map(|(_, r)| *r);

    assert_eq!(
        of(s1),
        Some(Reached {
            distance: 0,
            source: s1
        })
    );
    assert_eq!(
        of(s2),
        Some(Reached {
            distance: 0,
            source: s2
        })
    );
    // x is equidistant from s1 and s2: ties are broken by input order, s1 first.
    assert_eq!(
        of(x),
        Some(Reached {
            distance: 1,
            source: s1
        })
    );
    assert_eq!(
        of(far),
        Some(Reached {
            distance: 1,
            source: s2
        })
    );
}

#[test]
fn multi_source_bfs_omits_unreached_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let _isolated = graph.add_vertex("isolated");

    let reached = MultiSourceBfs.visit(&graph, [a]).unwrap();
    assert_eq!(
        reached,
        vec![(
            a,
            Reached {
                distance: 0,
                source: a
            }
        )]
    );
}

#[test]
fn multi_source_bfs_reports_missing_source_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let missing = bergraph::VertexId::new(5);

    assert_eq!(
        MultiSourceBfs.visit(&graph, [a, missing]),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn multi_source_dijkstra_attributes_each_vertex_to_the_cheapest_source() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let s1 = graph.add_vertex("s1");
    let s2 = graph.add_vertex("s2");
    let x = graph.add_vertex("x");
    let y = graph.add_vertex("y");
    graph.edges_mut().add_edge_directed(s1, x, Km(5.0));
    graph.edges_mut().add_edge_directed(s2, x, Km(1.0));
    graph.edges_mut().add_edge_directed(s1, y, Km(2.0));
    graph.edges_mut().add_edge_directed(s2, y, Km(2.0));

    let reached = MultiSourceDijkstra::new(f64::INFINITY)
        .visit(&graph, [s1, s2], |edge| edge.data.0)
        .unwrap();
    let of = |v| reached.iter().find(|(id, _)| *id == v).map(|(_, r)| *r);

    assert_eq!(
        of(s1),
        Some(DijkstraReached {
            distance: 0.0,
            source: s1
        })
    );
    assert_eq!(
        of(s2),
        Some(DijkstraReached {
            distance: 0.0,
            source: s2
        })
    );
    // s2 reaches x more cheaply (1.0 < 5.0), despite s1 being first in input order.
    assert_eq!(
        of(x),
        Some(DijkstraReached {
            distance: 1.0,
            source: s2
        })
    );
    // y is equidistant from both sources: the tie goes to s1, the lower id finalized first.
    assert_eq!(
        of(y),
        Some(DijkstraReached {
            distance: 2.0,
            source: s1
        })
    );
}

#[test]
fn multi_source_dijkstra_stops_beyond_the_maximum_distance() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, Km(1.0));
    graph.edges_mut().add_edge_directed(b, c, Km(1.0));

    let reached = MultiSourceDijkstra::new(1.0)
        .visit(&graph, [a], |edge| edge.data.0)
        .unwrap();
    let ids: Vec<_> = reached.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![a, b]);
    let sources_only = MultiSourceDijkstra::new(-1.0)
        .visit(&graph, [a], |edge| edge.data.0)
        .unwrap();
    assert_eq!(sources_only.len(), 1);
}

#[test]
fn multi_source_dijkstra_omits_unreached_vertices() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let _isolated = graph.add_vertex("isolated");

    let reached = MultiSourceDijkstra::new(f64::INFINITY)
        .visit(&graph, [a], |edge| edge.data.0)
        .unwrap();
    assert_eq!(
        reached,
        vec![(
            a,
            DijkstraReached {
                distance: 0.0,
                source: a
            }
        )]
    );
}

#[test]
fn multi_source_dijkstra_reports_missing_source_vertex() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let missing = bergraph::VertexId::new(5);

    assert_eq!(
        MultiSourceDijkstra::new(f64::INFINITY).visit(&graph, [a, missing], |edge| edge.data.0),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn connected_components_groups_vertices_reachable_from_one_another() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    let e = graph.add_vertex("e");
    let f = graph.add_vertex("f");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(d, e, ());

    let components = ConnectedComponents.visit(&graph);

    assert_eq!(components, vec![vec![a, b, c], vec![d, e], vec![f]]);
}

#[test]
fn connected_components_labels_each_vertex_with_the_first_of_its_component() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge(c, a, ());
    graph.edges_mut().add_edge(b, d, ());

    assert_eq!(
        ConnectedComponents.labels(&graph),
        vec![Some(a), Some(b), Some(a), Some(b)]
    );
    let view = graph.filtered_view(|_, v: &&str| *v != "b", |_| true);
    assert_eq!(
        ConnectedComponents.labels(&view),
        vec![Some(a), None, Some(a), Some(d)]
    );
    // A filtered-out last vertex still gets its slot.
    let view = graph.filtered_view(|_, v: &&str| *v != "d", |_| true);
    assert_eq!(
        ConnectedComponents.labels(&view),
        vec![Some(a), Some(b), Some(a), None]
    );
}

#[test]
fn connected_components_of_an_empty_graph_is_empty() {
    let graph = GraphBuilder::<&str, ()>::new().build();

    assert_eq!(ConnectedComponents.visit(&graph), Vec::<Vec<_>>::new());
}

#[test]
fn scc_groups_mutually_reachable_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    let e = graph.add_vertex("e");
    // a -> b -> c -> a is a cycle (one SCC); c -> d -> e is a one-way tail (two singletons).
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());
    graph.edges_mut().add_edge_directed(c, a, ());
    graph.edges_mut().add_edge_directed(c, d, ());
    graph.edges_mut().add_edge_directed(d, e, ());

    let mut components = StronglyConnectedComponents.visit(&graph);
    for component in &mut components {
        component.sort_by_key(|v| v.id());
    }
    components.sort_by_key(|component| component[0].id());

    assert_eq!(components, vec![vec![a, b, c], vec![d], vec![e]]);
}

#[test]
fn scc_of_an_empty_graph_is_empty() {
    let graph = GraphBuilder::<&str, ()>::new().build();

    assert_eq!(
        StronglyConnectedComponents.visit(&graph),
        Vec::<Vec<_>>::new()
    );
}

#[test]
fn scc_of_a_single_isolated_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");

    assert_eq!(StronglyConnectedComponents.visit(&graph), vec![vec![a]]);
}

#[test]
fn scc_groups_a_self_loop_into_its_own_component() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, a, ());
    graph.edges_mut().add_edge_directed(a, b, ());

    let mut components = StronglyConnectedComponents.visit(&graph);
    for component in &mut components {
        component.sort_by_key(|v| v.id());
    }
    components.sort_by_key(|component| component[0].id());

    assert_eq!(components, vec![vec![a], vec![b]]);
}

/// Two triangles {a, b, c} and {d, e, f} joined by a single edge c-d: that edge is the only
/// bridge, and its two endpoints are the only articulation points.
#[allow(clippy::type_complexity)]
fn two_triangles_joined_by_a_bridge() -> (
    bergraph::Graph<&'static str, AdjList<(), u32>>,
    [bergraph::VertexId<u32>; 6],
) {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    let e = graph.add_vertex("e");
    let f = graph.add_vertex("f");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());
    graph.edges_mut().add_edge(d, e, ());
    graph.edges_mut().add_edge(e, f, ());
    graph.edges_mut().add_edge(f, d, ());
    graph.edges_mut().add_edge(c, d, ());
    (graph, [a, b, c, d, e, f])
}

#[test]
fn bridges_finds_the_single_cut_edge_between_two_cycles() {
    let (graph, [_a, _b, c, d, _e, _f]) = two_triangles_joined_by_a_bridge();

    let bridges = Bridges.visit(&graph);

    assert_eq!(bridges.len(), 1);
    let (from, to) = bridges[0];
    assert!((from == c && to == d) || (from == d && to == c));
}

#[test]
fn bridges_finds_none_inside_a_single_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());

    assert_eq!(Bridges.visit(&graph), Vec::new());
}

#[test]
fn articulation_points_finds_the_bridge_endpoints() {
    let (graph, [_a, _b, c, d, _e, _f]) = two_triangles_joined_by_a_bridge();

    let mut points = ArticulationPoints.visit(&graph);
    points.sort_by_key(|v| v.id());
    let mut expected = [c, d];
    expected.sort_by_key(|v| v.id());

    assert_eq!(points, expected);
}

#[test]
fn articulation_points_finds_the_shared_vertex_of_a_bowtie() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let p1 = graph.add_vertex("p1");
    let p2 = graph.add_vertex("p2");
    let m = graph.add_vertex("m");
    let q1 = graph.add_vertex("q1");
    let q2 = graph.add_vertex("q2");
    graph.edges_mut().add_edge(p1, p2, ());
    graph.edges_mut().add_edge(p2, m, ());
    graph.edges_mut().add_edge(m, p1, ());
    graph.edges_mut().add_edge(q1, q2, ());
    graph.edges_mut().add_edge(q2, m, ());
    graph.edges_mut().add_edge(m, q1, ());

    // no single edge disconnects a bowtie, only its shared vertex does.
    assert_eq!(Bridges.visit(&graph), Vec::new());
    assert_eq!(ArticulationPoints.visit(&graph), vec![m]);
}

#[test]
fn bridges_and_articulation_points_ignore_a_self_loop() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());
    graph.edges_mut().add_edge(a, a, ());

    assert_eq!(Bridges.visit(&graph), Vec::new());
    assert_eq!(ArticulationPoints.visit(&graph), Vec::new());
}

#[test]
fn biconnected_components_splits_at_the_bridge() {
    let (graph, [_a, _b, c, d, _e, _f]) = two_triangles_joined_by_a_bridge();

    let mut components = BiconnectedComponents.visit(&graph);
    components.sort_by_key(|component| component.len());

    assert_eq!(components.len(), 3);
    assert_eq!(components[0].len(), 1);
    let (from, to) = components[0][0];
    assert!((from == c && to == d) || (from == d && to == c));
    assert_eq!(components[1].len(), 3);
    assert_eq!(components[2].len(), 3);
}

#[test]
fn biconnected_components_of_a_single_cycle_is_one_component() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());

    let components = BiconnectedComponents.visit(&graph);

    assert_eq!(components.len(), 1);
    assert_eq!(components[0].len(), 3);
}

#[test]
fn biconnected_components_reports_a_self_loop_as_its_own_singleton() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(a, a, ());

    let components = BiconnectedComponents.visit(&graph);

    assert_eq!(components.len(), 2);
    assert!(components.contains(&vec![(a, a)]));
    assert!(components.contains(&vec![(a, b)]) || components.contains(&vec![(b, a)]));
}

#[test]
fn bridges_and_articulation_points_of_an_empty_graph_are_empty() {
    let graph = GraphBuilder::<&str, ()>::new().build();

    assert_eq!(Bridges.visit(&graph), Vec::new());
    assert_eq!(ArticulationPoints.visit(&graph), Vec::new());
}

#[test]
fn bridges_and_articulation_points_do_not_interfere_across_disconnected_components() {
    let (mut graph, [_a, _b, c, d, _e, _f]) = two_triangles_joined_by_a_bridge();
    let g = graph.add_vertex("g");
    let h = graph.add_vertex("h");
    let i = graph.add_vertex("i");
    let j = graph.add_vertex("j");
    let k = graph.add_vertex("k");
    let l = graph.add_vertex("l");
    graph.edges_mut().add_edge(g, h, ());
    graph.edges_mut().add_edge(h, i, ());
    graph.edges_mut().add_edge(i, g, ());
    graph.edges_mut().add_edge(j, k, ());
    graph.edges_mut().add_edge(k, l, ());
    graph.edges_mut().add_edge(l, j, ());
    graph.edges_mut().add_edge(i, j, ());

    // Two independent bridge structures in one graph: the shared DFS timer used across
    // `for root in view.ids()` roots must not let one component's discovery order corrupt
    // the other's low-link comparisons.
    let bridges = Bridges.visit(&graph);
    let mut points = ArticulationPoints.visit(&graph);
    points.sort_by_key(|v| v.id());
    let mut expected_points = [c, d, i, j];
    expected_points.sort_by_key(|v| v.id());

    let has_bridge = |x, y| {
        bridges
            .iter()
            .any(|&(from, to)| (from == x && to == y) || (from == y && to == x))
    };
    assert_eq!(bridges.len(), 2);
    assert!(has_bridge(c, d));
    assert!(has_bridge(i, j));
    assert_eq!(points, expected_points);
}

#[test]
fn connected_components_only_follows_outgoing_edges_on_a_one_way_graph() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(b, a, ()); // b -> a, one-way

    let components = ConnectedComponents.visit(&graph);

    // a has no outgoing edge, so it's claimed as its own component before b's outgoing edge
    // to a can merge them: the documented one-way caveat, not a real pair of islands.
    assert_eq!(components, vec![vec![a], vec![b]]);
}

#[test]
fn multi_source_bfs_keeps_a_source_attributed_to_itself_when_reachable_from_another_source() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let s1 = graph.add_vertex("s1");
    let s2 = graph.add_vertex("s2");
    graph.edges_mut().add_edge_directed(s2, s1, ());

    let reached = MultiSourceBfs.visit(&graph, [s1, s2]).unwrap();
    let of = |v| reached.iter().find(|(id, _)| *id == v).map(|(_, r)| *r);

    assert_eq!(
        of(s1),
        Some(Reached {
            distance: 0,
            source: s1
        })
    );
    assert_eq!(
        of(s2),
        Some(Reached {
            distance: 0,
            source: s2
        })
    );
}

#[test]
fn multi_source_bfs_ignores_duplicate_sources() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge(a, b, ());

    let reached = MultiSourceBfs.visit(&graph, [a, a]).unwrap();

    assert_eq!(
        reached,
        vec![
            (
                a,
                Reached {
                    distance: 0,
                    source: a
                }
            ),
            (
                b,
                Reached {
                    distance: 1,
                    source: a
                }
            )
        ]
    );
}

#[test]
fn topological_sort_orders_dependencies_before_dependents() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, c, ());
    graph.edges_mut().add_edge_directed(b, c, ());

    let order = TopologicalSort.visit(&graph).unwrap();

    let position = |v| order.iter().position(|&id| id == v).unwrap();
    assert!(position(a) < position(b));
    assert!(position(b) < position(c));
}

#[test]
fn topological_sort_returns_none_on_a_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, a, ());

    assert_eq!(TopologicalSort.visit(&graph), None);
}

#[test]
fn topological_sort_ignores_an_edge_from_a_vertex_excluded_by_the_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let w = graph.add_vertex("w");
    let x = graph.add_vertex("x");
    graph.edges_mut().add_edge_directed(w, x, ());
    let view = graph.filtered_view(|_, v: &&str| *v != "w", |_| true);

    // w is excluded from the view, so its edge to x doesn't count: x has no in-view
    // predecessor and is free to sort on its own.
    assert_eq!(TopologicalSort.visit(&view), Some(vec![x]));
}

#[test]
fn cycle_detection_finds_a_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());
    graph.edges_mut().add_edge_directed(c, a, ());

    let cycle = CycleDetection.visit(&graph).unwrap();

    // The cycle starts wherever the DFS re-encounters an ancestor, but the three
    // vertices must appear in their cyclic order starting from that point.
    let position = |v| cycle.iter().position(|&id| id == v).unwrap();
    assert_eq!(cycle.len(), 3);
    let start = position(a);
    assert_eq!(cycle[(start + 1) % 3], b);
    assert_eq!(cycle[(start + 2) % 3], c);
}

#[test]
fn cycle_detection_returns_none_on_a_dag() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, c, ());
    graph.edges_mut().add_edge_directed(b, c, ());

    assert_eq!(CycleDetection.visit(&graph), None);
}

#[test]
fn max_flow_computes_the_classic_diamond_example() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, Km(3.0));
    graph.edges_mut().add_edge_directed(a, c, Km(2.0));
    graph.edges_mut().add_edge_directed(b, d, Km(2.0));
    graph.edges_mut().add_edge_directed(c, d, Km(3.0));

    let (flow, min_cut) = MaxFlow::new(d).visit(&graph, a).unwrap();

    assert_eq!(flow, 4.0);
    assert_eq!(min_cut.len(), 2);
    assert!(min_cut.contains(&(a, c)));
    assert!(min_cut.contains(&(b, d)));
}

#[test]
fn max_flow_is_zero_when_sink_is_unreachable() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, Km(5.0));

    let (flow, min_cut) = MaxFlow::new(d).visit(&graph, a).unwrap();

    assert_eq!(flow, 0.0);
    assert!(min_cut.is_empty());
}

#[test]
fn max_flow_min_cut_excludes_an_edge_to_a_vertex_outside_the_filtered_view() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let s = graph.add_vertex("s");
    let a = graph.add_vertex("a");
    let t = graph.add_vertex("t");
    let x = graph.add_vertex("x");
    graph.edges_mut().add_edge_directed(s, a, Km(5.0));
    graph.edges_mut().add_edge_directed(a, t, Km(3.0));
    graph.edges_mut().add_edge_directed(a, x, Km(10.0));
    let view = graph.filtered_view(|_, v: &&str| *v != "x", |_| true);

    let (flow, min_cut) = MaxFlow::new(t).visit(&view, s).unwrap();

    // x is excluded from the view, so a's edge to it doesn't count towards the flow or the
    // cut: only a's edge to t (capacity 3) limits how much can flow from s to t.
    assert_eq!(flow, 3.0);
    assert_eq!(min_cut, vec![(a, t)]);
}

#[test]
fn eulerian_trail_finds_a_circuit() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());
    graph.edges_mut().add_edge_directed(c, a, ());

    let trail = EulerianTrail.visit(&graph).unwrap();

    assert_eq!(trail, vec![a, b, c, a]);
}

#[test]
fn eulerian_trail_finds_an_open_path_between_the_two_unbalanced_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());
    graph.edges_mut().add_edge_directed(c, a, ());
    graph.edges_mut().add_edge_directed(a, d, ());

    let trail = EulerianTrail.visit(&graph).unwrap();

    // Starts at the vertex with one extra outgoing edge (a) and ends at the one with one
    // extra incoming edge (d), using each of the 4 edges exactly once.
    assert_eq!(trail.first(), Some(&a));
    assert_eq!(trail.last(), Some(&d));
    assert_eq!(trail.len(), 5);
}

#[test]
fn eulerian_trail_returns_none_for_a_disconnected_graph() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(c, d, ());

    assert_eq!(EulerianTrail.visit(&graph), None);
}

#[test]
fn eulerian_trail_ignores_edges_touching_a_vertex_excluded_by_the_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let x = graph.add_vertex("x");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, x, ());
    let view = graph.filtered_view(|_, v: &&str| *v != "x", |_| true);

    // x is excluded from the view, so a's edge to it doesn't count: the view exposes only
    // the balanced a-b edge, giving a one-edge trail.
    assert_eq!(EulerianTrail.visit(&view), Some(vec![a, b]));
}

#[test]
fn eccentricity_and_diameter_of_a_path() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, d, ());

    let (eccentricities, diameter) = Eccentricity.visit(&graph);
    let ecc_of = |v| {
        eccentricities
            .iter()
            .find(|(id, _)| *id == v)
            .map(|(_, e)| *e)
            .unwrap()
    };

    assert_eq!(ecc_of(a), 3);
    assert_eq!(ecc_of(b), 2);
    assert_eq!(ecc_of(c), 2);
    assert_eq!(ecc_of(d), 3);
    assert_eq!(diameter, Some(3));
}

#[test]
fn eccentricity_ignores_unreachable_vertices() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let isolated = graph.add_vertex("isolated");
    graph.edges_mut().add_edge(a, b, ());

    let (eccentricities, diameter) = Eccentricity.visit(&graph);
    let ecc_of = |v| {
        eccentricities
            .iter()
            .find(|(id, _)| *id == v)
            .map(|(_, e)| *e)
            .unwrap()
    };

    assert_eq!(ecc_of(a), 1);
    assert_eq!(ecc_of(b), 1);
    assert_eq!(ecc_of(isolated), 0);
    assert_eq!(diameter, Some(1));
}

#[test]
fn betweenness_centrality_ranks_the_middle_of_a_path_highest() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());

    let scores = BetweennessCentrality.visit(&graph);
    let score_of = |v| {
        scores
            .iter()
            .find(|(id, _)| *id == v)
            .map(|(_, s)| *s)
            .unwrap()
    };

    assert_eq!(score_of(a), 0.0);
    assert_eq!(score_of(c), 0.0);
    assert_eq!(score_of(b), 2.0);
}

#[test]
fn betweenness_centrality_of_a_triangle_is_zero_for_everyone() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());

    let scores = BetweennessCentrality.visit(&graph);
    assert!(scores.iter().all(|(_, s)| *s == 0.0));
}

#[test]
fn bipartite_finds_the_two_classes_of_an_even_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, d, ());
    graph.edges_mut().add_edge(d, a, ());

    let (class_a, class_b) = Bipartite.visit(&graph).unwrap();

    assert_eq!(class_a.len(), 2);
    assert_eq!(class_b.len(), 2);
    // a and c are opposite corners of the 4-cycle, always in the same class as each other.
    assert_eq!(class_a.contains(&a), class_a.contains(&c));
    assert_ne!(class_a.contains(&a), class_a.contains(&b));
}

#[test]
fn bipartite_returns_none_for_an_odd_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, c, ());
    graph.edges_mut().add_edge(c, a, ());

    assert_eq!(Bipartite.visit(&graph), None);
}

#[test]
fn transitive_closure_finds_everything_reachable_from_each_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, c, ());
    graph.edges_mut().add_edge_directed(a, d, ());

    let closure = TransitiveClosure.visit(&graph);
    let reachable_from = |v| {
        closure
            .iter()
            .find(|(id, _)| *id == v)
            .map(|(_, r)| r.clone())
            .unwrap()
    };

    let from_a = reachable_from(a);
    assert_eq!(from_a.len(), 3);
    assert!(from_a.contains(&b) && from_a.contains(&c) && from_a.contains(&d));
    assert_eq!(reachable_from(b), vec![c]);
    assert_eq!(reachable_from(c), Vec::<_>::new());
    assert_eq!(reachable_from(d), Vec::<_>::new());
}

#[test]
fn transitive_closure_includes_self_when_on_a_cycle() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(b, a, ());

    let closure = TransitiveClosure.visit(&graph);
    let from_a = closure
        .iter()
        .find(|(id, _)| *id == a)
        .map(|(_, r)| r.clone())
        .unwrap();

    assert!(from_a.contains(&a) && from_a.contains(&b));
}

#[test]
fn all_simple_paths_enumerates_every_route_to_the_target() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, ());
    graph.edges_mut().add_edge_directed(a, c, ());
    graph.edges_mut().add_edge_directed(b, d, ());
    graph.edges_mut().add_edge_directed(c, d, ());
    graph.edges_mut().add_edge_directed(b, c, ()); // opens a third route: a-b-c-d

    let paths = AllSimplePaths::new(d).visit(&graph, a).unwrap();

    assert_eq!(paths.len(), 3);
    assert!(paths.contains(&vec![a, b, d]));
    assert!(paths.contains(&vec![a, c, d]));
    assert!(paths.contains(&vec![a, b, c, d]));
}

#[test]
fn all_simple_paths_is_empty_for_an_unreachable_target() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");

    assert_eq!(
        AllSimplePaths::new(b).visit(&graph, a).unwrap(),
        Vec::<Vec<_>>::new()
    );
}

#[test]
fn astar_finds_shortest_path_with_an_admissible_heuristic() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, Km(5.0));
    graph.edges_mut().add_edge_directed(a, c, Km(1.0));
    graph.edges_mut().add_edge_directed(c, b, Km(1.0));

    let heuristic = move |v| {
        if v == b {
            0.0
        } else if v == c {
            0.5
        } else {
            1.0
        }
    };
    let (path, total) = AStar::new(b, heuristic)
        .visit(&graph, a, |edge| edge.data.0)
        .unwrap()
        .unwrap();
    assert_eq!(path, vec![a, c, b]);
    assert_eq!(total, 2.0);
}

#[test]
fn astar_returns_none_for_unreachable_target() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");

    assert_eq!(
        AStar::new(b, |_| 0.0)
            .visit(&graph, a, |edge| edge.data.0)
            .unwrap(),
        None
    );
}

#[test]
fn bellman_ford_finds_shortest_path_with_a_negative_edge() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, Km(5.0));
    graph.edges_mut().add_edge_directed(a, c, Km(2.0));
    graph.edges_mut().add_edge_directed(c, b, Km(-4.0));

    let (path, total) = BellmanFord::new(b).visit(&graph, a).unwrap().unwrap();
    assert_eq!(path, vec![a, c, b]);
    assert_eq!(total, -2.0);
}

#[test]
fn bellman_ford_reports_a_reachable_negative_cycle() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, b, Km(1.0));
    graph.edges_mut().add_edge_directed(b, a, Km(-3.0));

    assert_eq!(
        BellmanFord::new(b).visit(&graph, a),
        Err(VisitError::NegativeCycle)
    );
}

#[test]
fn bellman_ford_does_not_report_a_false_negative_cycle_on_a_filtered_view() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let s = graph.add_vertex("s");
    let m1 = graph.add_vertex("m1");
    let m2 = graph.add_vertex("m2");
    let t = graph.add_vertex("t");
    graph.edges_mut().add_edge_directed(s, m1, Km(1.0));
    graph.edges_mut().add_edge_directed(m1, m2, Km(1.0));
    graph.edges_mut().add_edge_directed(m2, t, Km(1.0));
    let view = graph.filtered_view(|_, v: &&str| *v == "s" || *v == "t", |_| true);

    // m1 and m2 are vertex-filtered out of the view, so their edges don't relax and s can't
    // reach t: the bound must still be settled without a spurious negative-cycle report.
    assert_eq!(BellmanFord::new(t).visit(&view, s).unwrap(), None);
}

#[test]
fn minimum_spanning_tree_finds_the_cheapest_connecting_edges() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge(a, b, Km(1.0));
    graph.edges_mut().add_edge(b, c, Km(2.0));
    graph.edges_mut().add_edge(c, d, Km(3.0));
    graph.edges_mut().add_edge(a, c, Km(4.0)); // would close a cycle, must be skipped
    graph.edges_mut().add_edge(a, d, Km(10.0)); // would close a cycle, must be skipped

    let (mst, total) = MinimumSpanningTree.visit(&graph);

    assert_eq!(total, 6.0);
    assert_eq!(mst.len(), 3);
    let has_edge = |x, y| {
        mst.iter()
            .any(|&(f, t, _)| (f == x && t == y) || (f == y && t == x))
    };
    assert!(has_edge(a, b));
    assert!(has_edge(b, c));
    assert!(has_edge(c, d));
}

#[test]
fn minimum_spanning_tree_builds_a_forest_over_disconnected_components() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    let _isolated = graph.add_vertex("isolated");
    graph.edges_mut().add_edge(a, b, Km(1.0));
    graph.edges_mut().add_edge(c, d, Km(2.0));

    let (mst, total) = MinimumSpanningTree.visit(&graph);

    assert_eq!(total, 3.0);
    assert_eq!(mst.len(), 2);
}

#[test]
fn minimum_spanning_tree_handles_a_filtered_view_with_gaps_in_vertex_ids() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let _b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    let e = graph.add_vertex("e");
    graph.edges_mut().add_edge(a, c, Km(1.0));
    graph.edges_mut().add_edge(d, e, Km(2.0));
    let view = graph.filtered_view(|_, v: &&str| *v != "b", |_| true);

    // The view's own vertex count (4) is smaller than e's raw id (4), since b is excluded:
    // a union-find sized to `view.len()` would index out of bounds on e.
    let (mst, total) = MinimumSpanningTree.visit(&view);

    assert_eq!(total, 3.0);
    assert_eq!(mst.len(), 2);
}

#[test]
fn multi_source_dijkstra_keeps_a_source_attributed_to_itself_when_reachable_more_cheaply() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let s1 = graph.add_vertex("s1");
    let s2 = graph.add_vertex("s2");
    graph.edges_mut().add_edge_directed(s2, s1, Km(0.001));

    let reached = MultiSourceDijkstra::new(f64::INFINITY)
        .visit(&graph, [s1, s2], |edge| edge.data.0)
        .unwrap();
    let of = |v| reached.iter().find(|(id, _)| *id == v).map(|(_, r)| *r);

    assert_eq!(
        of(s1),
        Some(DijkstraReached {
            distance: 0.0,
            source: s1
        })
    );
    assert_eq!(
        of(s2),
        Some(DijkstraReached {
            distance: 0.0,
            source: s2
        })
    );
}

#[test]
fn multi_source_dijkstra_ignores_duplicate_sources() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    graph.edges_mut().add_edge_directed(a, b, Km(1.0));

    let reached = MultiSourceDijkstra::new(f64::INFINITY)
        .visit(&graph, [a, a], |edge| edge.data.0)
        .unwrap();

    assert_eq!(
        reached,
        vec![
            (
                a,
                DijkstraReached {
                    distance: 0.0,
                    source: a
                }
            ),
            (
                b,
                DijkstraReached {
                    distance: 1.0,
                    source: a
                }
            )
        ]
    );
}

#[test]
fn priority_flood_raises_a_pit_to_the_level_it_is_flooded_from() {
    let mut graph = GraphBuilder::<f64, ()>::new().build();
    let s = graph.add_vertex(0.0);
    let x = graph.add_vertex(3.0);
    let y = graph.add_vertex(1.0);
    let pit = graph.add_vertex(0.0);
    graph.edges_mut().add_edge(s, x, ());
    graph.edges_mut().add_edge(s, y, ());
    graph.edges_mut().add_edge(y, pit, ());

    let flooding = PriorityFlood
        .visit(&graph, [s], |id| *graph.vertex(id).unwrap())
        .unwrap();

    let flooded = |level, from| Flooded { level, from };
    assert_eq!(
        flooding,
        vec![
            (s, flooded(0.0, None)),
            (y, flooded(1.0, Some(s))),
            (pit, flooded(1.0, Some(y))),
            (x, flooded(3.0, Some(s))),
        ]
    );
}

#[test]
fn priority_flood_reports_missing_seed_vertex() {
    let graph = GraphBuilder::<f64, ()>::new().build();

    assert_eq!(
        PriorityFlood.visit(&graph, [bergraph::VertexId::new(0)], |_| 0.0),
        Err(VisitError::VertexNotFound)
    );
}

#[test]
fn priority_flood_never_refloods_a_seed_and_breaks_ties_on_the_lower_id() {
    let mut graph = GraphBuilder::<f64, ()>::new().build();
    let a = graph.add_vertex(2.0);
    let b = graph.add_vertex(0.0);
    let c = graph.add_vertex(1.0);
    let d = graph.add_vertex(1.0);
    graph.edges_mut().add_edge(a, b, ());
    graph.edges_mut().add_edge(b, d, ());
    graph.edges_mut().add_edge(b, c, ());

    let flooding = PriorityFlood
        .visit(&graph, [b, a, b], |id| *graph.vertex(id).unwrap())
        .unwrap();

    let flooded = |level, from| Flooded { level, from };
    assert_eq!(
        flooding,
        vec![
            (b, flooded(0.0, None)),
            (c, flooded(1.0, Some(b))),
            (d, flooded(1.0, Some(b))),
            (a, flooded(2.0, None)),
        ]
    );
}

#[test]
fn search_buffers_reused_across_searches_give_the_same_results() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge_directed(a, b, Km(1.0));
    graph.edges_mut().add_edge_directed(b, c, Km(1.0));
    graph.edges_mut().add_edge_directed(a, c, Km(5.0));
    let cost = |edge: &bergraph::graph_view::EdgeView<'_, Km, u32>| edge.data.0;
    let mut buffers = SearchBuffers::default();

    let path = Dijkstra::new(c).visit_with(&mut buffers, &graph, a, cost);
    assert_eq!(path, Dijkstra::new(c).visit(&graph, a, cost));
    let near = MultiSourceDijkstra::new(1.0).visit_with(&mut buffers, &graph, [b], cost);
    assert_eq!(near, MultiSourceDijkstra::new(1.0).visit(&graph, [b], cost));
    let guided = AStar::new(c, |_| 0.0).visit_with(&mut buffers, &graph, a, cost);
    assert_eq!(guided, path);
    // Nothing reaches `a`: a distance left over from the searches above would report a path.
    assert_eq!(
        Dijkstra::new(a).visit_with(&mut buffers, &graph, c, cost),
        Ok(None)
    );
    let missing = bergraph::VertexId::new(9);
    let failed = MultiSourceDijkstra::new(f64::INFINITY).visit_with(
        &mut buffers,
        &graph,
        [b, missing],
        cost,
    );
    assert_eq!(failed, Err(VisitError::VertexNotFound));
    assert_eq!(
        Dijkstra::new(a).visit_with(&mut buffers, &graph, b, cost),
        Ok(None)
    );
}

#[test]
fn a_star_skips_a_stale_queue_entry() {
    let mut graph = GraphBuilder::<&str, Km>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let d = graph.add_vertex("d");
    graph.edges_mut().add_edge_directed(a, b, Km(5.0));
    graph.edges_mut().add_edge_directed(a, c, Km(1.0));
    graph.edges_mut().add_edge_directed(c, b, Km(1.0));
    graph.edges_mut().add_edge_directed(b, d, Km(10.0));

    // b is queued at 5 then at 2: the entry at 5 pops before d and must be skipped.
    let shortest = AStar::new(d, |_| 0.0).visit(&graph, a, |edge| edge.data.0);
    assert_eq!(shortest, Ok(Some((vec![a, c, b, d], 12.0))));
}

#[test]
fn visit_error_propagates_into_a_boxed_error() {
    fn first_reached() -> Result<usize, Box<dyn std::error::Error>> {
        let graph = GraphBuilder::<&str, ()>::new().build();
        Ok(Bfs.visit(&graph, bergraph::VertexId::new(0))?.len())
    }

    let error = first_reached().unwrap_err();
    assert_eq!(error.to_string(), "vertex not found in the graph view");
}
