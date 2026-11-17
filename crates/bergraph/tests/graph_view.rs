use bergraph::graph_edges::{GraphEdgeStorage, GraphEdgeStorageMut};
use bergraph::graph_view::GraphView;
use bergraph::{GraphBuilder, VertexId};

#[test]
fn reads_vertices_from_a_graph_view() {
    let mut graph = GraphBuilder::<String, ()>::new().build();
    let vertex = graph.add_vertex(String::from("vertex"));
    let view = graph.view();

    assert_eq!(view.vertex(vertex), Some(&String::from("vertex")));
    assert_eq!(view.vertex(VertexId::new(1)), None);
}

#[test]
fn iterates_no_edges_from_an_empty_graph() {
    let graph = GraphBuilder::<String, ()>::new().build();
    let view = graph.view();

    assert_eq!(view.edges().count(), 0);
}

#[test]
fn ids_enumerates_every_vertex_in_order() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");

    assert_eq!(graph.view().ids().collect::<Vec<_>>(), vec![a, b, c]);
}

#[test]
fn ids_skips_vertices_filtered_out_of_the_view() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let _b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let view = graph.filtered_view(|v: &&str| *v != "b", |_| true);

    assert_eq!(view.ids().collect::<Vec<_>>(), vec![a, c]);
}

#[test]
fn is_empty_is_false_for_a_view_with_no_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    graph.add_vertex("a");
    let view = graph.view();

    assert!(!view.is_empty());
}

#[test]
fn is_empty_is_true_when_the_filter_excludes_every_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    graph.add_vertex("a");
    let view = graph.filtered_view(|_: &&str| false, |_| true);

    assert!(view.is_empty());
}

#[test]
fn edges_of_a_vertex_exclude_arcs_reaching_a_vertex_excluded_by_the_filter() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let x = graph.add_vertex("x");
    graph.edges_mut().add_edge(a, x, ());
    let view = graph.filtered_view(|v: &&str| *v != "x", |_| true);

    assert_eq!(view.edges().of(a).count(), 0);
}

#[test]
fn is_empty_is_false_when_the_filter_keeps_at_least_one_vertex() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    graph.add_vertex("a");
    graph.add_vertex("b");
    let view = graph.filtered_view(|v: &&str| *v == "a", |_| true);

    assert!(!view.is_empty());
}

#[test]
fn filter_accepts_a_closure_capturing_runtime_state() {
    let mut graph = GraphBuilder::<&str, ()>::new().build();
    let a = graph.add_vertex("a");
    let _b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    let excluded = String::from("b");
    let view = graph.filtered_view(move |v: &&str| **v != excluded, |_| true);

    assert_eq!(view.ids().collect::<Vec<_>>(), vec![a, c]);
}

#[test]
fn edge_filter_accepts_a_closure_capturing_runtime_state() {
    let mut graph = GraphBuilder::<&str, i32>::new().build();
    let a = graph.add_vertex("a");
    let b = graph.add_vertex("b");
    let c = graph.add_vertex("c");
    graph.edges_mut().add_edge(a, b, 1);
    graph.edges_mut().add_edge(a, c, 5);
    let threshold = 2;
    let view = graph.filtered_view(|_| true, move |edge| *edge.data < threshold);

    assert_eq!(view.edges().of(a).count(), 1);
}

#[test]
fn filtered_view_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>(_: &T) {}
    let mut graph = GraphBuilder::<&str, i32>::new().build();
    graph.add_vertex("a");
    let excluded = String::from("b");

    assert_send_sync(&graph.filtered_view(move |v: &&str| *v != excluded, |_| true));
}

#[test]
fn filtered_view_debug_shows_the_graph_but_not_the_filters() {
    let mut graph = GraphBuilder::<&str, i32>::new().build();
    graph.add_vertex("a");

    let debug = format!("{:?}", graph.filtered_view(|_| true, |_| true));
    assert!(debug.starts_with("GraphFilteredView { graph: Graph {") && debug.ends_with(".. }"));
}
