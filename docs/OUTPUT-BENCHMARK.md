# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.53× for API return** and **1.62× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.30788 | 0.30905 | 0.20004 | 1.54× | 1.54× |
| block | 6 | 0.09209 | 0.09583 | 0.07608 | 1.21× | 1.26× |
| c4 | 6 | 0.46397 | 0.46924 | 0.35146 | 1.32× | 1.34× |
| class | 6 | 0.23333 | 0.23872 | 0.16603 | 1.41× | 1.44× |
| er | 6 | 0.20701 | 0.21059 | 0.16044 | 1.29× | 1.31× |
| eventmodeling | 6 | 0.16227 | 0.20122 | 0.09298 | 1.75× | 2.16× |
| gantt | 6 | 0.23721 | 0.24507 | 0.15320 | 1.55× | 1.60× |
| git | 6 | 0.40712 | 0.41495 | 0.23968 | 1.70× | 1.73× |
| info | 2 | 0.00230 | 0.00275 | 0.00211 | 1.09× | 1.30× |
| journey | 1 | 0.01551 | 0.01650 | 0.00998 | 1.55× | 1.65× |
| kanban | 3 | 0.05183 | 0.05345 | 0.04153 | 1.25× | 1.29× |
| mindmap | 6 | 0.09232 | 0.10763 | 0.06182 | 1.49× | 1.74× |
| packet | 4 | 0.06129 | 0.06202 | 0.03431 | 1.79× | 1.81× |
| pie | 3 | 0.03285 | 0.03734 | 0.02173 | 1.51× | 1.72× |
| quadrant | 3 | 0.05247 | 0.05652 | 0.03606 | 1.46× | 1.57× |
| requirement | 6 | 0.22751 | 0.23496 | 0.16949 | 1.34× | 1.39× |
| sankey | 6 | 0.29449 | 0.34786 | 0.12777 | 2.30× | 2.72× |
| sequence | 6 | 0.13158 | 0.13638 | 0.09212 | 1.43× | 1.48× |
| state | 6 | 0.31706 | 0.32882 | 0.20674 | 1.53× | 1.59× |
| timeline | 6 | 0.19822 | 0.21238 | 0.11954 | 1.66× | 1.78× |
| xychart | 6 | 0.11421 | 0.11770 | 0.08875 | 1.29× | 1.33× |
| agentflow | 6 | 0.31686 | 0.32679 | 0.27959 | 1.13× | 1.17× |
| architecture | 6 | 0.21889 | 0.22351 | 0.09607 | 2.28× | 2.33× |
| cynefin | 3 | 0.09951 | 0.12834 | 0.06513 | 1.53× | 1.97× |
| ishikawa | 1 | 0.01809 | 0.01846 | 0.01197 | 1.51× | 1.54× |
| radar | 3 | 0.07285 | 0.07482 | 0.03397 | 2.14× | 2.20× |
| railroad | 6 | 0.20094 | 0.20818 | 0.16669 | 1.21× | 1.25× |
| swimlane | 6 | 0.28342 | 0.29117 | 0.21329 | 1.33× | 1.37× |
| treemap | 6 | 0.13843 | 0.14222 | 0.07563 | 1.83× | 1.88× |
| treeview | 6 | 0.09354 | 0.10201 | 0.04835 | 1.93× | 2.11× |
| usecase | 6 | 0.29713 | 0.30105 | 0.23101 | 1.29× | 1.30× |
| venn | 6 | 0.38288 | 0.38880 | 0.16960 | 2.26× | 2.29× |
| wardley | 6 | 0.14242 | 0.14706 | 0.09816 | 1.45× | 1.50× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-shared-pipeline/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-shared-pipeline/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-shared-pipeline/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
