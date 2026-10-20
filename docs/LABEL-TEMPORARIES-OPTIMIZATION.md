# Direct construction of remaining SVG adapters

Across all **167 unchanged Samples and all 33 types**, paired native render time was **0.97956× the baseline (−2.04%)**. Documentation render was **0.96895× (−3.10%)**, with 29 types showing lower medians. Current reference speedup is **1.77× at API return** and **1.87× UTF-8 ready**. The **2× API-return target remains unmet**.

Baseline: `4a03a25c2ef74b393b283751cb04086edd3c2a81`, with identical production code at `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. Candidate: `2a3b5e01b99b96581194f433d637c6c9a5e3e3c7`. Reference: Pele 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1. [Full source, input and binary fingerprints](research/label-temporaries-optimization-2026-10-20/summary.json) identify every measured program.

## Profiling and implementation

Fresh captures covered Info, all Flowchart documentation, C4, Venn and Treeview. Info had **766 inclusive generic `Display::to_string` frame occurrences**, including repeated string growth below label writing. The candidate converts **62 known adapter calls in 31 files** to the existing private SVG `Part` implementation: 57 centered labels, three markers and two coordinate strings. Nonempty centered labels give an initial capacity hint of attribute bytes plus 96; empty labels give zero. The hint reserves storage only; complex markup grows normally and icons resolve during writing. Info's two labels share one output buffer. [Conversion inventory](research/label-temporaries-optimization-2026-10-20/conversion.json).

Afterward, Info's generic `to_string` occurrences were **zero** and shared label-writing occurrences **745 → 183**. C4's generic occurrences were **165 → 0**. Info root-wrapper occurrences rose **537 → 655** as a share of the captures; these overlapping inclusive counts include descendants and are not additive exclusive CPU percentages or isolated per-call latency measurements. [Before/after capture counts and raw capture hashes](research/label-temporaries-optimization-2026-10-20/hot-frames.json) retain the full observations.

Public helper signatures and `Display` behavior remain. Parsing, measurement callbacks, layout, floating-point conversion, full SVG output, limits and error paths retain their implementations. No source, model, label or output is cached. The earlier scanner/counter runtime experiments are absent; their four project scanner contracts remain independently tested.

## Timing, conditions and regressions

Three fresh processes per engine/version, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. All original inputs, Arial metrics, CSS output and fixed UTC clock remain. Builds, tests, CPU captures, allocator instrumentation and process-resource checks run separately from latency. No process run was discarded or replaced.

Timing was deferred while a listed external build/test driver was present. Both actual timing preflight and postflight then found **no listed driver**. These read-only snapshots bracket the runs; they do not prove sustained host idleness. [Exact snapshot scope and observations](research/label-temporaries-optimization-2026-10-20/timing-conditions.json).

The Sample native mean fell **2.04%**, with 21 types lower and 12 higher; Info fell **17.24%**, Mindmap **10.74%**. All regressions are retained below. The reference is measured again each round: the current 1.77× cannot be divided by the preceding 1.79× to infer a native regression. Small differences are not statistical-significance claims.

API speedups span **Agentflow 1.22× to Sankey 2.90×**: 23 types below 2×, ten at least 2×. An average alone does not mean every type reaches the target.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -2.49% | 1.62× | 1.61× |
| block | +0.06% | 1.36× | 1.39× |
| c4 | -4.26% | 1.55× | 1.57× |
| class | -1.35% | 1.54× | 1.60× |
| er | -1.73% | 1.47× | 1.48× |
| eventmodeling | -5.10% | 2.15× | 2.66× |
| gantt | -5.35% | 1.79× | 1.85× |
| git | +1.92% | 1.87× | 1.90× |
| info | -17.24% | 1.74× | 2.02× |
| journey | +0.96% | 1.89× | 2.01× |
| kanban | -0.38% | 1.42× | 1.45× |
| mindmap | -10.74% | 2.04× | 2.44× |
| packet | +0.04% | 2.00× | 2.07× |
| pie | -3.34% | 1.81× | 2.05× |
| quadrant | +0.61% | 1.63× | 1.72× |
| requirement | -0.12% | 1.59× | 1.63× |
| sankey | +1.65% | 2.90× | 3.29× |
| sequence | -4.56% | 1.66× | 1.72× |
| state | +2.56% | 1.65× | 1.73× |
| timeline | -5.11% | 2.14× | 2.31× |
| xychart | -0.66% | 1.59× | 1.66× |
| agentflow | -1.69% | 1.22× | 1.22× |
| architecture | +2.31% | 2.45× | 2.49× |
| cynefin | -4.80% | 1.86× | 2.40× |
| ishikawa | -1.16% | 1.92× | 2.00× |
| radar | +2.32% | 2.34× | 2.41× |
| railroad | +1.43% | 1.34× | 1.40× |
| swimlane | +0.23% | 1.39× | 1.40× |
| treemap | -4.18% | 2.11× | 2.18× |
| treeview | -0.09% | 2.10× | 2.29× |
| usecase | -1.96% | 1.40× | 1.36× |
| venn | +1.00% | 2.33× | 2.31× |
| wardley | -3.44% | 1.77× | 1.85× |

The current [full output-contract report](OUTPUT-BENCHMARK.md) links every raw Sample process. The four-engine common subset remains 110 Samples / 22 types and is reported independently in [its report](BENCHMARK.md).

All 39 extended workloads remain, including the large labelled mesh **+1.05%**, Flowchart documentation **+0.91%**, Venn **+1.23%**, and smaller Block/State regressions. The all-Flowchart batch fell 1.16%, which differs from the separate documentation group. Parse's documentation mean was **0.99853× (−0.15%)**; parsing was not modified and this is not attributed to a parsing optimization.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -1.16% | +0.47% |
| tree: 50 nodes | -0.54% | -0.63% |
| mesh: 50 nodes, 20 extra labelled edges | -1.51% | -0.51% |
| tree: 500 nodes | -1.84% | -0.49% |
| mesh: 500 nodes, 200 extra labelled edges | +1.05% | -2.20% |
| corpus: agentflow-docs | -3.21% | +0.18% |
| corpus: agentflow-reference | -1.23% | -0.39% |
| corpus: architecture-docs | -1.36% | -0.87% |
| corpus: block-docs | +0.20% | +0.92% |
| corpus: c4-docs | -6.33% | -1.08% |
| corpus: class-docs | -1.18% | +0.14% |
| corpus: cynefin-docs | -5.60% | -2.84% |
| corpus: er-docs | -3.35% | +0.59% |
| corpus: eventmodeling-docs | -4.86% | +0.78% |
| corpus: flowchart-docs | +0.91% | +0.36% |
| corpus: gantt-docs | -4.32% | -0.70% |
| corpus: git-docs | -0.60% | +0.93% |
| corpus: info-docs | -18.77% | +0.10% |
| corpus: ishikawa-docs | -0.11% | +0.66% |
| corpus: journey-docs | -1.66% | -0.07% |
| corpus: kanban-docs | -0.72% | -1.13% |
| corpus: mindmap-docs | -10.33% | -0.73% |
| corpus: packet-docs | -0.83% | +3.89% |
| corpus: pie-docs | -2.88% | +0.33% |
| corpus: quadrant-docs | -1.72% | -1.01% |
| corpus: radar-docs | -2.28% | -0.27% |
| corpus: railroad-docs | -0.90% | +0.02% |
| corpus: requirement-docs | -2.07% | -0.84% |
| corpus: sankey-docs | -0.27% | +0.39% |
| corpus: sequence-docs | -4.66% | +0.60% |
| corpus: state-docs | +0.21% | -0.06% |
| corpus: swimlane-docs | -0.48% | -2.47% |
| corpus: timeline-docs | -8.80% | +1.15% |
| corpus: treemap-docs | -1.12% | -5.66% |
| corpus: treeview-docs | -0.88% | +0.09% |
| corpus: usecase-docs | -2.13% | -0.07% |
| corpus: venn-docs | +1.23% | +1.37% |
| corpus: wardley-docs | -9.16% | -0.22% |
| corpus: xychart-docs | -1.58% | +1.06% |

[Every extended row and raw independent process](research/label-temporaries-optimization-2026-10-20/extended-comparison.json) is retained.

## Compiled size and allocator tradeoffs

Same locked dependencies, compiler, full features, release/thin LTO/one codegen unit and normalized source paths; no stripping. All 33 types stay enabled.

| Artifact | File before → after (bytes) | Instruction text before → after (bytes) |
| --- | ---: | ---: |
| cli | 5,210,752 → 5,194,400 | 3,853,944 → 3,845,164 |
| ffi-static | 41,708,256 → 41,710,992 | 4,273,968 → 4,273,740 |
| ffi-dynamic | 4,798,544 → 4,798,784 | 3,561,420 → 3,561,128 |

CLI file bytes fall **16,352**, instruction text **8,780**. FFI instruction text falls **292 / 228 bytes**, while its file totals rise **240 / 2,736 bytes**. File size includes data, symbols, metadata, bitcode and archive overhead. Static-archive text sums all members before host linker selection/dead stripping, not the final host link increment. [Every release section](research/label-temporaries-optimization-2026-10-20/size-comparison.json) and [separate driver sizes](research/label-temporaries-optimization-2026-10-20/driver-sizes.json) are retained.

Allocator probes run separately after five warm-up batches, counting one complete batch and excluding inputs/options. Full realloc requests count as requested bytes. Results are dropped every iteration.

| Metric | Samples lower / equal / higher | Extended lower / equal / higher |
| --- | ---: | ---: |
| Allocations | 1 / 32 / 0 | 1 / 38 / 0 |
| Reallocations | 23 / 10 / 0 | 27 / 12 / 0 |
| Requested bytes | 20 / 10 / 3 | 24 / 12 / 3 |
| Additional-live peak | 3 / 29 / 1 | 2 / 36 / 1 |

Info Samples/documentation changed **32 → 30 allocations**, **7 → 1 reallocations**, **4,379 → 3,827 requested bytes**, but additional-live peak **1,182 → 1,215 bytes**. Flowchart Samples' reallocations fell **663 → 610**. All three requested-byte regressions remain: Journey **44,389 → 44,458**, Radar **81,195 → 82,259**, Venn Samples **138,057 → 138,411** and Venn documentation **165,743 → 166,097 bytes**. Every batch ended with zero net retained bytes; this is not a general leak audit. [All Sample allocator data](research/label-temporaries-optimization-2026-10-20/allocations-samples.json), [all extended data](research/label-temporaries-optimization-2026-10-20/allocations-extended.json).

## OS process resources

Three fresh alternating processes per group/version, five warm-up batches, then at least one second of complete rendering. Peak RSS and process CPU totals include startup, input loading and warm-up; they are separate from render-only allocator counts and latency. Success/rejection counts and SVG byte totals match in each batch. Per-process peaks are not summed and requested bytes are not RSS.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 6 / 2 / 25 |
| extended | 3 / 1 / 35 |

Largest observed increases: Samples: cynefin 4,030,464 → 4,194,304 bytes; Treeview 4,096,000 → 4,243,456 bytes; Kanban 3,981,312 → 4,112,384 bytes; Extended: requirement-docs 4,882,432 → 5,177,344 bytes; corpus: architecture-docs 4,653,056 → 4,915,200 bytes; corpus: sankey-docs 4,636,672 → 4,882,432 bytes. Small page/allocator variations are not causal claims. [Every Sample process](research/label-temporaries-optimization-2026-10-20/resources-samples.json), [every extended process](research/label-temporaries-optimization-2026-10-20/resources-extended.json).

## Correctness and original acceptance

- Full candidate workspace: **3,236 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761/761 complete responses, 167/167 original Sample SVGs and 39/39 extended verification records match**.
- Additional non-timed primary-driver checks render 879 overlapping extended inputs with Arial 16/CSS: **820 successes, 59 expected rejections**, all reference outcomes and successful SVGs match. These checks supplement the standard extended benchmark; they do not increase original-test acceptance.
- Six original font/root-style cases are individually ported, preserving **37 original assertion conditions, 19 render calls**, all original loop inputs, expected values and per-test timeouts. The seventh safety/XML-helper case remains unmapped. Four project scanner lifecycle/boundary tests do not count as upstream ports.
- Both the runtime candidate and a separate test-only prospective-commit snapshot pass 3,236 workspace tests and the strict original gate. Existing assertions, expected values, seeds, loop counts and thresholds remain unchanged. Original test source/fixture inventories are unchanged; mappings only append the six actual tests.
- Strict original acceptance: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Completeness exits one with `complete: false`; full original-test acceptance is unfinished.

[Validation and source provenance](research/label-temporaries-optimization-2026-10-20/validation.json). After the test-only ports, rebuilding all four frozen drivers and three shipping artifacts produced identical bytes. The original artifact-source snapshots are preserved, with the exact test-only source supplement and [rebuild proof](research/label-temporaries-optimization-2026-10-20/release-revalidation.json) recorded separately.
