# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.67× for API return** and **1.76× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30539 | 0.31009 | 0.19496 | 1.57× | 1.59× |
| block | 6 | 0.09196 | 0.09558 | 0.07133 | 1.29× | 1.34× |
| c4 | 6 | 0.45603 | 0.47242 | 0.32246 | 1.41× | 1.47× |
| class | 6 | 0.23583 | 0.23623 | 0.15639 | 1.51× | 1.51× |
| er | 6 | 0.20706 | 0.21049 | 0.15234 | 1.36× | 1.38× |
| eventmodeling | 6 | 0.16037 | 0.20105 | 0.08238 | 1.95× | 2.44× |
| gantt | 6 | 0.23736 | 0.24061 | 0.14500 | 1.64× | 1.66× |
| git | 6 | 0.40656 | 0.41669 | 0.21396 | 1.90× | 1.95× |
| info | 2 | 0.00227 | 0.00272 | 0.00193 | 1.18× | 1.41× |
| journey | 1 | 0.01535 | 0.01626 | 0.00853 | 1.80× | 1.91× |
| kanban | 3 | 0.05218 | 0.05323 | 0.03837 | 1.36× | 1.39× |
| mindmap | 6 | 0.09432 | 0.10823 | 0.05240 | 1.80× | 2.07× |
| packet | 4 | 0.05975 | 0.06205 | 0.03147 | 1.90× | 1.97× |
| pie | 3 | 0.03325 | 0.03696 | 0.01974 | 1.68× | 1.87× |
| quadrant | 3 | 0.05398 | 0.05627 | 0.03443 | 1.57× | 1.63× |
| requirement | 6 | 0.22689 | 0.23353 | 0.14858 | 1.53× | 1.57× |
| sankey | 6 | 0.29132 | 0.34696 | 0.10430 | 2.79× | 3.33× |
| sequence | 6 | 0.13162 | 0.13674 | 0.09092 | 1.45× | 1.50× |
| state | 6 | 0.32122 | 0.32340 | 0.19423 | 1.65× | 1.67× |
| timeline | 6 | 0.19628 | 0.21264 | 0.10025 | 1.96× | 2.12× |
| xychart | 6 | 0.11408 | 0.11748 | 0.07890 | 1.45× | 1.49× |
| agentflow | 6 | 0.32156 | 0.32271 | 0.26673 | 1.21× | 1.21× |
| architecture | 6 | 0.21643 | 0.22412 | 0.08892 | 2.43× | 2.52× |
| cynefin | 3 | 0.09983 | 0.12799 | 0.05952 | 1.68× | 2.15× |
| ishikawa | 1 | 0.01794 | 0.01853 | 0.01072 | 1.67× | 1.73× |
| radar | 3 | 0.07313 | 0.07464 | 0.03198 | 2.29× | 2.33× |
| railroad | 6 | 0.20065 | 0.21298 | 0.15119 | 1.33× | 1.41× |
| swimlane | 6 | 0.28648 | 0.28623 | 0.20631 | 1.39× | 1.39× |
| treemap | 6 | 0.13929 | 0.14446 | 0.06541 | 2.13× | 2.21× |
| treeview | 6 | 0.09370 | 0.10208 | 0.04456 | 2.10× | 2.29× |
| usecase | 6 | 0.29663 | 0.29937 | 0.22254 | 1.33× | 1.35× |
| venn | 6 | 0.38340 | 0.39084 | 0.16507 | 2.32× | 2.37× |
| wardley | 6 | 0.14352 | 0.14791 | 0.09000 | 1.59× | 1.64× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-svg-concat/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-svg-concat/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-svg-concat/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
