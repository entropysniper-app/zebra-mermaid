# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.52× for API return** and **1.62× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30438 | 0.31076 | 0.19946 | 1.53× | 1.56× |
| block | 6 | 0.09159 | 0.09688 | 0.07614 | 1.20× | 1.27× |
| c4 | 6 | 0.45871 | 0.46777 | 0.35138 | 1.31× | 1.33× |
| class | 6 | 0.23370 | 0.23973 | 0.16443 | 1.42× | 1.46× |
| er | 6 | 0.20692 | 0.21217 | 0.15836 | 1.31× | 1.34× |
| eventmodeling | 6 | 0.16538 | 0.20024 | 0.09345 | 1.77× | 2.14× |
| gantt | 6 | 0.23636 | 0.24341 | 0.15409 | 1.53× | 1.58× |
| git | 6 | 0.40374 | 0.41540 | 0.24337 | 1.66× | 1.71× |
| info | 2 | 0.00225 | 0.00278 | 0.00211 | 1.07× | 1.32× |
| journey | 1 | 0.01531 | 0.01675 | 0.01007 | 1.52× | 1.66× |
| kanban | 3 | 0.05140 | 0.05397 | 0.04230 | 1.22× | 1.28× |
| mindmap | 6 | 0.09155 | 0.10647 | 0.06266 | 1.46× | 1.70× |
| packet | 4 | 0.05947 | 0.06290 | 0.03459 | 1.72× | 1.82× |
| pie | 3 | 0.03292 | 0.03723 | 0.02170 | 1.52× | 1.72× |
| quadrant | 3 | 0.05222 | 0.05637 | 0.03665 | 1.42× | 1.54× |
| requirement | 6 | 0.22529 | 0.23321 | 0.16893 | 1.33× | 1.38× |
| sankey | 6 | 0.29530 | 0.34812 | 0.12282 | 2.40× | 2.83× |
| sequence | 6 | 0.13130 | 0.13725 | 0.09262 | 1.42× | 1.48× |
| state | 6 | 0.31111 | 0.31942 | 0.20668 | 1.51× | 1.55× |
| timeline | 6 | 0.19770 | 0.21425 | 0.11914 | 1.66× | 1.80× |
| xychart | 6 | 0.11308 | 0.11845 | 0.08862 | 1.28× | 1.34× |
| agentflow | 6 | 0.32013 | 0.32642 | 0.27846 | 1.15× | 1.17× |
| architecture | 6 | 0.21663 | 0.22364 | 0.09605 | 2.26× | 2.33× |
| cynefin | 3 | 0.09889 | 0.12889 | 0.06552 | 1.51× | 1.97× |
| ishikawa | 1 | 0.01796 | 0.01853 | 0.01196 | 1.50× | 1.55× |
| radar | 3 | 0.07295 | 0.07518 | 0.03453 | 2.11× | 2.18× |
| railroad | 6 | 0.20029 | 0.21009 | 0.16859 | 1.19× | 1.25× |
| swimlane | 6 | 0.28501 | 0.28753 | 0.21642 | 1.32× | 1.33× |
| treemap | 6 | 0.13832 | 0.14246 | 0.07293 | 1.90× | 1.95× |
| treeview | 6 | 0.09328 | 0.10212 | 0.04847 | 1.92× | 2.11× |
| usecase | 6 | 0.30018 | 0.30576 | 0.23081 | 1.30× | 1.32× |
| venn | 6 | 0.37987 | 0.38452 | 0.16786 | 2.26× | 2.29× |
| wardley | 6 | 0.14024 | 0.14814 | 0.09796 | 1.43× | 1.51× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-decimal-formatting/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-decimal-formatting/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-decimal-formatting/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
