# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.79× for API return** and **1.89× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.33179 | 0.33167 | 0.20275 | 1.64× | 1.64× |
| block | 6 | 0.09524 | 0.09816 | 0.07073 | 1.35× | 1.39× |
| c4 | 6 | 0.47725 | 0.49972 | 0.32024 | 1.49× | 1.56× |
| class | 6 | 0.24422 | 0.25655 | 0.15404 | 1.59× | 1.67× |
| er | 6 | 0.24819 | 0.23162 | 0.14828 | 1.67× | 1.56× |
| eventmodeling | 6 | 0.19596 | 0.20676 | 0.08092 | 2.42× | 2.56× |
| gantt | 6 | 0.25447 | 0.25253 | 0.14459 | 1.76× | 1.75× |
| git | 6 | 0.41103 | 0.43081 | 0.21574 | 1.91× | 2.00× |
| info | 2 | 0.00230 | 0.00279 | 0.00172 | 1.34× | 1.63× |
| journey | 1 | 0.01585 | 0.01710 | 0.00835 | 1.90× | 2.05× |
| kanban | 3 | 0.05298 | 0.05455 | 0.03756 | 1.41× | 1.45× |
| mindmap | 6 | 0.09852 | 0.11329 | 0.05174 | 1.90× | 2.19× |
| packet | 4 | 0.06235 | 0.06322 | 0.03134 | 1.99× | 2.02× |
| pie | 3 | 0.03537 | 0.03834 | 0.02024 | 1.75× | 1.89× |
| quadrant | 3 | 0.05425 | 0.05841 | 0.03311 | 1.64× | 1.76× |
| requirement | 6 | 0.24260 | 0.27564 | 0.14540 | 1.67× | 1.90× |
| sankey | 6 | 0.30093 | 0.37883 | 0.10539 | 2.86× | 3.59× |
| sequence | 6 | 0.14151 | 0.14154 | 0.08605 | 1.64× | 1.64× |
| state | 6 | 0.43755 | 0.32714 | 0.20509 | 2.13× | 1.60× |
| timeline | 6 | 0.20397 | 0.21849 | 0.09968 | 2.05× | 2.19× |
| xychart | 6 | 0.11738 | 0.14085 | 0.07278 | 1.61× | 1.94× |
| agentflow | 6 | 0.33035 | 0.44012 | 0.27328 | 1.21× | 1.61× |
| architecture | 6 | 0.22588 | 0.23733 | 0.08770 | 2.58× | 2.71× |
| cynefin | 3 | 0.10310 | 0.13004 | 0.05774 | 1.79× | 2.25× |
| ishikawa | 1 | 0.01937 | 0.01948 | 0.00967 | 2.00× | 2.01× |
| radar | 3 | 0.07420 | 0.07674 | 0.03173 | 2.34× | 2.42× |
| railroad | 6 | 0.21380 | 0.21254 | 0.15426 | 1.39× | 1.38× |
| swimlane | 6 | 0.30762 | 0.29009 | 0.21175 | 1.45× | 1.37× |
| treemap | 6 | 0.14282 | 0.15760 | 0.06704 | 2.13× | 2.35× |
| treeview | 6 | 0.09461 | 0.10918 | 0.04572 | 2.07× | 2.39× |
| usecase | 6 | 0.32625 | 0.31231 | 0.22677 | 1.44× | 1.38× |
| venn | 6 | 0.41589 | 0.39869 | 0.16624 | 2.50× | 2.40× |
| wardley | 6 | 0.15755 | 0.15152 | 0.08467 | 1.86× | 1.79× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-svg-adapters/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-svg-adapters/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-svg-adapters/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
