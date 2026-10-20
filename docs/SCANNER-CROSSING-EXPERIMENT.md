# Scanner storage and crossing counter experiment

This combined candidate is **not retained as a completed optimization**. Across all 167 unchanged Samples and 33 types, paired native render time was **0.99953× the baseline (−0.05%)**, essentially unchanged. Documentation render was **1.00063× (+0.06%)** and parse **0.99922× (−0.08%)**. All three release instruction-text totals increased. Fewer allocations in 12 types do not establish progress to the current **2× API-return target**. A simpler indexed scanner representation is being evaluated next; the crossing counter remains under evaluation.

Baseline source: `4a03a25c2ef74b393b283751cb04086edd3c2a81`, with identical production code at `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. The exact uncommitted candidate over `c9db1c9` is saved as a [patch](research/scanner-crossing-experiment-2026-10-20/candidate.patch), SHA-256 `c6b44ac0e3e03cd7b164fe64b0bd66bf2561a60bf5256792e3a5de27da0820bb`. No production commit is assigned to this experimental variant. Pele remains fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1.

## Profiling and candidate

Fresh external CPU captures covered the complete 114-example Flowchart batch, the large labelled mesh, Info, Packet and State documentation. The large mesh had 2,571 inclusive layered-layout frame occurrences and 277 in the crossing-count closure out of 3,668 thread samples. Info had 472 `Grammar::reader` occurrences out of 3,455. These overlapping inclusive counts include descendants and are not additive exclusive CPU percentages.

The candidate puts small dead-rule flags on the stack with the original heap fallback for large public tables. A four-handle sorted scanner map preserves lazy factory invocation, mutable per-source state, token-index drop order and factory-panic behavior; larger active sets spill into a sorted Vec. Public APIs, boxed scanner factories and matching order remain unchanged.

The crossing counter uses a Fenwick tree to count the same strictly inverted target pairs, with smaller integer scratch storage. Target order, duplicate handling, offset positions, floating-point arithmetic, barycentric sweeps, stop rules and work limits remain unchanged. Every input still undergoes complete parsing, layout and SVG generation; no model, token, label or output is cached.

Afterward, large-mesh crossing-closure occurrences were 165, while Info reader occurrences were 257. Packet reader occurrences rose 351 → 361. These captures do not prove an overall improvement. [CPU aggregates](research/scanner-crossing-experiment-2026-10-20/hot-frames.json) and capture hashes retain both phases.

## Timing

Three independent processes per version/engine, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, CPU samples, allocator probes, section measurements and RSS checks did not overlap latency timing. Original sources, options, metrics and fixed UTC clock were retained. No run was discarded or replaced.

Sample medians fell in 16 types and rose in 17. The candidate's separately remeasured reference speedups are **1.73× at API return** and **1.89× UTF-8 ready**: 24 types below 2× and nine at least 2×. These do not prove that the committed renderer slowed relative to the previous round: the paired native ratio is 0.99953, and the reference is remeasured each round. Cross-round absolute ratios must not be divided to infer a native gain. Small changes below are not statistical-significance claims.

| Type | Samples | Before (ms) | Candidate (ms) | Time change | API speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 0.202304 | 0.199917 | -1.18% | 1.62× |
| block | 6 | 0.071075 | 0.070569 | -0.71% | 1.41× |
| c4 | 6 | 0.326717 | 0.316750 | -3.05% | 1.47× |
| class | 6 | 0.160427 | 0.156727 | -2.31% | 1.50× |
| er | 6 | 0.146424 | 0.148655 | +1.52% | 1.42× |
| eventmodeling | 6 | 0.081216 | 0.080006 | -1.49% | 2.05× |
| gantt | 6 | 0.147577 | 0.146393 | -0.80% | 1.64× |
| git | 6 | 0.221680 | 0.215947 | -2.59% | 1.88× |
| info | 2 | 0.001665 | 0.001630 | -2.11% | 1.39× |
| journey | 1 | 0.008225 | 0.008400 | +2.13% | 1.87× |
| kanban | 3 | 0.037271 | 0.037937 | +1.78% | 1.40× |
| mindmap | 6 | 0.051682 | 0.051611 | -0.14% | 1.81× |
| packet | 4 | 0.030359 | 0.030393 | +0.11% | 2.01× |
| pie | 3 | 0.019185 | 0.018838 | -1.81% | 1.77× |
| quadrant | 3 | 0.032974 | 0.033288 | +0.95% | 1.61× |
| requirement | 6 | 0.143917 | 0.144567 | +0.45% | 1.58× |
| sankey | 6 | 0.105186 | 0.105885 | +0.66% | 2.84× |
| sequence | 6 | 0.084148 | 0.085111 | +1.14% | 1.58× |
| state | 6 | 0.190036 | 0.192400 | +1.24% | 1.68× |
| timeline | 6 | 0.097802 | 0.099386 | +1.62% | 2.03× |
| xychart | 6 | 0.073479 | 0.073271 | -0.28% | 1.56× |
| agentflow | 6 | 0.270709 | 0.273559 | +1.05% | 1.20× |
| architecture | 6 | 0.087240 | 0.086949 | -0.33% | 2.56× |
| cynefin | 3 | 0.057069 | 0.057202 | +0.23% | 1.76× |
| ishikawa | 1 | 0.009738 | 0.009688 | -0.52% | 1.89× |
| radar | 3 | 0.030636 | 0.030847 | +0.69% | 2.38× |
| railroad | 6 | 0.153392 | 0.154423 | +0.67% | 1.33× |
| swimlane | 6 | 0.210959 | 0.216283 | +2.52% | 1.32× |
| treemap | 6 | 0.067816 | 0.067226 | -0.87% | 2.10× |
| treeview | 6 | 0.046673 | 0.045441 | -2.64% | 2.09× |
| usecase | 6 | 0.223249 | 0.228016 | +2.14% | 1.45× |
| venn | 6 | 0.166242 | 0.165331 | -0.55% | 2.35× |
| wardley | 6 | 0.084193 | 0.085265 | +1.27% | 1.69× |

[Every Sample process, exact inputs, SVG verification and source/binary fingerprints](research/scanner-crossing-experiment-2026-10-20/samples/comparison.json) are retained with the raw runs. The separate four-engine subset is preserved in [its own report](research/scanner-crossing-experiment-2026-10-20/four-engines/comparison.json); it does not replace the main full-type collection.

All 39 extended rows are preserved, including all 20 observed render regressions:

| Extended workload | Render time change | Parse time change |
| --- | ---: | ---: |
| docs: all flowchart examples | -1.07% | -0.74% |
| tree: 50 nodes | +1.75% | +2.90% |
| mesh: 50 nodes, 20 extra labelled edges | -0.85% | +0.35% |
| tree: 500 nodes | +0.32% | -0.45% |
| mesh: 500 nodes, 200 extra labelled edges | -1.76% | +0.13% |
| corpus: agentflow-docs | +0.86% | +0.46% |
| corpus: agentflow-reference | +1.00% | -0.85% |
| corpus: architecture-docs | +0.32% | +1.60% |
| corpus: block-docs | +1.66% | +0.93% |
| corpus: c4-docs | +1.34% | +1.51% |
| corpus: class-docs | +0.04% | +2.49% |
| corpus: cynefin-docs | -1.32% | -2.77% |
| corpus: er-docs | -0.30% | -0.16% |
| corpus: eventmodeling-docs | +1.34% | -0.07% |
| corpus: flowchart-docs | -1.26% | +0.09% |
| corpus: gantt-docs | +0.74% | -0.40% |
| corpus: git-docs | +0.82% | +0.77% |
| corpus: info-docs | -3.51% | -8.22% |
| corpus: ishikawa-docs | -0.05% | +5.30% |
| corpus: journey-docs | -0.31% | +4.15% |
| corpus: kanban-docs | -0.51% | +0.40% |
| corpus: mindmap-docs | +1.12% | +0.44% |
| corpus: packet-docs | -0.52% | +1.04% |
| corpus: pie-docs | -0.24% | -1.87% |
| corpus: quadrant-docs | +2.04% | +0.98% |
| corpus: radar-docs | -1.47% | -1.78% |
| corpus: railroad-docs | +2.11% | +0.00% |
| corpus: requirement-docs | +0.16% | -1.03% |
| corpus: sankey-docs | -0.12% | +1.92% |
| corpus: sequence-docs | -0.96% | +0.65% |
| corpus: state-docs | +0.38% | -0.36% |
| corpus: swimlane-docs | +0.96% | +0.25% |
| corpus: timeline-docs | +0.01% | -2.80% |
| corpus: treemap-docs | -1.20% | -3.16% |
| corpus: treeview-docs | -0.87% | +0.29% |
| corpus: usecase-docs | +0.27% | -0.44% |
| corpus: venn-docs | -0.36% | -0.14% |
| corpus: wardley-docs | -0.85% | -0.77% |
| corpus: xychart-docs | +2.01% | -1.04% |

[Extended comparison and raw independent runs](research/scanner-crossing-experiment-2026-10-20/extended-comparison.json) retain the full data. Info documentation parse fell 8.22%, while other scanner users did not all improve. The large labelled mesh fell 1.76%, the 500-node tree rose 0.32%, and the 50-node tree rose 1.75%. This evidence does not justify a universal parsing/layout speedup.

## Code size

All release builds use the same compiler, locked dependencies, full features, release/thin LTO/one codegen unit and normalized source paths; none is stripped. Section reading runs separately from latency and resource measurements.

| Release artifact | File before → candidate (bytes) | Text before → candidate (bytes) |
| --- | ---: | ---: |
| cli | 5,210,752 → 5,211,104 | 3,853,944 → 3,855,468 |
| ffi-static | 41,708,256 → 41,717,344 | 4,273,968 → 4,275,408 |
| ffi-dynamic | 4,798,544 → 4,798,960 | 3,561,420 → 3,562,936 |

The CLI text increase is 1,524 bytes; the dynamic library increase is 1,516, and the static archive increase is 1,440. The static archive's text sums every member before host dead stripping; its file bytes also contain bitcode and other data, and neither equals the final host link increment. [Complete release section totals and hashes](research/scanner-crossing-experiment-2026-10-20/size-comparison.json) and [separate driver totals](research/scanner-crossing-experiment-2026-10-20/driver-sizes.json) are retained.

## Allocations and process resources

The allocator probe runs five warm-up batches and one counted batch separately from latency, excluding inputs/options and dropping each result. Across Samples, allocation counts fell in **12 types**, with 21 unchanged; requested bytes fell in **19**, with 14 unchanged. Across 39 extended groups, counts fell in 12 and requested bytes in 25. Reallocations and additional-live peaks were unchanged everywhere. Info allocations were **32 → 28** and requested bytes **4,379 → 4,039**. All batch-end net retained bytes were zero; this is not a general leak audit. Requested bytes are not RSS. [Sample allocator data](research/scanner-crossing-experiment-2026-10-20/allocations-samples.json) and [extended allocator data](research/scanner-crossing-experiment-2026-10-20/allocations-extended.json) retain every row.

Separately, each group/version ran in three fresh alternating processes, five warm-up batches and at least a one-second complete-render loop. Process-wide OS peak RSS and CPU totals include startup, input loading and warm-up. Sample median peak RSS was **23 lower / one unchanged / nine higher**; extended groups were **19 lower / four unchanged / 16 higher**. The large labelled mesh rose **14,221,312 → 14,581,760 bytes (+352 KiB)** and Packet Samples **3,784,704 → 3,915,776 (+128 KiB)**. This is mixed evidence, not an overall RSS reduction claim. Peaks are not summed across independent processes. [Every Sample process](research/scanner-crossing-experiment-2026-10-20/resources-samples.json) and [every extended process](research/scanner-crossing-experiment-2026-10-20/resources-extended.json) are retained.

## Correctness and acceptance

- Full workspace candidate: **3,230 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy completed with unchanged warning counts/categories.
- **761/761 complete reference responses, 167/167 Sample SVGs and 39/39 extended verification records matched**.
- Four supplemental tests cover scanner lazy/state/drop/panic/opener behavior through inline and large-table paths, plus exact strict inversions against an independent quadratic definition for six explicit and 2,048 seeded rows (`0x11c0de`), including duplicates and dirty scratch reuse. The unchanged 15-table independent Node token reference contains 2,008 cases. These additions do not count as original-test ports.
- Original test bodies, fixture expectations, assertions, seeds, iteration counts, thresholds and mappings were unchanged. Existing token tests remained byte-identical.
- Original independent-timeout acceptance: **2,785 passed, zero failed, seven preserved skips, 4,034 unmapped, zero unverified**. Strict completeness exited one with `complete: false`; original-test acceptance remains incomplete.

[Validation](research/scanner-crossing-experiment-2026-10-20/validation.json) and [summary/source/input/binary fingerprints](research/scanner-crossing-experiment-2026-10-20/summary.json) identify the measured candidate. Its complete evidence is retained while the scanner representation is replaced. The README and current output benchmark continue to describe the last committed production implementation.
