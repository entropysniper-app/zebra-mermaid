# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.81× for API return** and **1.92× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30882 | 0.31538 | 0.18471 | 1.67× | 1.71× |
| block | 6 | 0.09213 | 0.09640 | 0.06969 | 1.32× | 1.38× |
| c4 | 6 | 0.45654 | 0.47125 | 0.29978 | 1.52× | 1.57× |
| class | 6 | 0.23758 | 0.24421 | 0.15327 | 1.55× | 1.59× |
| er | 6 | 0.20656 | 0.21461 | 0.14253 | 1.45× | 1.51× |
| eventmodeling | 6 | 0.16085 | 0.20143 | 0.07446 | 2.16× | 2.71× |
| gantt | 6 | 0.23681 | 0.24692 | 0.13507 | 1.75× | 1.83× |
| git | 6 | 0.40000 | 0.41587 | 0.21008 | 1.90× | 1.98× |
| info | 2 | 0.00232 | 0.00273 | 0.00124 | 1.87× | 2.21× |
| journey | 1 | 0.01572 | 0.01662 | 0.00801 | 1.96× | 2.08× |
| kanban | 3 | 0.05394 | 0.05394 | 0.03717 | 1.45× | 1.45× |
| mindmap | 6 | 0.09256 | 0.10879 | 0.04512 | 2.05× | 2.41× |
| packet | 4 | 0.06033 | 0.06255 | 0.02988 | 2.02× | 2.09× |
| pie | 3 | 0.03279 | 0.03871 | 0.01820 | 1.80× | 2.13× |
| quadrant | 3 | 0.05228 | 0.05627 | 0.03172 | 1.65× | 1.77× |
| requirement | 6 | 0.22891 | 0.24517 | 0.14064 | 1.63× | 1.74× |
| sankey | 6 | 0.29290 | 0.34697 | 0.10519 | 2.78× | 3.30× |
| sequence | 6 | 0.13829 | 0.13746 | 0.07843 | 1.76× | 1.75× |
| state | 6 | 0.31915 | 0.31865 | 0.18753 | 1.70× | 1.70× |
| timeline | 6 | 0.20062 | 0.21470 | 0.09089 | 2.21× | 2.36× |
| xychart | 6 | 0.11535 | 0.11846 | 0.07167 | 1.61× | 1.65× |
| agentflow | 6 | 0.31599 | 0.32716 | 0.25214 | 1.25× | 1.30× |
| architecture | 6 | 0.22290 | 0.22763 | 0.08718 | 2.56× | 2.61× |
| cynefin | 3 | 0.10034 | 0.12941 | 0.05292 | 1.90× | 2.45× |
| ishikawa | 1 | 0.01804 | 0.01882 | 0.00939 | 1.92× | 2.01× |
| radar | 3 | 0.07371 | 0.07567 | 0.02971 | 2.48× | 2.55× |
| railroad | 6 | 0.20084 | 0.21225 | 0.14928 | 1.35× | 1.42× |
| swimlane | 6 | 0.28700 | 0.28729 | 0.19483 | 1.47× | 1.47× |
| treemap | 6 | 0.14311 | 0.14320 | 0.06435 | 2.22× | 2.23× |
| treeview | 6 | 0.09621 | 0.10267 | 0.04433 | 2.17× | 2.32× |
| usecase | 6 | 0.29553 | 0.30820 | 0.20672 | 1.43× | 1.49× |
| venn | 6 | 0.38367 | 0.38865 | 0.16299 | 2.35× | 2.38× |
| wardley | 6 | 0.14390 | 0.15098 | 0.07846 | 1.83× | 1.92× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-shared-svg-costs/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-shared-svg-costs/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-shared-svg-costs/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
