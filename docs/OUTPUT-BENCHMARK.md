# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.73× for API return** and **1.83× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.31649 | 0.32132 | 0.19534 | 1.62× | 1.64× |
| block | 6 | 0.10607 | 0.09791 | 0.07097 | 1.49× | 1.38× |
| c4 | 6 | 0.47977 | 0.48046 | 0.32210 | 1.49× | 1.49× |
| class | 6 | 0.23503 | 0.24080 | 0.15717 | 1.50× | 1.53× |
| er | 6 | 0.21020 | 0.21658 | 0.14911 | 1.41× | 1.45× |
| eventmodeling | 6 | 0.16592 | 0.20319 | 0.07984 | 2.08× | 2.54× |
| gantt | 6 | 0.24179 | 0.24404 | 0.14483 | 1.67× | 1.68× |
| git | 6 | 0.40927 | 0.42006 | 0.21465 | 1.91× | 1.96× |
| info | 2 | 0.00232 | 0.00281 | 0.00168 | 1.38× | 1.67× |
| journey | 1 | 0.01572 | 0.01688 | 0.00834 | 1.88× | 2.02× |
| kanban | 3 | 0.05279 | 0.05483 | 0.03799 | 1.39× | 1.44× |
| mindmap | 6 | 0.09402 | 0.11052 | 0.05143 | 1.83× | 2.15× |
| packet | 4 | 0.06052 | 0.06346 | 0.03045 | 1.99× | 2.08× |
| pie | 3 | 0.03356 | 0.03926 | 0.01873 | 1.79× | 2.10× |
| quadrant | 3 | 0.05414 | 0.05719 | 0.03375 | 1.60× | 1.69× |
| requirement | 6 | 0.23307 | 0.23562 | 0.14549 | 1.60× | 1.62× |
| sankey | 6 | 0.30260 | 0.35287 | 0.10567 | 2.86× | 3.34× |
| sequence | 6 | 0.13854 | 0.14028 | 0.09129 | 1.52× | 1.54× |
| state | 6 | 0.32884 | 0.32464 | 0.19378 | 1.70× | 1.68× |
| timeline | 6 | 0.19980 | 0.21673 | 0.09979 | 2.00× | 2.17× |
| xychart | 6 | 0.11930 | 0.12281 | 0.07444 | 1.60× | 1.65× |
| agentflow | 6 | 0.32617 | 0.34713 | 0.26939 | 1.21× | 1.29× |
| architecture | 6 | 0.22005 | 0.22585 | 0.08843 | 2.49× | 2.55× |
| cynefin | 3 | 0.10102 | 0.13148 | 0.05843 | 1.73× | 2.25× |
| ishikawa | 1 | 0.01826 | 0.01906 | 0.01050 | 1.74× | 1.82× |
| radar | 3 | 0.07310 | 0.07637 | 0.03104 | 2.36× | 2.46× |
| railroad | 6 | 0.20668 | 0.21207 | 0.15179 | 1.36× | 1.40× |
| swimlane | 6 | 0.28353 | 0.29093 | 0.20844 | 1.36× | 1.40× |
| treemap | 6 | 0.13917 | 0.14685 | 0.06755 | 2.06× | 2.17× |
| treeview | 6 | 0.09470 | 0.10300 | 0.04498 | 2.11× | 2.29× |
| usecase | 6 | 0.30029 | 0.32016 | 0.22449 | 1.34× | 1.43× |
| venn | 6 | 0.39429 | 0.38781 | 0.16900 | 2.33× | 2.29× |
| wardley | 6 | 0.14399 | 0.15072 | 0.08485 | 1.70× | 1.78× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-compact-token/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-compact-token/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-compact-token/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
