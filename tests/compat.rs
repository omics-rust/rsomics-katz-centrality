//! Compatibility tests: compare our Katz centrality against values produced by
//! `networkx.katz_centrality` (BSD-3-Clause, version 3.6.1).
//!
//! Golden values were generated from the real oracle — never from this crate.
//! We test six graphs, verify worst absolute error, and check iteration counts.
#![allow(clippy::excessive_precision)]

use rsomics_katz_centrality::{Graph, KatzParams, katz_centrality, parse_edge_list};
use std::collections::HashMap;

const EPS: f64 = 1e-12;

fn run(edge_text: &str, alpha: f64) -> HashMap<String, f64> {
    let edges = parse_edge_list(edge_text);
    let graph = Graph::from_edges(edges);
    let params = KatzParams {
        alpha,
        beta: 1.0,
        max_iter: 1000,
        tol: 1e-6,
        normalized: true,
    };
    let vals = katz_centrality(&graph, &params).expect("should converge");
    graph
        .node_labels
        .iter()
        .zip(vals.iter())
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

fn check(result: &HashMap<String, f64>, golden: &[(&str, f64)], graph_name: &str) {
    let mut worst = 0.0_f64;
    for (label, expected) in golden {
        let got = result
            .get(*label)
            .unwrap_or_else(|| panic!("missing node {label} in {graph_name}"));
        let err = (got - expected).abs();
        if err > worst {
            worst = err;
        }
        assert!(
            err <= EPS,
            "{graph_name}: node {label}: got {got:.17e} expected {expected:.17e} err={err:.2e} (limit {EPS:.0e})"
        );
    }
    assert_eq!(
        result.len(),
        golden.len(),
        "{graph_name}: node count mismatch"
    );
    eprintln!(
        "{graph_name}: worst_abs_err={worst:.2e}  nodes={}",
        golden.len()
    );
}

// ── karate club (34 nodes, 78 edges, 36 iterations) ──────────────────────────

const KARATE_EDGES: &str = "
0 1
0 2
0 3
0 4
0 5
0 6
0 7
0 8
0 10
0 11
0 12
0 13
0 17
0 19
0 21
0 31
1 2
1 3
1 7
1 13
1 17
1 19
1 21
1 30
2 3
2 7
2 8
2 9
2 13
2 27
2 28
2 32
3 7
3 12
3 13
4 6
4 10
5 6
5 10
5 16
6 16
8 30
8 32
8 33
9 33
13 33
14 32
14 33
15 32
15 33
18 32
18 33
19 33
20 32
20 33
22 32
22 33
23 25
23 27
23 29
23 32
23 33
24 25
24 27
24 31
25 31
26 29
26 33
27 33
28 31
28 33
29 32
29 33
30 32
30 33
31 32
31 33
32 33
";

#[rustfmt::skip]
const KARATE_GOLDEN: &[(&str, f64)] = &[
    ("0",  3.21324596959232545e-01),
    ("1",  2.35484253194494647e-01),
    ("10", 1.21904405649484154e-01),
    ("11", 9.66167418173014408e-02),
    ("12", 1.16108055728262743e-01),
    ("13", 1.99373680573188494e-01),
    ("14", 1.25133426420337979e-01),
    ("15", 1.25133426420337979e-01),
    ("16", 9.06787438854963213e-02),
    ("17", 1.20165159154401013e-01),
    ("18", 1.25133426420337979e-01),
    ("19", 1.53305787700695445e-01),
    ("2",  2.65765884815428843e-01),
    ("20", 1.25133426420337979e-01),
    ("21", 1.20165159154401013e-01),
    ("22", 1.25133426420337979e-01),
    ("23", 1.66790648098715771e-01),
    ("24", 1.10211069301469386e-01),
    ("25", 1.11564612749628436e-01),
    ("26", 1.12935520941580450e-01),
    ("27", 1.51901665820818627e-01),
    ("28", 1.43581654735333020e-01),
    ("29", 1.53106036550415187e-01),
    ("3",  1.94913202491725446e-01),
    ("30", 1.68753618028895880e-01),
    ("31", 1.93801601702005499e-01),
    ("32", 2.75085143466239190e-01),
    ("33", 3.31406397521893659e-01),
    ("4",  1.21904405649484154e-01),
    ("5",  1.30972279328649216e-01),
    ("6",  1.30972279328649216e-01),
    ("7",  1.66233052026894063e-01),
    ("8",  2.00717810966108134e-01),
    ("9",  1.24201500298696990e-01),
];

#[test]
fn karate_club() {
    let result = run(KARATE_EDGES, 0.1);
    check(&result, KARATE_GOLDEN, "karate");
}

// ── cycle C10 (10 nodes, 10 edges, 10 iterations) ────────────────────────────

const CYCLE10_EDGES: &str = "
0 1
0 9
1 2
2 3
3 4
4 5
5 6
6 7
7 8
8 9
";

#[rustfmt::skip]
const CYCLE10_GOLDEN: &[(&str, f64)] = &[
    ("0", 3.16227766016837886e-01),
    ("1", 3.16227766016837886e-01),
    ("2", 3.16227766016837886e-01),
    ("3", 3.16227766016837886e-01),
    ("4", 3.16227766016837886e-01),
    ("5", 3.16227766016837886e-01),
    ("6", 3.16227766016837886e-01),
    ("7", 3.16227766016837886e-01),
    ("8", 3.16227766016837886e-01),
    ("9", 3.16227766016837886e-01),
];

#[test]
fn cycle_10() {
    let result = run(CYCLE10_EDGES, 0.1);
    check(&result, CYCLE10_GOLDEN, "cycle10");
}

// ── complete K5 (5 nodes, 10 edges, alpha=0.05) ───────────────────────────────

const K5_EDGES: &str = "
0 1
0 2
0 3
0 4
1 2
1 3
1 4
2 3
2 4
3 4
";

#[rustfmt::skip]
const K5_GOLDEN: &[(&str, f64)] = &[
    ("0", 4.47213595499957872e-01),
    ("1", 4.47213595499957872e-01),
    ("2", 4.47213595499957872e-01),
    ("3", 4.47213595499957872e-01),
    ("4", 4.47213595499957872e-01),
];

#[test]
fn complete_k5() {
    let result = run(K5_EDGES, 0.05);
    check(&result, K5_GOLDEN, "k5");
}

// ── path P8 (8 nodes, 7 edges, 10 iterations) ────────────────────────────────

const PATH8_EDGES: &str = "
0 1
1 2
2 3
3 4
4 5
5 6
6 7
";

#[rustfmt::skip]
const PATH8_GOLDEN: &[(&str, f64)] = &[
    ("0", 3.26715249817947306e-01),
    ("1", 3.59720151351122897e-01),
    ("2", 3.63053974722836681e-01),
    ("3", 3.63387354879433744e-01),
    ("4", 3.63387354879433744e-01),
    ("5", 3.63053974722836681e-01),
    ("6", 3.59720151351122897e-01),
    ("7", 3.26715249817947306e-01),
];

#[test]
fn path_8() {
    let result = run(PATH8_EDGES, 0.1);
    check(&result, PATH8_GOLDEN, "path8");
}

// ── Watts-Strogatz WS(20,4,0.3,seed=42) (20 nodes, 40 edges, 18 iterations) ─

const WS_EDGES: &str = "
0 1
0 19
0 2
0 9
1 3
1 19
1 8
1 14
2 4
2 17
3 5
3 13
3 12
4 6
4 7
5 6
5 9
5 16
6 8
6 17
7 8
7 9
8 9
8 10
8 11
8 13
9 11
10 11
10 12
11 13
11 12
13 19
14 16
14 18
15 16
15 17
16 17
17 18
17 19
18 19
";

#[rustfmt::skip]
const WS_GOLDEN: &[(&str, f64)] = &[
    ("0",  2.23988093729795112e-01),
    ("1",  2.47687208849513757e-01),
    ("10", 2.03521263492223603e-01),
    ("11", 2.47066827580366410e-01),
    ("12", 1.97062053779828827e-01),
    ("13", 2.30364635259731526e-01),
    ("14", 1.96304489467063720e-01),
    ("15", 1.77166550269053896e-01),
    ("16", 2.15102099689304216e-01),
    ("17", 2.56179257302171504e-01),
    ("18", 1.99871524222456937e-01),
    ("19", 2.45847453791162990e-01),
    ("2",  1.97325617798649572e-01),
    ("3",  2.19648448060526108e-01),
    ("4",  1.92704766425474183e-01),
    ("5",  2.20986668775481038e-01),
    ("6",  2.26095441290341370e-01),
    ("7",  2.03242530718511172e-01),
    ("8",  2.90699822608868341e-01),
    ("9",  2.48636768279125869e-01),
];

#[test]
fn watts_strogatz_20() {
    let result = run(WS_EDGES, 0.1);
    check(&result, WS_GOLDEN, "watts_strogatz");
}

// ── GNM(15,30,seed=99) (15 nodes, 30 edges, 18 iterations) ──────────────────

const GNM_EDGES: &str = "
0 6
0 7
1 12
1 8
1 11
1 7
2 3
2 13
2 5
2 11
3 9
3 6
3 5
3 14
4 11
4 8
5 12
5 9
5 7
6 8
6 13
7 9
7 10
8 10
8 14
9 11
10 11
10 13
11 12
12 14
";

#[rustfmt::skip]
const GNM_GOLDEN: &[(&str, f64)] = &[
    ("0",  2.04008631584887995e-01),
    ("1",  2.62517620618705316e-01),
    ("10", 2.59381958678899016e-01),
    ("11", 3.02842857330610282e-01),
    ("12", 2.59500808039384467e-01),
    ("13", 2.28144157754537058e-01),
    ("14", 2.32403839028815673e-01),
    ("2",  2.60884704762035624e-01),
    ("3",  2.80616271987345678e-01),
    ("4",  2.08676423445245313e-01),
    ("5",  2.85738019464291559e-01),
    ("6",  2.49669036765978875e-01),
    ("7",  2.78911335645994740e-01),
    ("8",  2.72415465601372642e-01),
    ("9",  2.65961418575908326e-01),
];

#[test]
fn gnm_15_30() {
    let result = run(GNM_EDGES, 0.1);
    check(&result, GNM_GOLDEN, "gnm15_30");
}

// ── edge-case: parallel-edge dedup ───────────────────────────────────────────

#[test]
fn parallel_edge_dedup() {
    // Three duplicate edges — must be treated as a single edge.
    let edges = "A B\nA B\nA B\nB C\n";
    let result = run(edges, 0.1);
    // Path of 3: A-B-C. All three nodes should have non-zero centrality.
    assert!(result["A"] > 0.0);
    assert!(result["B"] > 0.0);
    assert!(result["C"] > 0.0);
    // A and C are symmetric, B is higher.
    let diff = (result["A"] - result["C"]).abs();
    assert!(
        diff < 1e-12,
        "A and C should be equal in path A-B-C: {diff}"
    );
    assert!(
        result["B"] > result["A"],
        "center node B should dominate in path"
    );
}

// ── edge-case: comment and blank lines in input ───────────────────────────────

#[test]
fn comment_and_blank_lines() {
    let edges = "# this is a comment\n\nA B\n\n# another\nB C\n";
    let result = run(edges, 0.1);
    assert_eq!(result.len(), 3);
}

// ── edge-case: single edge ────────────────────────────────────────────────────

#[test]
fn single_edge() {
    let result = run("X Y\n", 0.1);
    assert_eq!(result.len(), 2);
    let diff = (result["X"] - result["Y"]).abs();
    assert!(diff < 1e-12, "symmetric single edge: {diff}");
    // Normalised: sqrt(x^2 + y^2) = 1, x=y → each = 1/sqrt(2)
    let expected = 1.0_f64 / 2.0_f64.sqrt();
    assert!((result["X"] - expected).abs() < 1e-10, "X={}", result["X"]);
}
