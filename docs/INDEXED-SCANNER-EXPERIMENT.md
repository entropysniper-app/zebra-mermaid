# Indexed scanner experiment under observed contention

This variant remains **experimental**. The full 167-Sample / 33-type measurement returned **1.91× at API return** and **2.10× UTF-8 ready**, with paired native time **0.94790× (−5.21%)**. Documentation render instead rose **10.17%**, and parse **8.76%**. An unrelated build/test driver was observed during Sample timing. These conflicting observations do not establish an isolated causal speedup or attainment of the **2× API-return target**. Every raw process and regression is retained; none is replaced or discarded.

All three release instruction-text totals increased, and most process RSS medians rose. The scanner/counter runtime candidate is therefore not committed as a validated optimization. The next candidate evaluates remaining label-string temporaries against the unchanged committed renderer. Supplemental scanner contracts will remain useful independently of this representation.

Baseline: `4a03a25c2ef74b393b283751cb04086edd3c2a81`, with identical production code at `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. The exact candidate over `f5d27e5` is saved as a [patch](research/scanner-indexed-experiment-2026-10-20/candidate.patch), SHA-256 `889a470a7b750730350b651515bd9e588304ea931d0a141b3ab5c6b0153f4b5f`. No production commit is assigned to it. Reference: Pele 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1.

## Implementation and measurement conditions

Fresh CPU captures preceded baseline reuse, with all frozen source, input and binary hashes checked. Built-in token grammars use 64-entry indexed stack arrays for dead flags and optional boxed scanner handles; larger public tables retain the original heap vectors. Lazy factories, matching order, mutable per-source state, token-index drops and panic paths remain intact. The crossing counter still uses the exact integer Fenwick representation from the previous experiment, preserving target/offset order, duplicates, layout sweeps, floating-point calculations and limits.

The target-build diagnostic measured **1,024 bytes for scanner handles and 64 for dead flags**. This is stack scratch, separate from counted heap allocations and OS peak RSS, and is not a public ABI assertion. [Diagnostic](research/scanner-indexed-experiment-2026-10-20/scratch-layout.json).

Latency used three independent processes per engine/version in alternating order, five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms. Our builds, tests, CPU captures, allocator probes and RSS runs were serialized outside latency. **That orchestration did not ensure whole-host isolation:** a read-only snapshot during `final-output` found an unrelated build/test driver. An earlier diagnostic also showed other active workloads. A snapshot does not measure their duration or contribution to any row's change. [Observation](research/scanner-indexed-experiment-2026-10-20/measurement-conditions.json) records only sanitized conditions and the ignored raw capture's fingerprint.

The original size-tool scheduling field describes its own measurement pipeline; it is not evidence that the entire host was idle. The tool now states its own scope directly. A new read-only `tools/bench/check-timing-window.py` preflight reports listed build/test drivers before starting subsequent timings; it does not stop processes or prove sustained idleness.

## Full Samples and extended loads

Observed Sample medians were lower in 24 types and higher in nine. API speedups span **Swimlane 1.23× to Sankey 3.34×**, with 20 types below 2× and 13 at least 2×. UTF-8 is a separate endpoint and does not satisfy the API-return target. The reference is remeasured each round; dividing cross-round absolute speedups does not establish a native improvement. No statistical-significance claim is made.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -6.65% | 1.98× | 1.99× |
| block | -15.56% | 1.55× | 1.79× |
| c4 | -13.31% | 1.68× | 1.94× |
| class | -11.77% | 2.11× | 2.14× |
| er | -8.42% | 1.69× | 1.72× |
| eventmodeling | -11.77% | 2.24× | 2.71× |
| gantt | -14.44% | 1.95× | 2.02× |
| git | -22.83% | 2.53× | 2.33× |
| info | -6.03% | 1.46× | 1.87× |
| journey | -12.35% | 2.29× | 2.57× |
| kanban | -12.51% | 1.57× | 1.74× |
| mindmap | -14.00% | 2.26× | 2.55× |
| packet | -13.25% | 2.34× | 2.54× |
| pie | -4.38% | 2.08× | 1.98× |
| quadrant | -11.49% | 1.74× | 2.05× |
| requirement | -10.64% | 1.74× | 1.93× |
| sankey | -9.45% | 3.34× | 4.28× |
| sequence | -11.17% | 1.77× | 1.96× |
| state | +17.02% | 1.62× | 1.68× |
| timeline | -0.39% | 2.25× | 2.28× |
| xychart | -4.72% | 1.93× | 1.96× |
| agentflow | -5.39% | 1.43× | 1.30× |
| architecture | -1.34% | 2.64× | 2.83× |
| cynefin | +3.63% | 1.66× | 2.21× |
| ishikawa | +9.78% | 1.79× | 2.06× |
| radar | +7.20% | 2.32× | 2.60× |
| railroad | +11.79% | 1.50× | 1.48× |
| swimlane | +5.09% | 1.23× | 1.47× |
| treemap | +1.30% | 1.99× | 2.59× |
| treeview | +15.83% | 1.93× | 2.36× |
| usecase | +12.39% | 1.49× | 1.54× |
| venn | -14.47% | 2.30× | 2.50× |
| wardley | -2.92% | 2.08× | 2.13× |

[All Sample inputs, SVG verification, timings and fingerprints](research/scanner-indexed-experiment-2026-10-20/samples/comparison.json) and every raw process remain available. The separate [four-engine common subset](research/scanner-indexed-experiment-2026-10-20/four-engines/comparison.json) does not replace the full set.

All 39 extended rows remain recorded, including the documentation regressions and unchanged parsing paths:

| Extended workload | Render time change | Parse time change |
| --- | ---: | ---: |
| docs: all flowchart examples | +3.54% | +3.21% |
| tree: 50 nodes | -2.07% | +1.20% |
| mesh: 50 nodes, 20 extra labelled edges | +6.96% | -9.46% |
| tree: 500 nodes | +0.27% | +0.42% |
| mesh: 500 nodes, 200 extra labelled edges | -8.21% | -2.56% |
| corpus: agentflow-docs | -0.48% | -10.24% |
| corpus: agentflow-reference | -1.71% | -5.27% |
| corpus: architecture-docs | +1.10% | +2.84% |
| corpus: block-docs | -2.58% | +0.11% |
| corpus: c4-docs | +16.73% | +3.26% |
| corpus: class-docs | +0.05% | +5.91% |
| corpus: cynefin-docs | +21.44% | -7.46% |
| corpus: er-docs | -0.24% | +3.89% |
| corpus: eventmodeling-docs | +49.98% | +30.03% |
| corpus: flowchart-docs | +27.75% | +44.37% |
| corpus: gantt-docs | +49.27% | +51.85% |
| corpus: git-docs | +19.92% | +41.74% |
| corpus: info-docs | +6.79% | +8.31% |
| corpus: ishikawa-docs | +2.38% | +6.39% |
| corpus: journey-docs | +2.77% | +5.60% |
| corpus: kanban-docs | -7.89% | +2.60% |
| corpus: mindmap-docs | +2.60% | -2.04% |
| corpus: packet-docs | +0.41% | -0.95% |
| corpus: pie-docs | +2.22% | -2.79% |
| corpus: quadrant-docs | +3.64% | +3.01% |
| corpus: radar-docs | +13.68% | +7.65% |
| corpus: railroad-docs | +5.87% | +13.91% |
| corpus: requirement-docs | +6.51% | +21.92% |
| corpus: sankey-docs | +21.39% | +4.70% |
| corpus: sequence-docs | +16.27% | +11.30% |
| corpus: state-docs | +23.87% | +21.27% |
| corpus: swimlane-docs | +24.12% | +12.29% |
| corpus: timeline-docs | +6.47% | +3.07% |
| corpus: treemap-docs | +1.60% | -4.38% |
| corpus: treeview-docs | +13.69% | -5.72% |
| corpus: usecase-docs | +17.68% | +28.43% |
| corpus: venn-docs | +10.48% | +3.03% |
| corpus: wardley-docs | +2.38% | +13.99% |
| corpus: xychart-docs | +0.07% | +0.89% |

[Extended comparison and every raw independent process](research/scanner-indexed-experiment-2026-10-20/extended-comparison.json) preserve the complete evidence. Neither the larger regressions nor the lower mesh median is assigned solely to the implementation or to external load.

## Release size and resources

Full features, identical locked dependencies, compiler, release/thin LTO/one codegen unit and source-path remapping; no stripping. File bytes and instruction text are measured separately:

| Release artifact | File before → candidate (bytes) | Text before → candidate (bytes) |
| --- | ---: | ---: |
| cli | 5,210,752 → 5,211,056 | 3,853,944 → 3,855,064 |
| ffi-static | 41,708,256 → 41,716,048 | 4,273,968 → 4,275,064 |
| ffi-dynamic | 4,798,544 → 4,799,152 | 3,561,420 → 3,562,020 |

Static-archive totals include unselected objects; full file bytes also include bitcode and other data. They are not the final host link increment. [Every release section and fingerprint](research/scanner-indexed-experiment-2026-10-20/size-comparison.json) and [separate development-driver sizes](research/scanner-indexed-experiment-2026-10-20/driver-sizes.json) identify the artifacts.

Sample allocation counts fell in **12 types**, with 21 unchanged; requested bytes fell in **19**, with 14 unchanged. Across 39 extended groups, counts fell in 12 and requested bytes in 25. Reallocations and additional-live peaks were unchanged throughout. Info allocations were **32 → 28** and requested bytes **4,379 → 4,039**. Every batch ended with zero net retained bytes; this is not a general leak audit or an RSS measurement. [Sample allocator data](research/scanner-indexed-experiment-2026-10-20/allocations-samples.json), [extended allocator data](research/scanner-indexed-experiment-2026-10-20/allocations-extended.json).

Separate OS resource checks used three fresh alternating processes per group/version, five warm-up batches and at least one second of complete rendering, including startup, input loading and warm-up in process totals. Sample median peak RSS was **8 lower / 0 unchanged / 25 higher**; extended groups were **8 lower / 5 unchanged / 26 higher**. The large labelled mesh rose **12,189,696 → 12,582,912 bytes (+384 KiB)** and ER Samples **4,440,064 → 4,702,208 (+256 KiB)**. This does not support a general RSS reduction. Peaks are not summed across processes. [Every Sample process](research/scanner-indexed-experiment-2026-10-20/resources-samples.json), [every extended process](research/scanner-indexed-experiment-2026-10-20/resources-extended.json).

## Correctness and acceptance

- Candidate full workspace: **3,231 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761/761 complete responses, 167/167 Sample SVGs and 39/39 extended verification records matched**.
- Four scanner tests cover small/large tables, 63/64/65-entry boundaries, final-slot indices, laziness, state, drops, panic and opener behavior. One inversion test compares the actual counter with an independent quadratic definition on six explicit and 2,048 seeded rows (`0x11c0de`). These five supplemental tests do not count as original ports.
- Original tests, assertions, expectations, random seeds, loop counts, thresholds and mappings remain unchanged. Existing token test bodies remained byte-identical.
- Original independent-timeout acceptance: **2,785 passed, zero failed, seven preserved skips, 4,034 unmapped, zero unverified**. Strict completeness exited one with `complete: false`; full original-test acceptance remains incomplete.

[Validation](research/scanner-indexed-experiment-2026-10-20/validation.json), [full summary and fingerprints](research/scanner-indexed-experiment-2026-10-20/summary.json), [CPU frames](research/scanner-indexed-experiment-2026-10-20/hot-frames.json). Current README/output benchmarks continue to describe the last committed production renderer; the contended candidate does not replace them.
