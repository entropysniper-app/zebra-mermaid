# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.48× for API return** and **1.57× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30476 | 0.31032 | 0.19700 | 1.55× | 1.58× |
| block | 6 | 0.09086 | 0.09461 | 0.08543 | 1.06× | 1.11× |
| c4 | 6 | 0.45347 | 0.47364 | 0.35532 | 1.28× | 1.33× |
| class | 6 | 0.23278 | 0.23810 | 0.16420 | 1.42× | 1.45× |
| er | 6 | 0.20506 | 0.21094 | 0.15927 | 1.29× | 1.32× |
| eventmodeling | 6 | 0.16134 | 0.19942 | 0.09365 | 1.72× | 2.13× |
| gantt | 6 | 0.23663 | 0.24095 | 0.15433 | 1.53× | 1.56× |
| git | 6 | 0.39992 | 0.41253 | 0.24045 | 1.66× | 1.72× |
| info | 2 | 0.00233 | 0.00276 | 0.00247 | 0.94× | 1.12× |
| journey | 1 | 0.01521 | 0.01611 | 0.01023 | 1.49× | 1.57× |
| kanban | 3 | 0.05117 | 0.05311 | 0.04164 | 1.23× | 1.28× |
| mindmap | 6 | 0.09155 | 0.10643 | 0.06304 | 1.45× | 1.69× |
| packet | 4 | 0.05950 | 0.06250 | 0.03483 | 1.71× | 1.79× |
| pie | 3 | 0.03250 | 0.03664 | 0.02242 | 1.45× | 1.63× |
| quadrant | 3 | 0.05169 | 0.05581 | 0.03857 | 1.34× | 1.45× |
| requirement | 6 | 0.22591 | 0.22910 | 0.17186 | 1.31× | 1.33× |
| sankey | 6 | 0.28948 | 0.34884 | 0.12816 | 2.26× | 2.72× |
| sequence | 6 | 0.12949 | 0.13467 | 0.10578 | 1.22× | 1.27× |
| state | 6 | 0.31349 | 0.32072 | 0.21228 | 1.48× | 1.51× |
| timeline | 6 | 0.19444 | 0.21509 | 0.12001 | 1.62× | 1.79× |
| xychart | 6 | 0.11255 | 0.11711 | 0.08855 | 1.27× | 1.32× |
| agentflow | 6 | 0.31763 | 0.32151 | 0.27817 | 1.14× | 1.16× |
| architecture | 6 | 0.21618 | 0.22042 | 0.09678 | 2.23× | 2.28× |
| cynefin | 3 | 0.09789 | 0.12643 | 0.06621 | 1.48× | 1.91× |
| ishikawa | 1 | 0.01771 | 0.01852 | 0.01208 | 1.47× | 1.53× |
| radar | 3 | 0.07168 | 0.07418 | 0.03449 | 2.08× | 2.15× |
| railroad | 6 | 0.19954 | 0.20787 | 0.16804 | 1.19× | 1.24× |
| swimlane | 6 | 0.28592 | 0.28455 | 0.21332 | 1.34× | 1.33× |
| treemap | 6 | 0.13821 | 0.14261 | 0.07919 | 1.75× | 1.80× |
| treeview | 6 | 0.09254 | 0.10155 | 0.04914 | 1.88× | 2.07× |
| usecase | 6 | 0.30059 | 0.30140 | 0.23433 | 1.28× | 1.29× |
| venn | 6 | 0.38524 | 0.39176 | 0.17032 | 2.26× | 2.30× |
| wardley | 6 | 0.14052 | 0.14547 | 0.09886 | 1.42× | 1.47× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-render-optimization/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-render-optimization/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-render-optimization/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
