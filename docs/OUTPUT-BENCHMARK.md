# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.78× for API return** and **1.90× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30376 | 0.32020 | 0.18767 | 1.62× | 1.71× |
| block | 6 | 0.09227 | 0.09580 | 0.07000 | 1.32× | 1.37× |
| c4 | 6 | 0.46222 | 0.47093 | 0.30385 | 1.52× | 1.55× |
| class | 6 | 0.23726 | 0.24047 | 0.15147 | 1.57× | 1.59× |
| er | 6 | 0.20768 | 0.21223 | 0.14202 | 1.46× | 1.49× |
| eventmodeling | 6 | 0.16505 | 0.20076 | 0.07462 | 2.21× | 2.69× |
| gantt | 6 | 0.23951 | 0.24465 | 0.13615 | 1.76× | 1.80× |
| git | 6 | 0.41027 | 0.41198 | 0.21659 | 1.89× | 1.90× |
| info | 2 | 0.00225 | 0.00277 | 0.00136 | 1.66× | 2.03× |
| journey | 1 | 0.01551 | 0.01646 | 0.00815 | 1.90× | 2.02× |
| kanban | 3 | 0.05339 | 0.05380 | 0.03749 | 1.42× | 1.44× |
| mindmap | 6 | 0.09240 | 0.10799 | 0.04505 | 2.05× | 2.40× |
| packet | 4 | 0.06014 | 0.06243 | 0.02974 | 2.02× | 2.10× |
| pie | 3 | 0.03286 | 0.03731 | 0.01820 | 1.81× | 2.05× |
| quadrant | 3 | 0.05261 | 0.05618 | 0.03214 | 1.64× | 1.75× |
| requirement | 6 | 0.22552 | 0.23003 | 0.14209 | 1.59× | 1.62× |
| sankey | 6 | 0.29152 | 0.34781 | 0.10472 | 2.78× | 3.32× |
| sequence | 6 | 0.13221 | 0.13846 | 0.07885 | 1.68× | 1.76× |
| state | 6 | 0.31782 | 0.33344 | 0.18943 | 1.68× | 1.76× |
| timeline | 6 | 0.19510 | 0.21300 | 0.08985 | 2.17× | 2.37× |
| xychart | 6 | 0.11318 | 0.11741 | 0.07131 | 1.59× | 1.65× |
| agentflow | 6 | 0.32005 | 0.32915 | 0.25831 | 1.24× | 1.27× |
| architecture | 6 | 0.21959 | 0.22648 | 0.08593 | 2.56× | 2.64× |
| cynefin | 3 | 0.09959 | 0.12833 | 0.05362 | 1.86× | 2.39× |
| ishikawa | 1 | 0.01832 | 0.01855 | 0.00958 | 1.91× | 1.94× |
| radar | 3 | 0.07391 | 0.07751 | 0.03018 | 2.45× | 2.57× |
| railroad | 6 | 0.20291 | 0.21007 | 0.14703 | 1.38× | 1.43× |
| swimlane | 6 | 0.28146 | 0.28738 | 0.19554 | 1.44× | 1.47× |
| treemap | 6 | 0.13944 | 0.14421 | 0.06427 | 2.17× | 2.24× |
| treeview | 6 | 0.09380 | 0.10502 | 0.04427 | 2.12× | 2.37× |
| usecase | 6 | 0.29414 | 0.29976 | 0.20981 | 1.40× | 1.43× |
| venn | 6 | 0.38250 | 0.39376 | 0.16593 | 2.31× | 2.37× |
| wardley | 6 | 0.14333 | 0.14835 | 0.08049 | 1.78× | 1.84× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-parser-ownership/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-parser-ownership/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-parser-ownership/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
