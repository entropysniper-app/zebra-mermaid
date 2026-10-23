# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele 1.9.0, commit `e7f092a4de147cc32210913c20e36eaa5fcd0ef1`; v26.3.0; rustc 1.96.1 (31fca3adb 2026-06-26).

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **1.76× for API return** and **1.89× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.46143 | 0.50117 | 0.24760 | 1.86× | 2.02× |
| block | 6 | 0.09285 | 0.09739 | 0.07348 | 1.26× | 1.33× |
| c4 | 6 | 0.49826 | 0.51406 | 0.31557 | 1.58× | 1.63× |
| class | 6 | 0.30222 | 0.31698 | 0.17011 | 1.78× | 1.86× |
| er | 6 | 0.20020 | 0.21578 | 0.14826 | 1.35× | 1.46× |
| eventmodeling | 6 | 0.14884 | 0.15626 | 0.06610 | 2.25× | 2.36× |
| gantt | 6 | 0.23665 | 0.24729 | 0.14138 | 1.67× | 1.75× |
| git | 6 | 0.40046 | 0.40797 | 0.21288 | 1.88× | 1.92× |
| info | 2 | 0.00257 | 0.00306 | 0.00114 | 2.25× | 2.68× |
| journey | 1 | 0.01236 | 0.01391 | 0.00839 | 1.47× | 1.66× |
| kanban | 3 | 0.05125 | 0.05390 | 0.04081 | 1.26× | 1.32× |
| mindmap | 6 | 0.09882 | 0.11683 | 0.04794 | 2.06× | 2.44× |
| packet | 4 | 0.05127 | 0.05515 | 0.02904 | 1.77× | 1.90× |
| pie | 3 | 0.03239 | 0.03738 | 0.01850 | 1.75× | 2.02× |
| quadrant | 3 | 0.05278 | 0.05653 | 0.03207 | 1.65× | 1.76× |
| requirement | 6 | 0.23627 | 0.24861 | 0.14939 | 1.58× | 1.66× |
| sankey | 6 | 0.41271 | 0.45642 | 0.14896 | 2.77× | 3.06× |
| sequence | 6 | 0.11549 | 0.12890 | 0.07760 | 1.49× | 1.66× |
| state | 6 | 0.34430 | 0.35754 | 0.20158 | 1.71× | 1.77× |
| timeline | 6 | 0.19780 | 0.22364 | 0.09781 | 2.02× | 2.29× |
| xychart | 6 | 0.11593 | 0.12525 | 0.07254 | 1.60× | 1.73× |
| agentflow | 6 | 0.33215 | 0.38668 | 0.27638 | 1.20× | 1.40× |
| architecture | 6 | 0.21082 | 0.22607 | 0.09089 | 2.32× | 2.49× |
| cynefin | 3 | 0.10498 | 0.13768 | 0.05374 | 1.95× | 2.56× |
| ishikawa | 1 | 0.01616 | 0.01722 | 0.00986 | 1.64× | 1.75× |
| radar | 3 | 0.07131 | 0.07328 | 0.03508 | 2.03× | 2.09× |
| railroad | 6 | 0.20581 | 0.21719 | 0.14836 | 1.39× | 1.46× |
| swimlane | 6 | 0.31842 | 0.29023 | 0.20596 | 1.55× | 1.41× |
| treemap | 6 | 0.13556 | 0.14054 | 0.06660 | 2.04× | 2.11× |
| treeview | 6 | 0.09539 | 0.10407 | 0.04585 | 2.08× | 2.27× |
| usecase | 6 | 0.43197 | 0.43225 | 0.23964 | 1.80× | 1.80× |
| venn | 6 | 0.41535 | 0.41593 | 0.17191 | 2.42× | 2.42× |
| wardley | 6 | 0.14676 | 0.15418 | 0.08099 | 1.81× | 1.90× |

## Evidence and reproduction

[Summary, source hashes and binary hashes](research/output-benchmark-2026-10-23-pele-1-9-0-ratex/comparison.json) · [Exact inputs](research/output-benchmark-2026-10-23-pele-1-9-0-ratex/cases.json) · [Verified SVG hashes](research/output-benchmark-2026-10-23-pele-1-9-0-ratex/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
