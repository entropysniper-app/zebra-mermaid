# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.77× for API return** and **1.87× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.31210 | 0.30832 | 0.19208 | 1.62× | 1.61× |
| block | 6 | 0.09462 | 0.09720 | 0.06979 | 1.36× | 1.39× |
| c4 | 6 | 0.45995 | 0.46623 | 0.29691 | 1.55× | 1.57× |
| class | 6 | 0.23282 | 0.24172 | 0.15074 | 1.54× | 1.60× |
| er | 6 | 0.20963 | 0.21151 | 0.14283 | 1.47× | 1.48× |
| eventmodeling | 6 | 0.16217 | 0.20139 | 0.07558 | 2.15× | 2.66× |
| gantt | 6 | 0.24045 | 0.24871 | 0.13427 | 1.79× | 1.85× |
| git | 6 | 0.40574 | 0.41267 | 0.21723 | 1.87× | 1.90× |
| info | 2 | 0.00233 | 0.00272 | 0.00134 | 1.74× | 2.02× |
| journey | 1 | 0.01559 | 0.01660 | 0.00825 | 1.89× | 2.01× |
| kanban | 3 | 0.05239 | 0.05356 | 0.03688 | 1.42× | 1.45× |
| mindmap | 6 | 0.09170 | 0.10965 | 0.04500 | 2.04× | 2.44× |
| packet | 4 | 0.06035 | 0.06225 | 0.03012 | 2.00× | 2.07× |
| pie | 3 | 0.03292 | 0.03739 | 0.01822 | 1.81× | 2.05× |
| quadrant | 3 | 0.05294 | 0.05599 | 0.03253 | 1.63× | 1.72× |
| requirement | 6 | 0.22539 | 0.23114 | 0.14175 | 1.59× | 1.63× |
| sankey | 6 | 0.30772 | 0.34877 | 0.10596 | 2.90× | 3.29× |
| sequence | 6 | 0.13142 | 0.13678 | 0.07936 | 1.66× | 1.72× |
| state | 6 | 0.31591 | 0.33167 | 0.19203 | 1.65× | 1.73× |
| timeline | 6 | 0.19631 | 0.21223 | 0.09186 | 2.14× | 2.31× |
| xychart | 6 | 0.11384 | 0.11928 | 0.07164 | 1.59× | 1.66× |
| agentflow | 6 | 0.32566 | 0.32479 | 0.26648 | 1.22× | 1.22× |
| architecture | 6 | 0.21664 | 0.22025 | 0.08845 | 2.45× | 2.49× |
| cynefin | 3 | 0.09894 | 0.12748 | 0.05315 | 1.86× | 2.40× |
| ishikawa | 1 | 0.01785 | 0.01858 | 0.00928 | 1.92× | 2.00× |
| radar | 3 | 0.07263 | 0.07460 | 0.03101 | 2.34× | 2.41× |
| railroad | 6 | 0.19992 | 0.20874 | 0.14921 | 1.34× | 1.40× |
| swimlane | 6 | 0.28780 | 0.29025 | 0.20724 | 1.39× | 1.40× |
| treemap | 6 | 0.13750 | 0.14181 | 0.06517 | 2.11× | 2.18× |
| treeview | 6 | 0.09354 | 0.10219 | 0.04461 | 2.10× | 2.29× |
| usecase | 6 | 0.30576 | 0.29738 | 0.21869 | 1.40× | 1.36× |
| venn | 6 | 0.38665 | 0.38385 | 0.16608 | 2.33× | 2.31× |
| wardley | 6 | 0.14119 | 0.14747 | 0.07983 | 1.77× | 1.85× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-label-temporaries/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-label-temporaries/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-label-temporaries/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
