# Performance comparison

Measured: 2026-10-20T11:21:19.374740+00:00.

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
| flowchart | 6 / 6 | 82.90000 | 0.30617 | 0.19904 | 87.53354 |
| block | 6 / 6 | 77.80000 | 0.09435 | 0.07550 | 0.14161 |
| c4 | 6 / 6 | 90.80000 | 0.45684 | 0.35040 | 1.10236 |
| class | 5 / 6 | 116.90000 | 0.19848 | 0.13863 | 0.36179 |
| er | 6 / 6 | 77.50000 | 0.20947 | 0.15893 | 6.05648 |
| eventmodeling | 0 / 6 | — | — | — | — |
| gantt | 6 / 6 | 15.50000 | 0.23572 | 0.15359 | 0.38668 |
| git | 6 / 6 | 100.80000 | 0.40088 | 0.24074 | 0.59998 |
| info | 0 / 2 | — | — | — | — |
| journey | 1 / 1 | 2.82500 | 0.01541 | 0.01012 | 0.03904 |
| kanban | 3 / 3 | 26.20000 | 0.05110 | 0.04187 | 0.12274 |
| mindmap | 6 / 6 | 83.10000 | 0.09121 | 0.06265 | 0.34748 |
| packet | 4 / 4 | 6.35000 | 0.05921 | 0.03504 | 2.32719 |
| pie | 3 / 3 | 4.40000 | 0.03208 | 0.02175 | 0.07164 |
| quadrant | 3 / 3 | 8.80000 | 0.05277 | 0.03673 | 0.08281 |
| requirement | 6 / 6 | 77.90000 | 0.22656 | 0.16956 | 1.80640 |
| sankey | 6 / 6 | 29.50000 | 0.29022 | 0.12349 | 0.41759 |
| sequence | 4 / 6 | 22.10000 | 0.08847 | 0.06273 | 0.69995 |
| state | 6 / 6 | 112.30000 | 0.31940 | 0.21026 | 5.18648 |
| timeline | 6 / 6 | 27.30000 | 0.19553 | 0.11976 | 0.37111 |
| xychart | 6 / 6 | 23.60000 | 0.11354 | 0.08875 | 0.20724 |
| agentflow | 0 / 6 | — | — | — | — |
| architecture | 6 / 6 | 64.20000 | 0.21129 | 0.09674 | 0.25560 |
| cynefin | 0 / 3 | — | — | — | — |
| ishikawa | 0 / 1 | — | — | — | — |
| radar | 3 / 3 | 5.70000 | 0.07228 | 0.03455 | 0.08873 |
| railroad | 0 / 6 | — | — | — | — |
| swimlane | 0 / 6 | — | — | — | — |
| treemap | 6 / 6 | 13.60000 | 0.13802 | 0.07279 | 0.24867 |
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

[Summary and versions](research/four-engine-benchmark-2026-10-20-decimal-formatting/comparison.json) · [Per-input outcomes](research/four-engine-benchmark-2026-10-20-decimal-formatting/verification.json) · [Shared inputs](research/four-engine-benchmark-2026-10-20-decimal-formatting/common.json) · [All inputs](research/four-engine-benchmark-2026-10-20-decimal-formatting/cases.json) · [Source and binary fingerprints](research/four-engine-benchmark-2026-10-20-decimal-formatting/metadata.json)

The same directory contains twelve compressed raw timing reports, SVG output, and Rust/Pele equality records. Benchmark success is separate from [original-test acceptance](UPSTREAM-TEST-PARITY.md).
