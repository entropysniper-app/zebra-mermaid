# Performance comparison

Measured: 2026-10-20T19:43:57.232150+00:00.

The current library is compared with three other renderers using the same **167 Playground samples across 33 diagram types**. All four engines attempt every original input without syntax rewriting. Timing includes only samples that pass type and SVG validation in all four engines.

## Versions

| Renderer | Version | Execution |
| --- | --- | --- |
| zebra-mermaid | 0.1.0 | Native Rust, release, thin LTO, codegen-units=1 |
| Pele | 1.7.0 | Node, commit `8603bf3313add9143cd417015d4a6bd6c522dada` |
| Mermaid.js | 12.1.0 | Chromium 151.0.7922.34 |
| mermaid-rs-renderer | 0.3.1 | Native Rust, release, thin LTO, codegen-units=1; commit `3726ccbffe0e8032361eb9668694b24f77858060` |

Runtime versions: rustc 1.96.1 (31fca3adb 2026-06-26); Node v26.3.0. The two native libraries share one development benchmark driver and matching optimization settings. The comparison renderer is not a dependency of the production library.

## Method

- Measure full parsing, layout, and SVG generation, with no parsed-model or output-cache reuse.
- Use identical inputs within each row. Values are milliseconds for the **entire batch**, not per diagram.
- Run five warm-up batches, calibrate each timed segment to at least 10 ms, then collect at least nine samples and 300 ms per group.
- Run three independent processes per engine, alternating engine order. Take the lower median within each run, then the median of the three runs.
- Exclude process/browser startup, imports, font warm-up, validation, IPC, and disk output. Mermaid DOM cleanup and mermaid-rs-renderer's owned-options clone remain inside their loops.
- Request Arial 16. Rust/Pele use the same Arial metrics and CSS output; Mermaid uses browser measurements and a dark theme; mermaid-rs-renderer uses its default native font measurement with `fast_text_metrics=false`. Authored directives remain active. The clock is fixed to 2026-10-19 00:00 UTC where supported.

These are steady-state rendering measurements. Layout and font behavior differ between engines, so timings are not a claim of identical work or pixel equivalence across all four engines. Compare values within a row; batch sizes and diagram complexity differ across rows.

## Coverage

| Renderer | Returned SVG / 167 | Passed type and SVG checks / 167 |
| --- | ---: | ---: |
| Mermaid.js | 167 | 167 |
| Pele | 167 | 167 |
| zebra-mermaid | 167 | 167 |
| mermaid-rs-renderer | 156 | 110 |

The common set contains **110/167 samples across 22 diagram types**. A dash means no shared eligible input, not a zero rendering cost. The complete success, failure, and exclusion records are retained.

SVG validation requires valid XML and positive finite dimensions. The native comparison renderer must also report the expected diagram kind: inputs misidentified as flowcharts are recorded as returned SVG but excluded from eligible success and timing. Mermaid HTML output is serialized through the DOM outside timing; its Info output uses the browser viewport when no viewBox is present.

Rust/Pele SVG bytes and dimensions match on **167/167 samples**. The other engines are checked for renderability and type; this does not establish full semantic or visual equivalence.

## Results

| Diagram | Shared / total samples | Mermaid.js (ms) | Pele (ms) | zebra-mermaid (ms) | mermaid-rs-renderer (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 / 6 | 81.80000 | 0.30690 | 0.18737 | 87.99788 |
| block | 6 / 6 | 77.90000 | 0.09335 | 0.07027 | 0.14056 |
| c4 | 6 / 6 | 92.10000 | 0.45881 | 0.29815 | 1.09953 |
| class | 5 / 6 | 115.50000 | 0.20188 | 0.12824 | 0.36211 |
| er | 6 / 6 | 77.90000 | 0.21898 | 0.14286 | 6.06925 |
| eventmodeling | 0 / 6 | — | — | — | — |
| gantt | 6 / 6 | 15.30000 | 0.23793 | 0.13613 | 0.38637 |
| git | 6 / 6 | 101.10000 | 0.40431 | 0.21322 | 0.59711 |
| info | 0 / 2 | — | — | — | — |
| journey | 1 / 1 | 2.82500 | 0.01560 | 0.00799 | 0.03888 |
| kanban | 3 / 3 | 26.00000 | 0.05144 | 0.03759 | 0.12177 |
| mindmap | 6 / 6 | 82.60000 | 0.09250 | 0.04590 | 0.35470 |
| packet | 4 / 4 | 6.30000 | 0.05852 | 0.03045 | 2.33180 |
| pie | 3 / 3 | 4.32500 | 0.03243 | 0.01826 | 0.07220 |
| quadrant | 3 / 3 | 8.90000 | 0.05227 | 0.03236 | 0.08427 |
| requirement | 6 / 6 | 77.90000 | 0.22924 | 0.14127 | 1.80328 |
| sankey | 6 / 6 | 29.50000 | 0.29463 | 0.10521 | 0.41971 |
| sequence | 4 / 6 | 22.00000 | 0.08845 | 0.05366 | 0.70634 |
| state | 6 / 6 | 112.60000 | 0.32142 | 0.19023 | 5.20194 |
| timeline | 6 / 6 | 27.30000 | 0.19698 | 0.09189 | 0.37105 |
| xychart | 6 / 6 | 23.70000 | 0.11391 | 0.07227 | 0.20699 |
| agentflow | 0 / 6 | — | — | — | — |
| architecture | 6 / 6 | 64.60000 | 0.20928 | 0.08704 | 0.25304 |
| cynefin | 0 / 3 | — | — | — | — |
| ishikawa | 0 / 1 | — | — | — | — |
| radar | 3 / 3 | 5.75000 | 0.07176 | 0.03015 | 0.08910 |
| railroad | 0 / 6 | — | — | — | — |
| swimlane | 0 / 6 | — | — | — | — |
| treemap | 6 / 6 | 13.60000 | 0.13893 | 0.06586 | 0.24751 |
| treeview | 0 / 6 | — | — | — | — |
| usecase | 0 / 6 | — | — | — | — |
| venn | 0 / 6 | — | — | — | — |
| wardley | 0 / 6 | — | — | — | — |

## Per-engine coverage

| Diagram | Samples | Mermaid.js | Pele | zebra-mermaid | mermaid-rs-renderer |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 6 | 6 | 6 | 6 |
| block | 6 | 6 | 6 | 6 | 6 |
| c4 | 6 | 6 | 6 | 6 | 6 |
| class | 6 | 6 | 6 | 6 | 5 |
| er | 6 | 6 | 6 | 6 | 6 |
| eventmodeling | 6 | 6 | 6 | 6 | 0 |
| gantt | 6 | 6 | 6 | 6 | 6 |
| git | 6 | 6 | 6 | 6 | 6 |
| info | 2 | 2 | 2 | 2 | 0 |
| journey | 1 | 1 | 1 | 1 | 1 |
| kanban | 3 | 3 | 3 | 3 | 3 |
| mindmap | 6 | 6 | 6 | 6 | 6 |
| packet | 4 | 4 | 4 | 4 | 4 |
| pie | 3 | 3 | 3 | 3 | 3 |
| quadrant | 3 | 3 | 3 | 3 | 3 |
| requirement | 6 | 6 | 6 | 6 | 6 |
| sankey | 6 | 6 | 6 | 6 | 6 |
| sequence | 6 | 6 | 6 | 6 | 4 |
| state | 6 | 6 | 6 | 6 | 6 |
| timeline | 6 | 6 | 6 | 6 | 6 |
| xychart | 6 | 6 | 6 | 6 | 6 |
| agentflow | 6 | 6 | 6 | 6 | 0 |
| architecture | 6 | 6 | 6 | 6 | 6 |
| cynefin | 3 | 3 | 3 | 3 | 0 |
| ishikawa | 1 | 1 | 1 | 1 | 0 |
| radar | 3 | 3 | 3 | 3 | 3 |
| railroad | 6 | 6 | 6 | 6 | 0 |
| swimlane | 6 | 6 | 6 | 6 | 0 |
| treemap | 6 | 6 | 6 | 6 | 6 |
| treeview | 6 | 6 | 6 | 6 | 0 |
| usecase | 6 | 6 | 6 | 6 | 0 |
| venn | 6 | 6 | 6 | 6 | 0 |
| wardley | 6 | 6 | 6 | 6 | 0 |

## Reproduce

Use the reference revisions listed above, the locked website dependencies, and Playwright with Chromium installed. From the repository root:

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/run-cross.py
BENCH_REPORT_SUFFIX=-pele-1-7-0 python3 tools/bench/publish-cross.py
```

`PLAYWRIGHT_MODULE` and `PLAYWRIGHT_BROWSERS_PATH` can point to a separate browser installation. Publishing requires a fresh report suffix to preserve existing measurement evidence.

[Summary and versions](research/four-engine-benchmark-2026-10-20-borrowed-entry/comparison.json) · [Per-input outcomes](research/four-engine-benchmark-2026-10-20-borrowed-entry/verification.json) · [Shared inputs](research/four-engine-benchmark-2026-10-20-borrowed-entry/common.json) · [All inputs](research/four-engine-benchmark-2026-10-20-borrowed-entry/cases.json) · [Source and binary fingerprints](research/four-engine-benchmark-2026-10-20-borrowed-entry/metadata.json)

The same directory contains twelve compressed raw timing reports, SVG output, and Rust/Pele equality records. Benchmark success is separate from [original-test acceptance](UPSTREAM-TEST-PARITY.md).
