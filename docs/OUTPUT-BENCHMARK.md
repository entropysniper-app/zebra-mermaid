# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.89× for API return** and **1.98× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.31014 | 0.31736 | 0.18478 | 1.68× | 1.72× |
| block | 6 | 0.09673 | 0.09982 | 0.06918 | 1.40× | 1.44× |
| c4 | 6 | 0.46618 | 0.48133 | 0.29469 | 1.58× | 1.63× |
| class | 6 | 0.25733 | 0.25002 | 0.15061 | 1.71× | 1.66× |
| er | 6 | 0.20726 | 0.21361 | 0.14097 | 1.47× | 1.52× |
| eventmodeling | 6 | 0.16373 | 0.20708 | 0.07020 | 2.33× | 2.95× |
| gantt | 6 | 0.24102 | 0.24596 | 0.13591 | 1.77× | 1.81× |
| git | 6 | 0.40927 | 0.42145 | 0.20755 | 1.97× | 2.03× |
| info | 2 | 0.00232 | 0.00281 | 0.00112 | 2.06× | 2.50× |
| journey | 1 | 0.01609 | 0.01651 | 0.00782 | 2.06× | 2.11× |
| kanban | 3 | 0.05221 | 0.05405 | 0.03631 | 1.44× | 1.49× |
| mindmap | 6 | 0.09454 | 0.10973 | 0.04388 | 2.15× | 2.50× |
| packet | 4 | 0.06150 | 0.06307 | 0.02693 | 2.28× | 2.34× |
| pie | 3 | 0.03390 | 0.03798 | 0.01791 | 1.89× | 2.12× |
| quadrant | 3 | 0.05317 | 0.05698 | 0.03169 | 1.68× | 1.80× |
| requirement | 6 | 0.22787 | 0.23412 | 0.13654 | 1.67× | 1.71× |
| sankey | 6 | 0.31028 | 0.35284 | 0.10351 | 3.00× | 3.41× |
| sequence | 6 | 0.13705 | 0.14257 | 0.07759 | 1.77× | 1.84× |
| state | 6 | 0.33984 | 0.32642 | 0.18995 | 1.79× | 1.72× |
| timeline | 6 | 0.20243 | 0.21687 | 0.08747 | 2.31× | 2.48× |
| xychart | 6 | 0.11476 | 0.11984 | 0.06981 | 1.64× | 1.72× |
| agentflow | 6 | 0.33136 | 0.33237 | 0.25594 | 1.29× | 1.30× |
| architecture | 6 | 0.22394 | 0.22713 | 0.08423 | 2.66× | 2.70× |
| cynefin | 3 | 0.10104 | 0.13111 | 0.05092 | 1.98× | 2.57× |
| ishikawa | 1 | 0.01826 | 0.01888 | 0.00918 | 1.99× | 2.06× |
| radar | 3 | 0.07418 | 0.07607 | 0.02895 | 2.56× | 2.63× |
| railroad | 6 | 0.21032 | 0.21328 | 0.14575 | 1.44× | 1.46× |
| swimlane | 6 | 0.28746 | 0.29536 | 0.19406 | 1.48× | 1.52× |
| treemap | 6 | 0.15142 | 0.14361 | 0.06150 | 2.46× | 2.34× |
| treeview | 6 | 0.09561 | 0.10458 | 0.04269 | 2.24× | 2.45× |
| usecase | 6 | 0.31354 | 0.30492 | 0.20610 | 1.52× | 1.48× |
| venn | 6 | 0.40040 | 0.40077 | 0.16613 | 2.41× | 2.41× |
| wardley | 6 | 0.14504 | 0.15290 | 0.07745 | 1.87× | 1.97× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-21-lazy-token-state/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-21-lazy-token-state/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-21-lazy-token-state/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
