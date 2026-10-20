# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.83× for API return** and **1.95× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30865 | 0.31255 | 0.18654 | 1.65× | 1.68× |
| block | 6 | 0.09272 | 0.09721 | 0.06933 | 1.34× | 1.40× |
| c4 | 6 | 0.46299 | 0.47684 | 0.29522 | 1.57× | 1.62× |
| class | 6 | 0.23235 | 0.23860 | 0.15178 | 1.53× | 1.57× |
| er | 6 | 0.20776 | 0.21379 | 0.14208 | 1.46× | 1.50× |
| eventmodeling | 6 | 0.16187 | 0.20058 | 0.07127 | 2.27× | 2.81× |
| gantt | 6 | 0.24027 | 0.24311 | 0.13444 | 1.79× | 1.81× |
| git | 6 | 0.40635 | 0.41579 | 0.20931 | 1.94× | 1.99× |
| info | 2 | 0.00224 | 0.00273 | 0.00125 | 1.79× | 2.19× |
| journey | 1 | 0.01592 | 0.01684 | 0.00779 | 2.04× | 2.16× |
| kanban | 3 | 0.05252 | 0.05383 | 0.03637 | 1.44× | 1.48× |
| mindmap | 6 | 0.09136 | 0.11010 | 0.04373 | 2.09× | 2.52× |
| packet | 4 | 0.06014 | 0.06276 | 0.02848 | 2.11× | 2.20× |
| pie | 3 | 0.03313 | 0.03851 | 0.01787 | 1.85× | 2.16× |
| quadrant | 3 | 0.05258 | 0.05642 | 0.03177 | 1.65× | 1.78× |
| requirement | 6 | 0.22575 | 0.23620 | 0.13540 | 1.67× | 1.74× |
| sankey | 6 | 0.29175 | 0.35219 | 0.10373 | 2.81× | 3.40× |
| sequence | 6 | 0.13236 | 0.13938 | 0.07782 | 1.70× | 1.79× |
| state | 6 | 0.31519 | 0.32749 | 0.18926 | 1.67× | 1.73× |
| timeline | 6 | 0.19763 | 0.21743 | 0.08601 | 2.30× | 2.53× |
| xychart | 6 | 0.12318 | 0.11871 | 0.07115 | 1.73× | 1.67× |
| agentflow | 6 | 0.32505 | 0.33065 | 0.25285 | 1.29× | 1.31× |
| architecture | 6 | 0.22130 | 0.22523 | 0.08467 | 2.61× | 2.66× |
| cynefin | 3 | 0.10065 | 0.12889 | 0.05144 | 1.96× | 2.51× |
| ishikawa | 1 | 0.01806 | 0.01884 | 0.00918 | 1.97× | 2.05× |
| radar | 3 | 0.07332 | 0.07547 | 0.02989 | 2.45× | 2.52× |
| railroad | 6 | 0.20101 | 0.21115 | 0.14943 | 1.35× | 1.41× |
| swimlane | 6 | 0.28242 | 0.29262 | 0.19694 | 1.43× | 1.49× |
| treemap | 6 | 0.14314 | 0.14217 | 0.06308 | 2.27× | 2.25× |
| treeview | 6 | 0.09356 | 0.10183 | 0.04327 | 2.16× | 2.35× |
| usecase | 6 | 0.30086 | 0.30225 | 0.20887 | 1.44× | 1.45× |
| venn | 6 | 0.38091 | 0.39103 | 0.16336 | 2.33× | 2.39× |
| wardley | 6 | 0.14336 | 0.15038 | 0.07864 | 1.82× | 1.91× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-label-scanning/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-label-scanning/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-label-scanning/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
