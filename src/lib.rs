//! Katz centrality — power-iteration port of `networkx.katz_centrality`.
//!
//! Nodes are interned to `0..n` indices; the inner loop operates on a
//! `Vec<Vec<usize>>` adjacency list for cache-friendly iteration.
//!
//! Algorithm: Leo Katz, "A New Status Index Derived from Sociometric Index",
//! Psychometrika 18(1):39–43, 1953.

use std::collections::HashMap;

/// Error returned when the iteration limit is reached without convergence.
#[derive(Debug)]
pub struct ConvergenceError {
    pub max_iter: usize,
}

impl std::fmt::Display for ConvergenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "power iteration did not converge in {} iterations",
            self.max_iter
        )
    }
}

impl std::error::Error for ConvergenceError {}

/// Parsed, deduplicated graph with string→index mapping.
pub struct Graph {
    /// Mapping from original string label to intern index.
    pub node_index: HashMap<String, usize>,
    /// Intern index → original label.
    pub node_labels: Vec<String>,
    /// Adjacency list indexed by intern index.
    pub adj: Vec<Vec<usize>>,
}

impl Graph {
    /// Build from an iterator of `(u, v)` string-label edge pairs.
    ///
    /// Preserves insertion order (first-seen wins), deduplicates parallel
    /// edges, and builds undirected adjacency exactly as `nx.Graph()` does.
    pub fn from_edges<I, S>(edges: I) -> Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<String>,
    {
        let mut node_index: HashMap<String, usize> = HashMap::new();
        let mut node_labels: Vec<String> = Vec::new();
        let mut edge_set: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();
        let mut pending: Vec<(usize, usize)> = Vec::new();

        let intern = |label: String,
                      index: &mut HashMap<String, usize>,
                      labels: &mut Vec<String>|
         -> usize {
            if let Some(&idx) = index.get(&label) {
                idx
            } else {
                let idx = labels.len();
                index.insert(label.clone(), idx);
                labels.push(label);
                idx
            }
        };

        for (u_s, v_s) in edges {
            let u_str: String = u_s.into();
            let v_str: String = v_s.into();
            // intern in the order they appear to match nx.Graph insertion order
            let u = intern(u_str, &mut node_index, &mut node_labels);
            let v = intern(v_str, &mut node_index, &mut node_labels);
            // deduplicate (nx.Graph silently ignores duplicate edges)
            let key = if u <= v { (u, v) } else { (v, u) };
            if edge_set.insert(key) {
                pending.push((u, v));
            }
        }

        let n = node_labels.len();
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (u, v) in pending {
            if u == v {
                // A self-loop is a single entry in nx.Graph's G[u], so the
                // power iteration adds the diagonal term x[u] exactly once.
                adj[u].push(u);
            } else {
                adj[u].push(v);
                adj[v].push(u);
            }
        }

        Self {
            node_index,
            node_labels,
            adj,
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.node_labels.len()
    }
}

/// Katz centrality parameters.
pub struct KatzParams {
    pub alpha: f64,
    pub beta: f64,
    pub max_iter: usize,
    pub tol: f64,
    pub normalized: bool,
}

impl Default for KatzParams {
    fn default() -> Self {
        Self {
            alpha: 0.1,
            beta: 1.0,
            max_iter: 1000,
            tol: 1e-6,
            normalized: true,
        }
    }
}

/// Compute Katz centrality via power iteration.
///
/// Returns `Vec<f64>` indexed by intern index on success.
///
/// Replicates `networkx.katz_centrality` exactly:
/// - x initialised to 0
/// - each iteration: xlast=x; x=0; for n: for nbr in adj[n]: x[nbr]+=xlast[n]; then x[n]=alpha*x[n]+beta
/// - convergence: sum(|x[n]-xlast[n]|) < N*tol
/// - normalisation: s=1/hypot(x[0],..,x[n-1]); x[n]*=s
pub fn katz_centrality(g: &Graph, p: &KatzParams) -> Result<Vec<f64>, ConvergenceError> {
    let n = g.num_nodes();
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut x = vec![0.0_f64; n];

    for _ in 0..p.max_iter {
        let xlast = x.clone();
        x.iter_mut().for_each(|v| *v = 0.0);

        // y^T = alpha * x^T * A + beta  (unweighted)
        for (node, xl) in xlast.iter().enumerate() {
            for &nbr in &g.adj[node] {
                x[nbr] += xl;
            }
        }
        for xv in x.iter_mut() {
            *xv = p.alpha * *xv + p.beta;
        }

        // convergence check: sum(|x[i] - xlast[i]|) < N * tol
        let error: f64 = x.iter().zip(xlast.iter()).map(|(a, b)| (a - b).abs()).sum();
        if error < (n as f64) * p.tol {
            if p.normalized {
                // s = 1.0 / hypot(x[0], ..., x[n-1])
                // std::f64::hypot is 2-arg only; replicate Python's math.hypot(*values)
                let sumsq: f64 = x.iter().map(|v| v * v).sum();
                let s = if sumsq == 0.0 {
                    1.0
                } else {
                    1.0 / sumsq.sqrt()
                };
                x.iter_mut().for_each(|v| *v *= s);
            }
            return Ok(x);
        }
    }

    Err(ConvergenceError {
        max_iter: p.max_iter,
    })
}

/// Parse an edge-list text (stdin), skipping `#` comment lines and blanks.
/// Returns pairs of string labels in order of first appearance.
pub fn parse_edge_list(input: &str) -> Vec<(String, String)> {
    input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.split_ascii_whitespace();
            let u = parts.next()?.to_string();
            let v = parts.next()?.to_string();
            Some((u, v))
        })
        .collect()
}
