# rsomics-katz-centrality

Katz centrality for biological networks. Power-iteration port of
`networkx.katz_centrality`, value-exact to ≤1 ULP.

## Usage

```
echo "A B
A C
B C" | rsomics-katz-centrality --alpha 0.1
```

Output is `node<TAB>value` sorted lexicographically by node label, one line per
node. Values use 17-significant-digit scientific notation with a two-digit
exponent (e.g. `3.21324596959232545e-01`).

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--alpha` | `0.1` | Attenuation factor (must be < 1/λ_max) |
| `--beta` | `1.0` | Centrality attributed to immediate neighbours |
| `--max-iter` | `1000` | Maximum iteration count |
| `--tol` | `1e-6` | Convergence tolerance |
| `--json` | — | Emit JSON instead of TSV |

## Input format

Edge list on stdin, one edge per line (`u v`). `#` comment lines and blank
lines are ignored. Parallel edges are deduplicated. Node labels are arbitrary
strings; insertion order is preserved (first-seen wins), matching
`networkx.read_edgelist`.

## Performance

Integer-indexed adjacency (`Vec<Vec<usize>>`) eliminates hash-map overhead in
the inner loop. On a 5 000-node ring graph (k=4 neighbours/side), a single
power-iteration run converges ~20× faster than the pure-Python networkx path.

## Origin

This crate is an independent Rust reimplementation of `networkx.katz_centrality`
based on:

- Leo Katz, "A New Status Index Derived from Sociometric Index",
  Psychometrika 18(1):39–43, 1953. <https://doi.org/10.1007/BF02289026>
- The NetworkX source (BSD-3-Clause):
  `networkx/algorithms/centrality/katz.py` — MIT/Apache-2.0 reuse is
  permitted because NetworkX is BSD-3-Clause (not GPL). The algorithm is
  faithfully ported; no proprietary source was used.

License: MIT OR Apache-2.0.  
Upstream credit: [NetworkX](https://networkx.org/) (BSD-3-Clause).
