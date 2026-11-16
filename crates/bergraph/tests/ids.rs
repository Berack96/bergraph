use bergraph::graph_view::EdgeView;
use bergraph::{EdgeId, VertexId};

#[test]
fn creates_vertex_ids_from_usize() {
    let id = VertexId::<u16>::new(7);

    assert_eq!(id.id(), 7);
}

#[test]
fn creates_the_highest_representable_vertex_id() {
    let id = VertexId::<u8>::new((u8::MAX - 1) as usize);

    assert_eq!(id.id(), (u8::MAX - 1) as usize);
}

#[test]
#[should_panic(expected = "ID overflow")]
fn rejects_an_index_above_the_id_range() {
    VertexId::<u8>::new(256);
}

#[test]
#[should_panic(expected = "ID overflow")]
fn rejects_the_index_equal_to_the_id_maximum() {
    VertexId::<u8>::new(u8::MAX as usize);
}

#[test]
fn orders_vertex_ids_by_their_dense_index() {
    let first = VertexId::<u16>::new(0);
    let second = VertexId::<u16>::new(1);
    let second_again = VertexId::<u16>::new(1);

    assert!(first < second);
    assert!(second > first);
    assert_eq!(second.cmp(&second_again), std::cmp::Ordering::Equal);
}

#[test]
fn creates_edge_views() {
    let data = String::from("edge");
    let from = VertexId::<u32>::new(1);
    let to = VertexId::<u32>::new(2);
    let edge = EdgeView::new(from, to, &data);

    assert_eq!(edge.from, from);
    assert_eq!(edge.to, to);
    assert_eq!(edge.data, &data);
}

#[test]
fn creates_edge_ids_and_orders_them_by_index() {
    let first = EdgeId::<u16>::new(0);
    let second = EdgeId::<u16>::new(1);

    assert_eq!(second.id(), 1);
    assert!(first < second);
}

#[test]
fn keeps_an_optional_edge_id_as_small_as_the_id() {
    assert_eq!(
        std::mem::size_of::<Option<EdgeId<u32>>>(),
        std::mem::size_of::<u32>()
    );
}

#[test]
#[should_panic(expected = "ID overflow")]
fn rejects_an_edge_index_above_the_id_range() {
    EdgeId::<u8>::new(255);
}
