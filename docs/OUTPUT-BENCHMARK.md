# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.7.0, commit `8603bf3313add9143cd417015d4a6bd6c522dada`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.74× for API return** and **1.84× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.32505 | 0.34330 | 0.19714 | 1.65× | 1.74× |
| block | 6 | 0.09575 | 0.10231 | 0.07079 | 1.35× | 1.45× |
| c4 | 6 | 0.48838 | 0.49445 | 0.31651 | 1.54× | 1.56× |
| class | 6 | 0.25074 | 0.24648 | 0.15697 | 1.60× | 1.57× |
| er | 6 | 0.21622 | 0.22062 | 0.15192 | 1.42× | 1.45× |
| eventmodeling | 6 | 0.16945 | 0.20614 | 0.08430 | 2.01× | 2.45× |
| gantt | 6 | 0.25005 | 0.25724 | 0.14568 | 1.72× | 1.77× |
| git | 6 | 0.41420 | 0.42248 | 0.21990 | 1.88× | 1.92× |
| info | 2 | 0.00235 | 0.00332 | 0.00189 | 1.24× | 1.76× |
| journey | 1 | 0.01657 | 0.01708 | 0.00850 | 1.95× | 2.01× |
| kanban | 3 | 0.05536 | 0.05420 | 0.03907 | 1.42× | 1.39× |
| mindmap | 6 | 0.09504 | 0.11069 | 0.05364 | 1.77× | 2.06× |
| packet | 4 | 0.06122 | 0.06432 | 0.03218 | 1.90× | 2.00× |
| pie | 3 | 0.03444 | 0.04033 | 0.01952 | 1.76× | 2.07× |
| quadrant | 3 | 0.05456 | 0.05923 | 0.03389 | 1.61× | 1.75× |
| requirement | 6 | 0.27734 | 0.23680 | 0.14587 | 1.90× | 1.62× |
| sankey | 6 | 0.35430 | 0.37626 | 0.10854 | 3.26× | 3.47× |
| sequence | 6 | 0.13360 | 0.14517 | 0.09397 | 1.42× | 1.54× |
| state | 6 | 0.32331 | 0.35384 | 0.20188 | 1.60× | 1.75× |
| timeline | 6 | 0.19815 | 0.22760 | 0.10255 | 1.93× | 2.22× |
| xychart | 6 | 0.11555 | 0.12233 | 0.07551 | 1.53× | 1.62× |
| agentflow | 6 | 0.33245 | 0.32704 | 0.27232 | 1.22× | 1.20× |
| architecture | 6 | 0.22432 | 0.23041 | 0.09129 | 2.46× | 2.52× |
| cynefin | 3 | 0.10426 | 0.13998 | 0.05889 | 1.77× | 2.38× |
| ishikawa | 1 | 0.01880 | 0.01992 | 0.01028 | 1.83× | 1.94× |
| radar | 3 | 0.07620 | 0.07758 | 0.03169 | 2.40× | 2.45× |
| railroad | 6 | 0.21173 | 0.22291 | 0.15732 | 1.35× | 1.42× |
| swimlane | 6 | 0.29082 | 0.31974 | 0.20936 | 1.39× | 1.53× |
| treemap | 6 | 0.14076 | 0.14567 | 0.06753 | 2.08× | 2.16× |
| treeview | 6 | 0.10130 | 0.10393 | 0.04776 | 2.12× | 2.18× |
| usecase | 6 | 0.34933 | 0.34204 | 0.22743 | 1.54× | 1.50× |
| venn | 6 | 0.40469 | 0.40134 | 0.17346 | 2.33× | 2.31× |
| wardley | 6 | 0.15342 | 0.14994 | 0.09287 | 1.65× | 1.61× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-20-compact-label/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-20-compact-label/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-20-compact-label/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
