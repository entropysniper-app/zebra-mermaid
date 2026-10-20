# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.52× for API return** and **1.61× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30317 | 0.30338 | 0.19907 | 1.52× | 1.52× |
| block | 6 | 0.09191 | 0.09591 | 0.07579 | 1.21× | 1.27× |
| c4 | 6 | 0.45973 | 0.46816 | 0.34576 | 1.33× | 1.35× |
| class | 6 | 0.23220 | 0.24015 | 0.16691 | 1.39× | 1.44× |
| er | 6 | 0.20958 | 0.21203 | 0.16010 | 1.31× | 1.32× |
| eventmodeling | 6 | 0.16205 | 0.20153 | 0.09298 | 1.74× | 2.17× |
| gantt | 6 | 0.23750 | 0.24691 | 0.15467 | 1.54× | 1.60× |
| git | 6 | 0.40522 | 0.41642 | 0.24178 | 1.68× | 1.72× |
| info | 2 | 0.00234 | 0.00273 | 0.00206 | 1.13× | 1.33× |
| journey | 1 | 0.01568 | 0.01689 | 0.01023 | 1.53× | 1.65× |
| kanban | 3 | 0.05210 | 0.05353 | 0.04252 | 1.23× | 1.26× |
| mindmap | 6 | 0.09306 | 0.10772 | 0.06290 | 1.48× | 1.71× |
| packet | 4 | 0.06031 | 0.06165 | 0.03502 | 1.72× | 1.76× |
| pie | 3 | 0.03411 | 0.03744 | 0.02177 | 1.57× | 1.72× |
| quadrant | 3 | 0.05293 | 0.05615 | 0.03660 | 1.45× | 1.53× |
| requirement | 6 | 0.22624 | 0.23126 | 0.17139 | 1.32× | 1.35× |
| sankey | 6 | 0.29334 | 0.34982 | 0.12361 | 2.37× | 2.83× |
| sequence | 6 | 0.13218 | 0.13690 | 0.09287 | 1.42× | 1.47× |
| state | 6 | 0.31787 | 0.32224 | 0.20825 | 1.53× | 1.55× |
| timeline | 6 | 0.19658 | 0.21301 | 0.12070 | 1.63× | 1.76× |
| xychart | 6 | 0.11434 | 0.11813 | 0.08995 | 1.27× | 1.31× |
| agentflow | 6 | 0.32259 | 0.32625 | 0.28202 | 1.14× | 1.16× |
| architecture | 6 | 0.21747 | 0.22466 | 0.09648 | 2.25× | 2.33× |
| cynefin | 3 | 0.09940 | 0.12859 | 0.06624 | 1.50× | 1.94× |
| ishikawa | 1 | 0.01813 | 0.01856 | 0.01202 | 1.51× | 1.55× |
| radar | 3 | 0.07279 | 0.07518 | 0.03432 | 2.12× | 2.19× |
| railroad | 6 | 0.20193 | 0.20903 | 0.16957 | 1.19× | 1.23× |
| swimlane | 6 | 0.28709 | 0.28397 | 0.21424 | 1.34× | 1.33× |
| treemap | 6 | 0.13837 | 0.14208 | 0.07387 | 1.87× | 1.92× |
| treeview | 6 | 0.09420 | 0.10201 | 0.04904 | 1.92× | 2.08× |
| usecase | 6 | 0.29895 | 0.30209 | 0.23255 | 1.29× | 1.30× |
| venn | 6 | 0.38342 | 0.38847 | 0.16894 | 2.27× | 2.30× |
| wardley | 6 | 0.14340 | 0.14760 | 0.09855 | 1.46× | 1.50× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-literal-label/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-literal-label/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-literal-label/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
