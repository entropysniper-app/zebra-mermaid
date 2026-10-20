# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.79× for API return** and **1.90× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30534 | 0.31755 | 0.18926 | 1.61× | 1.68× |
| block | 6 | 0.09209 | 0.09636 | 0.06981 | 1.32× | 1.38× |
| c4 | 6 | 0.45437 | 0.46958 | 0.29987 | 1.52× | 1.57× |
| class | 6 | 0.23095 | 0.23986 | 0.15422 | 1.50× | 1.56× |
| er | 6 | 0.20958 | 0.21132 | 0.14233 | 1.47× | 1.48× |
| eventmodeling | 6 | 0.16232 | 0.20107 | 0.07503 | 2.16× | 2.68× |
| gantt | 6 | 0.24296 | 0.24288 | 0.13588 | 1.79× | 1.79× |
| git | 6 | 0.42811 | 0.41461 | 0.21158 | 2.02× | 1.96× |
| info | 2 | 0.00226 | 0.00281 | 0.00135 | 1.68× | 2.08× |
| journey | 1 | 0.01587 | 0.01655 | 0.00803 | 1.98× | 2.06× |
| kanban | 3 | 0.05171 | 0.05403 | 0.03786 | 1.37× | 1.43× |
| mindmap | 6 | 0.09330 | 0.11184 | 0.04527 | 2.06× | 2.47× |
| packet | 4 | 0.06046 | 0.06275 | 0.02980 | 2.03× | 2.11× |
| pie | 3 | 0.03382 | 0.03803 | 0.01828 | 1.85× | 2.08× |
| quadrant | 3 | 0.05364 | 0.05581 | 0.03213 | 1.67× | 1.74× |
| requirement | 6 | 0.22788 | 0.23303 | 0.14093 | 1.62× | 1.65× |
| sankey | 6 | 0.29273 | 0.35326 | 0.10598 | 2.76× | 3.33× |
| sequence | 6 | 0.13290 | 0.13660 | 0.08063 | 1.65× | 1.69× |
| state | 6 | 0.31906 | 0.32504 | 0.19086 | 1.67× | 1.70× |
| timeline | 6 | 0.20099 | 0.21348 | 0.09026 | 2.23× | 2.37× |
| xychart | 6 | 0.11656 | 0.11952 | 0.07219 | 1.61× | 1.66× |
| agentflow | 6 | 0.31836 | 0.32611 | 0.25403 | 1.25× | 1.28× |
| architecture | 6 | 0.21688 | 0.22420 | 0.08529 | 2.54× | 2.63× |
| cynefin | 3 | 0.09995 | 0.12980 | 0.05470 | 1.83× | 2.37× |
| ishikawa | 1 | 0.01787 | 0.01850 | 0.00949 | 1.88× | 1.95× |
| radar | 3 | 0.07296 | 0.07525 | 0.02954 | 2.47× | 2.55× |
| railroad | 6 | 0.20375 | 0.20878 | 0.14798 | 1.38× | 1.41× |
| swimlane | 6 | 0.28509 | 0.28438 | 0.19625 | 1.45× | 1.45× |
| treemap | 6 | 0.13899 | 0.14249 | 0.06551 | 2.12× | 2.18× |
| treeview | 6 | 0.09392 | 0.10202 | 0.04459 | 2.11× | 2.29× |
| usecase | 6 | 0.29363 | 0.30456 | 0.21056 | 1.39× | 1.45× |
| venn | 6 | 0.38196 | 0.38966 | 0.16423 | 2.33× | 2.37× |
| wardley | 6 | 0.14359 | 0.14737 | 0.07923 | 1.81× | 1.86× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-borrowed-entry/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-borrowed-entry/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-borrowed-entry/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
