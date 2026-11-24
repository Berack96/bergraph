use bergraph::{
    Graph, GraphBuilder, VertexId,
    graph_edges::{
        AdjCsr, AdjList, AdjListFixed, AdjMatrix, GraphEdgeStorage, GraphEdgeStorageDirectedMut,
        GraphEdgeStorageMut,
    },
    graph_visit::{Dijkstra, MultiSourceDijkstra, PriorityFlood},
};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

const VERTICES: usize = 1_024;
const EDGES_PER_VERTEX: usize = 4;

fn build_adj_list() -> AdjList<(), u32> {
    let mut storage = AdjList::with_capacity(VERTICES);
    add_edges(&mut storage);
    storage
}

fn build_adj_list_fixed() -> AdjListFixed<(), u32, EDGES_PER_VERTEX> {
    let mut storage = AdjListFixed::with_capacity(VERTICES);
    add_edges(&mut storage);
    storage
}

fn build_csr() -> AdjCsr {
    let edges: Vec<_> = (0..VERTICES)
        .flat_map(|from| {
            (0..EDGES_PER_VERTEX).map(move |offset| {
                let to = (from + offset + 1) % VERTICES;
                (VertexId::new(from), VertexId::new(to), ())
            })
        })
        .collect();
    AdjCsr::directed(VERTICES, &edges).unwrap().0
}

fn build_matrix() -> AdjMatrix<(), u32> {
    let mut storage = AdjMatrix::with_capacity(VERTICES);
    add_edges(&mut storage);
    storage
}

fn add_edges<S: GraphEdgeStorageDirectedMut<Edge = (), Id = u32>>(storage: &mut S) {
    for from in 0..VERTICES {
        for offset in 0..EDGES_PER_VERTEX {
            let to = (from + offset + 1) % VERTICES;
            storage.add_edge_directed(VertexId::new(from), VertexId::new(to), ());
        }
    }
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn adj_list_build() -> AdjList<(), u32> {
    build_adj_list()
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn adj_list_fixed_build() -> AdjListFixed<(), u32, EDGES_PER_VERTEX> {
    build_adj_list_fixed()
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn adj_list_iterate(bencher: divan::Bencher) {
    let storage = build_adj_list();
    bencher.bench(|| divan::black_box(storage.all().count()));
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn adj_list_fixed_iterate(bencher: divan::Bencher) {
    let storage = build_adj_list_fixed();
    bencher.bench(|| divan::black_box(storage.all().count()));
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn csr_build() -> AdjCsr {
    build_csr()
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn csr_iterate(bencher: divan::Bencher) {
    let storage = build_csr();
    bencher.bench(|| divan::black_box(storage.all().count()));
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn matrix_build() -> AdjMatrix<(), u32> {
    build_matrix()
}

#[divan::bench(counter = divan::counter::ItemsCount::new(
    VERTICES * EDGES_PER_VERTEX,
))]
fn matrix_iterate(bencher: divan::Bencher) {
    let storage = build_matrix();
    bencher.bench(|| divan::black_box(storage.all().count()));
}

/// Side of the periodic triangular lattice the visits run on.
const LATTICE_SIDE: usize = 1024;

/// Pseudo-random value in `[0, 1)` for `k`.
fn noise(k: usize) -> f64 {
    let mut h = (k as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Periodic triangular lattice (six neighbors per vertex) and a weight per edge.
fn build_lattice() -> (Graph<(), AdjCsr>, Vec<f64>) {
    let n = LATTICE_SIDE;
    let id = |x: usize, y: usize| VertexId::new((y % n) * n + x % n);
    let edges: Vec<_> = (0..n * n)
        .flat_map(|k| {
            let (x, y) = (k % n, k / n);
            [(1, 0), (0, 1), (1, 1)].map(|(dx, dy)| (id(x, y), id(x + dx, y + dy), ()))
        })
        .collect();
    let (graph, _) = GraphBuilder::new()
        .with_vertices(vec![(); n * n])
        .build_csr_undirected(&edges)
        .unwrap();
    let weights = (0..2 * edges.len()).map(|e| 1.0 + noise(e)).collect();
    (graph, weights)
}

#[divan::bench(sample_count = 20)]
fn dijkstra_visit_all(bencher: divan::Bencher) {
    let (graph, weights) = build_lattice();
    bencher
        .bench(|| Dijkstra::visit_all(&graph, VertexId::new(0), |e| weights[e.key.id()]).unwrap());
}

#[divan::bench(sample_count = 20)]
fn multi_source_dijkstra(bencher: divan::Bencher) {
    let (graph, weights) = build_lattice();
    let sources: Vec<_> = (0..64)
        .map(|k| VertexId::new(k * 4099 % (LATTICE_SIDE * LATTICE_SIDE)))
        .collect();
    bencher.bench(|| {
        MultiSourceDijkstra::new(f64::INFINITY)
            .visit(&graph, sources.iter().copied(), |e| weights[e.key.id()])
            .unwrap()
    });
}

#[divan::bench(sample_count = 20)]
fn priority_flood(bencher: divan::Bencher) {
    let (graph, _) = build_lattice();
    let height: Vec<f64> = (0..LATTICE_SIDE * LATTICE_SIDE).map(noise).collect();
    let seeds: Vec<_> = (0..height.len())
        .filter(|&k| height[k] < 0.01)
        .map(VertexId::new)
        .collect();
    bencher.bench(|| {
        PriorityFlood
            .visit(&graph, seeds.iter().copied(), |v| height[v.id()])
            .unwrap()
    });
}

fn main() {
    divan::main();
}
