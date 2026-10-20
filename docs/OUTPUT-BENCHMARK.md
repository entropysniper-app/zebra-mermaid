# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.85× for API return** and **1.97× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30785 | 0.30754 | 0.18430 | 1.67× | 1.67× |
| block | 6 | 0.09265 | 0.09636 | 0.06891 | 1.34× | 1.40× |
| c4 | 6 | 0.45326 | 0.46945 | 0.29279 | 1.55× | 1.60× |
| class | 6 | 0.23232 | 0.24128 | 0.14978 | 1.55× | 1.61× |
| er | 6 | 0.20648 | 0.21418 | 0.14014 | 1.47× | 1.53× |
| eventmodeling | 6 | 0.16175 | 0.20307 | 0.07035 | 2.30× | 2.89× |
| gantt | 6 | 0.23951 | 0.24185 | 0.13349 | 1.79× | 1.81× |
| git | 6 | 0.41454 | 0.41498 | 0.20604 | 2.01× | 2.01× |
| info | 2 | 0.00227 | 0.00277 | 0.00117 | 1.94× | 2.37× |
| journey | 1 | 0.01560 | 0.01660 | 0.00773 | 2.02× | 2.15× |
| kanban | 3 | 0.05320 | 0.05460 | 0.03561 | 1.49× | 1.53× |
| mindmap | 6 | 0.09212 | 0.10770 | 0.04323 | 2.13× | 2.49× |
| packet | 4 | 0.06072 | 0.06250 | 0.02695 | 2.25× | 2.32× |
| pie | 3 | 0.03297 | 0.03691 | 0.01751 | 1.88× | 2.11× |
| quadrant | 3 | 0.05201 | 0.05804 | 0.03130 | 1.66× | 1.85× |
| requirement | 6 | 0.22616 | 0.23342 | 0.13595 | 1.66× | 1.72× |
| sankey | 6 | 0.29199 | 0.34731 | 0.10300 | 2.83× | 3.37× |
| sequence | 6 | 0.13174 | 0.13722 | 0.07640 | 1.72× | 1.80× |
| state | 6 | 0.32212 | 0.31923 | 0.18777 | 1.72× | 1.70× |
| timeline | 6 | 0.19507 | 0.21325 | 0.08486 | 2.30× | 2.51× |
| xychart | 6 | 0.11494 | 0.11752 | 0.06804 | 1.69× | 1.73× |
| agentflow | 6 | 0.31209 | 0.33486 | 0.25337 | 1.23× | 1.32× |
| architecture | 6 | 0.21816 | 0.22132 | 0.08314 | 2.62× | 2.66× |
| cynefin | 3 | 0.10193 | 0.12913 | 0.05039 | 2.02× | 2.56× |
| ishikawa | 1 | 0.01895 | 0.01888 | 0.00909 | 2.08× | 2.08× |
| radar | 3 | 0.07297 | 0.07510 | 0.02900 | 2.52× | 2.59× |
| railroad | 6 | 0.20631 | 0.21071 | 0.14485 | 1.42× | 1.45× |
| swimlane | 6 | 0.28937 | 0.28473 | 0.19512 | 1.48× | 1.46× |
| treemap | 6 | 0.13957 | 0.14398 | 0.06474 | 2.16× | 2.22× |
| treeview | 6 | 0.09334 | 0.10343 | 0.04272 | 2.18× | 2.42× |
| usecase | 6 | 0.29482 | 0.30010 | 0.20326 | 1.45× | 1.48× |
| venn | 6 | 0.38714 | 0.38982 | 0.16247 | 2.38× | 2.40× |
| wardley | 6 | 0.14102 | 0.15064 | 0.07629 | 1.85× | 1.97× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-number-text/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-number-text/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-number-text/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
