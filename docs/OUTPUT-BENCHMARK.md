# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.50× for API return** and **1.60× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30440 | 0.34053 | 0.19891 | 1.53× | 1.71× |
| block | 6 | 0.09241 | 0.09693 | 0.07711 | 1.20× | 1.26× |
| c4 | 6 | 0.46360 | 0.47554 | 0.35177 | 1.32× | 1.35× |
| class | 6 | 0.23289 | 0.23801 | 0.16480 | 1.41× | 1.44× |
| er | 6 | 0.20814 | 0.21514 | 0.15922 | 1.31× | 1.35× |
| eventmodeling | 6 | 0.16189 | 0.20188 | 0.09491 | 1.71× | 2.13× |
| gantt | 6 | 0.23703 | 0.24221 | 0.15486 | 1.53× | 1.56× |
| git | 6 | 0.40870 | 0.41734 | 0.24061 | 1.70× | 1.73× |
| info | 2 | 0.00233 | 0.00276 | 0.00247 | 0.95× | 1.12× |
| journey | 1 | 0.01558 | 0.01649 | 0.01022 | 1.52× | 1.61× |
| kanban | 3 | 0.05316 | 0.05370 | 0.04193 | 1.27× | 1.28× |
| mindmap | 6 | 0.09146 | 0.10754 | 0.06434 | 1.42× | 1.67× |
| packet | 4 | 0.05993 | 0.06254 | 0.03531 | 1.70× | 1.77× |
| pie | 3 | 0.03299 | 0.03735 | 0.02248 | 1.47× | 1.66× |
| quadrant | 3 | 0.05249 | 0.05703 | 0.03699 | 1.42× | 1.54× |
| requirement | 6 | 0.22552 | 0.23103 | 0.16968 | 1.33× | 1.36× |
| sankey | 6 | 0.29147 | 0.34899 | 0.12753 | 2.29× | 2.74× |
| sequence | 6 | 0.13112 | 0.14055 | 0.09261 | 1.42× | 1.52× |
| state | 6 | 0.31334 | 0.32123 | 0.20988 | 1.49× | 1.53× |
| timeline | 6 | 0.19547 | 0.21300 | 0.12250 | 1.60× | 1.74× |
| xychart | 6 | 0.11437 | 0.11838 | 0.08926 | 1.28× | 1.33× |
| agentflow | 6 | 0.32396 | 0.32551 | 0.27773 | 1.17× | 1.17× |
| architecture | 6 | 0.21766 | 0.22159 | 0.09661 | 2.25× | 2.29× |
| cynefin | 3 | 0.09934 | 0.12787 | 0.06664 | 1.49× | 1.92× |
| ishikawa | 1 | 0.01776 | 0.01867 | 0.01228 | 1.45× | 1.52× |
| radar | 3 | 0.07313 | 0.07537 | 0.03482 | 2.10× | 2.16× |
| railroad | 6 | 0.20024 | 0.20983 | 0.17051 | 1.17× | 1.23× |
| swimlane | 6 | 0.28707 | 0.28860 | 0.21497 | 1.34× | 1.34× |
| treemap | 6 | 0.13873 | 0.14249 | 0.07662 | 1.81× | 1.86× |
| treeview | 6 | 0.09322 | 0.10234 | 0.04901 | 1.90× | 2.09× |
| usecase | 6 | 0.29383 | 0.32120 | 0.23113 | 1.27× | 1.39× |
| venn | 6 | 0.38407 | 0.39210 | 0.17031 | 2.26× | 2.30× |
| wardley | 6 | 0.14092 | 0.14792 | 0.09908 | 1.42× | 1.49× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-render-optimization-round2/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-render-optimization-round2/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-render-optimization-round2/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
