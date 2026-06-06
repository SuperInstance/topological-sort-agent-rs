use criterion::{black_box, criterion_group, criterion_main, Criterion};
use topological_sort_agent::TopoGraph;

fn bench_kahn_small(c: &mut Criterion) {
    let mut g = TopoGraph::new();
    for i in 0u64..100 {
        if i > 0 { g.add_edge(i - 1, i); }
    }
    c.bench_function("kahn_sort_100_nodes", |b| b.iter(|| g.kahn_sort()));
}

fn bench_kahn_large(c: &mut Criterion) {
    let mut g = TopoGraph::new();
    for i in 0u64..10000 {
        g.add_node(i);
        if i > 0 && i % 2 == 0 { g.add_edge(i - 1, i); }
        if i > 2 { g.add_edge(i - 3, i); }
    }
    c.bench_function("kahn_sort_10k_nodes", |b| b.iter(|| g.kahn_sort()));
}

fn bench_parallel_large(c: &mut Criterion) {
    let mut g = TopoGraph::new();
    for i in 0u64..10000 {
        g.add_node(i);
        if i > 0 && i % 2 == 0 { g.add_edge(i - 1, i); }
    }
    c.bench_function("parallel_sort_10k_nodes", |b| b.iter(|| g.parallel_sort()));
}

criterion_group!(benches, bench_kahn_small, bench_kahn_large, bench_parallel_large);
criterion_main!(benches);
