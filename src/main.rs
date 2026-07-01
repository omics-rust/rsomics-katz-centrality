use clap::Parser;
use rsomics_katz_centrality::{Graph, KatzParams, katz_centrality, parse_edge_list};
use std::io::Read;

/// Katz centrality for biological networks.
///
/// Reads an undirected edge list from stdin (one edge per line: "u v"; "#"
/// comments and blank lines are skipped). Parallel edges are deduplicated.
/// Outputs node<TAB>value sorted lexicographically by node label.
///
/// Values are formatted as 17-significant-digit scientific notation with a
/// two-digit exponent (e.g. 3.21324596959232545e-01) to match Python's
/// repr(float) output.
///
/// Algorithm: power iteration matching networkx.katz_centrality (BSD-3-Clause).
/// Alpha must be < 1/largest_eigenvalue for convergence.
#[derive(Parser, Debug)]
#[command(name = "rsomics-katz-centrality", version)]
struct Cli {
    /// Attenuation factor (must be < 1/lambda_max of the adjacency matrix)
    #[arg(long, default_value_t = 0.1)]
    alpha: f64,

    /// Centrality attributed to immediate neighbours
    #[arg(long, default_value_t = 1.0)]
    beta: f64,

    /// Maximum power-iteration steps before error exit
    #[arg(long, default_value_t = 1000)]
    max_iter: usize,

    /// Convergence tolerance (sum of absolute changes < N*tol)
    #[arg(long, default_value_t = 1e-6)]
    tol: f64,

    /// Emit JSON object {"node": value, ...} instead of TSV
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read stdin");

    let edges = parse_edge_list(&input);
    let graph = Graph::from_edges(edges);

    let params = KatzParams {
        alpha: cli.alpha,
        beta: cli.beta,
        max_iter: cli.max_iter,
        tol: cli.tol,
        normalized: true,
    };

    let centrality = katz_centrality(&graph, &params).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    // Pair labels with values and sort lexicographically (output is deterministic).
    let mut rows: Vec<(&str, f64)> = graph
        .node_labels
        .iter()
        .enumerate()
        .map(|(i, label)| (label.as_str(), centrality[i]))
        .collect();
    rows.sort_unstable_by(|a, b| a.0.cmp(b.0));

    if cli.json {
        // Build a serde_json::Map preserving lex order.
        let map: serde_json::Map<String, serde_json::Value> = rows
            .into_iter()
            .map(|(label, v)| (label.to_string(), serde_json::Value::from(v)))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap()
        );
    } else {
        for (label, value) in rows {
            // Format matching Python's "%.17e" with two-digit exponent.
            println!("{}\t{}", label, fmt_f64(value));
        }
    }
}

/// Format f64 as 17-significant-digit scientific notation with two-digit exponent,
/// matching Python's `f"{v:.17e}"` output (e.g. `3.21324596959232545e-01`).
fn fmt_f64(v: f64) -> String {
    // Rust's {:e} produces a variable-length exponent (e.g. `e-1` not `e-01`).
    // We format with {:e} then normalise the exponent to two digits.
    let s = format!("{:.17e}", v);
    // Split at 'e'
    if let Some(pos) = s.find('e') {
        let mantissa = &s[..pos];
        let exp_str = &s[pos + 1..];
        let exp: i32 = exp_str.parse().unwrap_or(0);
        format!("{mantissa}e{exp:+03}")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn fmt_f64_two_digit_exponent() {
        let s = fmt_f64(3.21324596959232545e-01);
        assert!(s.contains("e-01"), "exponent should be two digits: {s}");
        let s2 = fmt_f64(1.0e+10);
        assert!(s2.contains("e+10"), "got: {s2}");
        let s3 = fmt_f64(1.0e-100);
        assert!(s3.contains("e-100"), "got: {s3}");
    }
}
