# Token ownership and one-pass usecase publication

Across all **167 unchanged Samples and all 33 types**, paired native rendering time is **0.99449× the baseline (-0.55%)**. Current speedup relative to the pinned reference is **1.78× at API return**, **1.90× UTF-8 ready**. The **2× API-return target remains unmet**.

Baseline source: `7235d3f5a7ba881a43796c82778617df54b618f7`, with the same production code as `2a3b5e01b99b96581194f433d637c6c9a5e3e3c7`. Candidate: `9de7ca548364b364d4d0f5149af48c1eacd242bc`. Pele 1.7.0 remains fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1. [Full source, input and artifact fingerprints](research/parser-ownership-optimization-2026-10-20/summary.json) identify both programs.

## Profiling and implementation

CPU-only primary-contract captures use the original Sample inputs, Arial 16, CSS and the fixed clock. Five slower types were captured: Agentflow, Block, Railroad, Swimlane and Usecase. Inclusive frames showed Agentflow parser/vertex work, Flowchart parsing used by Swimlane, and Usecase resolution. Each sampler batch's input count and SVG byte total matches the independently verified primary driver. The same sampler source, inputs and options are used in both phases. No latency statistics are collected in the sampler. [Captures and sampler provenance](research/parser-ownership-optimization-2026-10-20/cpu-before.json).

Flowchart's private parser now retains the lexer's `Cow<str>` tokens instead of immediately allocating every token. Consumed text moves into owned semantic values; subsequent fragments append without temporary clones. Agentflow likewise moves consumed owned strings. Skipped punctuation/whitespace need not be copied into semantic strings. Public lexer/model APIs and owned parsed results remain unchanged. Token precedence, UTF-16 diagnostics, source spans, parser branches and loop counts are retained.

Usecase resolution keeps mutable semantic states through all metadata checks, validates them in the original insertion order, and then moves them once into public maps sorted by first mention, stably for ties. Classes/styles still apply afterward. This removes intermediate publication, cloned model reordering, and final republishing. Errors retain their original precedence and position; public model fields/types, duplicate handling and map order remain. No source, model, token or SVG result is cached.

A new independent pinned-Pele semantic fixture covers **161 inputs: 132 successful models and 29 expected rejections**, including forward references, repeated declarations, class/style application, metadata and competing invalid conditions. It projects the affected public ordered maps and metadata, not unchanged JSON payload arenas or statement/source trees. Expected values come only from the pinned Node implementation, with fixed seed `0x4a0e14` and 128 generated cases; the generator rejects initialization/projection errors. This supplementary regression does not increase original-test acceptance.

## Timing and all regressions

Three fresh processes per engine/version, alternating order; five warm-ups, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Every original input and full parse/layout/SVG operation remains. Builds, tests, CPU captures, allocator probes and process-resource checks are separate from latency. No raw process run is discarded or replaced. Preflight/postflight found no listed build/test driver, but these limited snapshots do not prove sustained host idleness. [Conditions](research/parser-ownership-optimization-2026-10-20/timing-conditions.json).

The full-type Sample median falls in **18 types** and rises in **15**. These small overall differences are not statistical-significance claims. Parsing benefits in the four affected diagram types do not replace the full 33-type target. Node is remeasured each round; cross-round reference ratios do not establish a native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -1.91% | 1.62× | 1.71× |
| block | +0.25% | 1.32× | 1.37× |
| c4 | +1.54% | 1.52× | 1.55× |
| class | -0.87% | 1.57× | 1.59× |
| er | -1.30% | 1.46× | 1.49× |
| eventmodeling | -3.43% | 2.21× | 2.69× |
| gantt | +0.05% | 1.76× | 1.80× |
| git | +1.79% | 1.89× | 1.90× |
| info | +0.73% | 1.66× | 2.03× |
| journey | +1.04% | 1.90× | 2.02× |
| kanban | +0.58% | 1.42× | 1.44× |
| mindmap | -0.23% | 2.05× | 2.40× |
| packet | -3.01% | 2.02× | 2.10× |
| pie | +0.04% | 1.81× | 2.05× |
| quadrant | +0.13% | 1.64× | 1.75× |
| requirement | +1.72% | 1.59× | 1.62× |
| sankey | -0.58% | 2.78× | 3.32× |
| sequence | -0.10% | 1.68× | 1.76× |
| state | -0.70% | 1.68× | 1.76× |
| timeline | -1.04% | 2.17× | 2.37× |
| xychart | -0.97% | 1.59× | 1.65× |
| agentflow | -2.76% | 1.24× | 1.27× |
| architecture | -1.57% | 2.56× | 2.64× |
| cynefin | +1.16% | 1.86× | 2.39× |
| ishikawa | +1.27% | 1.91× | 1.94× |
| radar | -0.17% | 2.45× | 2.57× |
| railroad | +0.09% | 1.38× | 1.43× |
| swimlane | -4.49% | 1.44× | 1.47× |
| treemap | -2.05% | 2.17× | 2.24× |
| treeview | -0.99% | 2.12× | 2.37× |
| usecase | -3.86% | 1.40× | 1.43× |
| venn | +0.32% | 2.31× | 2.37× |
| wardley | +1.59% | 1.78× | 1.84× |

The current [output-contract report](OUTPUT-BENCHMARK.md) retains all original Samples and raw independent processes. API and UTF-8 ready are separate endpoints. The four-engine common subset remains independently reported in [BENCHMARK.md](BENCHMARK.md).

The 33-type documentation means are render **0.99816× (-0.18%)**, parse **0.98257× (-1.74%)**. Every extended load and observed regression is retained:

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -4.55% | -12.05% |
| tree: 50 nodes | -7.57% | -14.27% |
| mesh: 50 nodes, 20 extra labelled edges | -0.89% | -9.82% |
| tree: 500 nodes | -6.06% | -13.65% |
| mesh: 500 nodes, 200 extra labelled edges | +1.43% | -8.71% |
| corpus: agentflow-docs | -2.86% | -5.31% |
| corpus: agentflow-reference | -3.21% | -7.12% |
| corpus: architecture-docs | +0.56% | -1.32% |
| corpus: block-docs | +1.06% | -1.48% |
| corpus: c4-docs | +1.15% | +1.22% |
| corpus: class-docs | +2.01% | -1.42% |
| corpus: cynefin-docs | +0.75% | -0.35% |
| corpus: er-docs | +1.19% | +0.06% |
| corpus: eventmodeling-docs | -0.85% | -1.32% |
| corpus: flowchart-docs | -5.39% | -10.46% |
| corpus: gantt-docs | +0.60% | -0.42% |
| corpus: git-docs | -0.55% | -1.71% |
| corpus: info-docs | -1.15% | -2.64% |
| corpus: ishikawa-docs | +1.70% | -2.88% |
| corpus: journey-docs | +0.71% | -0.72% |
| corpus: kanban-docs | +0.24% | -0.59% |
| corpus: mindmap-docs | +0.23% | -0.46% |
| corpus: packet-docs | -0.62% | -0.96% |
| corpus: pie-docs | +0.90% | -0.89% |
| corpus: quadrant-docs | +0.66% | +1.44% |
| corpus: radar-docs | +3.91% | +2.78% |
| corpus: railroad-docs | -0.58% | +1.34% |
| corpus: requirement-docs | +1.94% | -0.96% |
| corpus: sankey-docs | -0.28% | -0.19% |
| corpus: sequence-docs | -2.31% | -1.19% |
| corpus: state-docs | -1.76% | -0.02% |
| corpus: swimlane-docs | -4.12% | -10.67% |
| corpus: timeline-docs | +0.68% | -1.96% |
| corpus: treemap-docs | +0.81% | -1.52% |
| corpus: treeview-docs | -0.21% | +2.44% |
| corpus: usecase-docs | -6.50% | -13.50% |
| corpus: venn-docs | +0.65% | -0.05% |
| corpus: wardley-docs | +1.12% | -0.33% |
| corpus: xychart-docs | +0.96% | -1.32% |

[Complete extended comparison and raw processes](research/parser-ownership-optimization-2026-10-20/extended-comparison.json).

## Compiled code and allocations

Same compiler, locked dependencies, release/thin LTO/one codegen unit, source-path remapping and all 33 types; no stripping. File size and instruction text are separate metrics.

| Artifact | File before → after (bytes) | Instruction text before → after (bytes) |
| --- | ---: | ---: |
| cli | 5,194,400 → 5,160,832 | 3,845,164 → 3,800,608 |
| ffi-static | 41,710,992 → 41,545,872 | 4,273,740 → 4,230,436 |
| ffi-dynamic | 4,798,784 → 4,765,824 | 3,561,128 → 3,517,824 |

All three shipping files and instruction-text totals fall in this round. Static-archive text sums all members before host linker selection/dead stripping; file totals include bitcode, symbols and other data. Neither is the host application's final link increment. [Every shipping section and hash](research/parser-ownership-optimization-2026-10-20/size-comparison.json), [separate development drivers](research/parser-ownership-optimization-2026-10-20/driver-sizes.json).

Allocator measurements use five warm-up batches then one complete counted batch, excluding inputs/options and dropping each result. Requested bytes include full realloc requests and are not RSS.

| Metric | Samples lower / equal / higher | Extended lower / equal / higher |
| --- | ---: | ---: |
| Allocations | 4 / 29 / 0 | 10 / 29 / 0 |
| Reallocations | 0 / 32 / 1 | 1 / 37 / 1 |
| Requested bytes | 4 / 29 / 0 | 10 / 29 / 0 |
| Additional-live peak | 0 / 33 / 0 | 0 / 39 / 0 |

Sample allocation counts: Flowchart **5,439 → 5,073**, Agentflow **7,754 → 7,175**, Swimlane **4,904 → 4,438**, Usecase **5,668 → 5,255**. Sample requested bytes fall in those four types, with all 29 others unchanged. Additional-live peaks are unchanged in every Sample/extended group. Usecase reallocations rise **913 → 917** for Samples and **2,280 → 2,302** for documentation; Agentflow reference reallocations fall **9,234 → 9,228**. All batch-end net retained bytes are zero, which is not a general leak audit. [Sample allocator rows](research/parser-ownership-optimization-2026-10-20/allocations-samples.json), [all extended rows](research/parser-ownership-optimization-2026-10-20/allocations-extended.json).

## OS process resources

Three fresh alternating processes per group/version, five warm-ups and at least one second of complete rendering. OS peak RSS and CPU totals include startup, input loading and warm-up; these are separate from render-only allocator metrics and latency. Every batch's success/rejection counts and SVG byte totals match. Independent process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 12 / 2 / 19 |
| extended | 21 / 0 / 18 |

Largest observed increases: samples: flowchart 4,800,512 → 5,013,504 bytes; samples: swimlane 4,620,288 → 4,816,896 bytes; samples: agentflow 4,849,664 → 4,980,736 bytes; tree: 500 nodes 8,077,312 → 8,519,680 bytes; mesh: 50 nodes, 20 extra labelled edges 5,423,104 → 5,799,936 bytes; corpus: swimlane-docs 5,013,504 → 5,275,648 bytes. All per-process observations remain in the [Sample report](research/parser-ownership-optimization-2026-10-20/resources-samples.json) and [extended report](research/parser-ownership-optimization-2026-10-20/resources-extended.json). Small page/allocator variations are not assigned a causal interpretation.

## Correctness and acceptance

- Full workspace: **3,237 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761/761 complete response comparisons, 167/167 Sample SVGs and 39/39 extended verification records match**.
- The 161-case independent semantic regression passes, with all 132 models and all 29 failures checked. Existing Flowchart tests and original Node token/label/number reference fixtures remain enabled and unchanged.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness still exits one with `complete: false`; full original-test acceptance is unfinished.
- Existing source fixtures, expected values, assertions, loop counts, seeds, performance thresholds and test mappings are unchanged. The new semantic fixture is independent supplementary coverage, not a generated Rust expected value or an original-test mapping.

[Validation](research/parser-ownership-optimization-2026-10-20/validation.json) and all fingerprints remain alongside raw measurement evidence. The optimization keeps the full API workload and output precision.
