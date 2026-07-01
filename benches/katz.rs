use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_katz_centrality::{Graph, KatzParams, katz_centrality, parse_edge_list};

/// Generate a Watts-Strogatz-like edge list with ~5 000 nodes for benchmarking.
/// We use a deterministic ring + short-range rewiring to stay within alpha < 1/lambda_max.
fn make_ws5k_edges() -> String {
    let n = 5_000usize;
    let k = 4usize; // each node connected to k nearest neighbours on each side
    let mut edges = Vec::with_capacity(n * k);
    for i in 0..n {
        for d in 1..=k {
            let j = (i + d) % n;
            edges.push(format!("{i} {j}"));
        }
    }
    edges.join("\n")
}

fn bench_compute(c: &mut Criterion) {
    let edge_text = make_ws5k_edges();
    let edges = parse_edge_list(&edge_text);
    let graph = Graph::from_edges(edges);
    let params = KatzParams {
        alpha: 0.05,
        beta: 1.0,
        max_iter: 1000,
        tol: 1e-6,
        normalized: true,
    };

    c.bench_function("katz_5k_nodes_compute_only", |b| {
        b.iter(|| katz_centrality(&graph, &params).unwrap());
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
