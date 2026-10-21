# Lazy tokenizer state

Across all **167 original Samples / 33 types**, paired native render time is **0.99508× (-0.49%)**. The independently remeasured reference speedup is **API 1.89×**, **UTF-8 ready 1.98×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `68a20d8e68f51881f0e697f7fe08bb1219d1d12e` (runtime `528010f970b8b9b54d94d88275c43e96f4b32015`), candidate `b85c7c10bcc0db77d4758f7ad0c4a029257a56fe`. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/lazy-token-state-optimization-2026-10-21/summary.json).

This is the final retained snapshot before continuous optimization was stopped at the user’s request. The per-scan allocation reduction is retained with its small observed time benefit and explicit instruction-text/RSS tradeoffs. It does not establish the 2× target or a compiled-size reduction.

## Measured scope and implementation

Fresh before/after CPU captures cover eight selected types / 36 original inputs: Info, Packet, Radar, Pie, GitGraph, Railroad, TreeView and Wardley. The full timing collection remains **all 167 original Samples / 33 types**. Before captures show token writing/growth and allocation descendants under the shared tokenizer. Counts are inclusive, overlapping and dependent on inlining, not additive exclusive CPU percentages. [Before](research/lazy-token-state-optimization-2026-10-21/cpu-before.json), [after](research/lazy-token-state-optimization-2026-10-21/cpu-after.json), [target frames](research/lazy-token-state-optimization-2026-10-21/hot-frames.json), [trait inventory](research/lazy-token-state-optimization-2026-10-21/trait-frames.json). [Baseline allocator descendants under tokenizer subtrees](research/lazy-token-state-optimization-2026-10-21/tokenizer-allocation-frames-before.json) preserve the measured reason for targeting scan-local state; they do not assign every allocation exclusively to that state.

Baseline shipping CLI and dynamic-library function intervals were also attributed from their frozen Mach-O function-start tables and symbols, and their total text bytes matched the independent section reader. Intervals include alignment, aliases and unnamed locals; they are not disassembled exclusive instruction sizes or exclusive CPU costs. [CLI](research/lazy-token-state-optimization-2026-10-21/text-attribution-cli.json), [dynamic library](research/lazy-token-state-optimization-2026-10-21/text-attribution-ffi-dynamic.json), [reader source](research/lazy-token-state-optimization-2026-10-21/attribute_text.py).

## Local state and public dynamic tables

Only `diagrams/common/tokens.rs` changes at runtime. Both scratch arrays keep their original full-index `Vec` representation and start empty. A missing disabled-pattern slot means false; the flag array is materialized when an opener first disables a type. Scanner slots are materialized when a `Pattern::Scanner` is first tried. Slots still use original kind indices, with each factory called on its original first attempt. There is no fixed-width bit-set or separate overflow owner.

State is local to the scan and dropped at its end. Existing matcher functions, hidden tokens, first-match priority, LONGER_ALT, opener suppression, EOF, output vectors, public dynamic-table kind range and precise errors remain unchanged. No state is reused for another source or operation.

Two supplementary public-API tests cover 38 handwritten tokenization calls / 12 expected rejections: direct and longer-alternative disabling around 64/128/256 boundaries through kind 1,023; exact tokens, byte spans and error positions; lazy factories, state reuse within one scan and reset across scans. The unmodified production implementation passed these tests before the runtime edit. The existing independent Node token reference retains all **2,008 cases / 15 tables**, seed and mutation counts unchanged. These are not new original-test ports. [Scope and fingerprints](research/lazy-token-state-optimization-2026-10-21/changes.json), [baseline handwritten-contract validation](research/lazy-token-state-optimization-2026-10-21/baseline-supplementary.json), [supplementary source](research/lazy-token-state-optimization-2026-10-21/token_workspace.rs).

Full parsing, layout and SVG output, workloads, precision and diagram features remain. No dependency, toolchain/release-profile change, unsafe code, preparse or returned cache is introduced. Baseline artifacts and inputs were frozen and all source/binary hashes checked before editing. [Baseline check](research/lazy-token-state-optimization-2026-10-21/baseline-check.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/lazy-token-state-optimization-2026-10-21/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | +0.37% | 1.68× | 1.72× |
| block | +0.27% | 1.40× | 1.44× |
| c4 | +0.09% | 1.58× | 1.63× |
| class | +1.48% | 1.71× | 1.66× |
| er | -0.38% | 1.47× | 1.52× |
| eventmodeling | -3.11% | 2.33× | 2.95× |
| gantt | -0.18% | 1.77× | 1.81× |
| git | -1.03% | 1.97× | 2.03× |
| info | -6.12% | 2.06× | 2.50× |
| journey | +0.09% | 2.06× | 2.11× |
| kanban | +0.66% | 1.44× | 1.49× |
| mindmap | +0.96% | 2.15× | 2.50× |
| packet | -0.95% | 2.28× | 2.34× |
| pie | +0.48% | 1.89× | 2.12× |
| quadrant | +1.22% | 1.68× | 1.80× |
| requirement | -0.33% | 1.67× | 1.71× |
| sankey | -1.46% | 3.00× | 3.41× |
| sequence | +0.96% | 1.77× | 1.84× |
| state | -0.06% | 1.79× | 1.72× |
| timeline | +1.78% | 2.31× | 2.48× |
| xychart | +0.38% | 1.64× | 1.72× |
| agentflow | +1.21% | 1.29× | 1.30× |
| architecture | -1.57% | 2.66× | 2.70× |
| cynefin | -0.58% | 1.98× | 2.57× |
| ishikawa | +0.25% | 1.99× | 2.06× |
| radar | -2.32% | 2.56× | 2.63× |
| railroad | -0.47% | 1.44× | 1.46× |
| swimlane | -0.39% | 1.48× | 1.52× |
| treemap | -5.91% | 2.46× | 2.34× |
| treeview | -2.75% | 2.24× | 2.45× |
| usecase | -0.07% | 1.52× | 1.48× |
| venn | +1.84% | 2.41× | 2.41× |
| wardley | -0.04% | 1.87× | 1.97× |

API speedup ranges from 1.29× (agentflow, slowest relative to Pele) to 3.00× (sankey); 21 of 33 types remain below 2×. An aggregate target does not imply every type reaches it.

[Complete output-contract timings](OUTPUT-BENCHMARK.md) preserve all Sample process reports. The [four-engine comparison](BENCHMARK.md) has its own 110-input / 22-type common-success subset; it does not replace the primary collection.

Across the 33 documentation types, render is **-0.38%**, parse **-2.04%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -0.34% | +0.22% |
| tree: 50 nodes | +0.61% | +0.00% |
| mesh: 50 nodes, 20 extra labelled edges | +0.10% | +0.87% |
| tree: 500 nodes | +1.01% | -0.42% |
| mesh: 500 nodes, 200 extra labelled edges | -2.34% | +1.41% |
| corpus: agentflow-docs | +0.28% | -0.27% |
| corpus: agentflow-reference | +0.16% | +2.06% |
| corpus: architecture-docs | -1.12% | -2.25% |
| corpus: block-docs | -0.01% | -0.96% |
| corpus: c4-docs | +2.20% | -0.22% |
| corpus: class-docs | +7.29% | +0.42% |
| corpus: cynefin-docs | +5.02% | -3.28% |
| corpus: er-docs | -0.59% | -0.02% |
| corpus: eventmodeling-docs | -2.09% | -5.29% |
| corpus: flowchart-docs | -3.06% | +0.59% |
| corpus: gantt-docs | -0.82% | +0.03% |
| corpus: git-docs | -2.26% | -4.66% |
| corpus: info-docs | -3.60% | -10.81% |
| corpus: ishikawa-docs | +1.37% | -2.88% |
| corpus: journey-docs | +0.49% | -0.46% |
| corpus: kanban-docs | +2.55% | +0.24% |
| corpus: mindmap-docs | +0.44% | -1.62% |
| corpus: packet-docs | -1.30% | -2.62% |
| corpus: pie-docs | -0.74% | -0.02% |
| corpus: quadrant-docs | -0.87% | -1.65% |
| corpus: radar-docs | -2.56% | -7.35% |
| corpus: railroad-docs | -1.70% | -1.87% |
| corpus: requirement-docs | +2.02% | -0.67% |
| corpus: sankey-docs | +0.65% | +0.50% |
| corpus: sequence-docs | +0.65% | +0.25% |
| corpus: state-docs | -0.46% | -1.06% |
| corpus: swimlane-docs | +0.45% | +0.79% |
| corpus: timeline-docs | -0.46% | -0.07% |
| corpus: treemap-docs | -4.86% | -16.04% |
| corpus: treeview-docs | -2.61% | -4.36% |
| corpus: usecase-docs | +0.18% | +0.09% |
| corpus: venn-docs | -4.72% | -0.26% |
| corpus: wardley-docs | -0.32% | +1.36% |
| corpus: xychart-docs | -1.17% | -0.56% |

[Every extended process and verification record](research/lazy-token-state-optimization-2026-10-21/extended-comparison.json).

| Rejection workload | Rejection time change |
| --- | ---: |
| corpus: agentflow-reference | -3.40% |


## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,160,080 → 5,160,080 | +0 | 3,797,408 → 3,797,916 | +508 |
| ffi-static | 41,590,632 → 41,593,080 | +2,448 | 4,228,952 → 4,229,256 | +304 |
| ffi-dynamic | 4,766,240 → 4,766,544 | +304 | 3,516,308 → 3,516,592 | +284 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/lazy-token-state-optimization-2026-10-21/size-comparison.json), [development drivers separately](research/lazy-token-state-optimization-2026-10-21/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 12 / 21 / 0 |
| Primary Arial16 Samples | reallocations | 0 / 33 / 0 |
| Primary Arial16 Samples | requestedBytes | 12 / 21 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default Samples | allocations | 12 / 21 / 0 |
| Default Samples | reallocations | 0 / 33 / 0 |
| Default Samples | requestedBytes | 12 / 21 / 0 |
| Default Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default extended | allocations | 12 / 27 / 0 |
| Default extended | reallocations | 0 / 39 / 0 |
| Default extended | requestedBytes | 12 / 27 / 0 |
| Default extended | peakAdditionalLiveBytes | 0 / 39 / 0 |

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/lazy-token-state-optimization-2026-10-21/allocations-primary.json), [probe source](research/lazy-token-state-optimization-2026-10-21/primary_allocations.rs), [default Samples](research/lazy-token-state-optimization-2026-10-21/allocations-samples.json), [default extended](research/lazy-token-state-optimization-2026-10-21/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 14 / 3 / 16 |
| extended | 15 / 4 / 20 |

Largest samples increases: samples: gantt 4,407,296 → 4,554,752 bytes; samples: mindmap 4,096,000 → 4,194,304 bytes; samples: flowchart 4,898,816 → 4,980,736 bytes.

Largest extended increases: mesh: 500 nodes, 200 extra labelled edges 11,452,416 → 15,155,200 bytes; corpus: quadrant-docs 4,390,912 → 4,521,984 bytes; corpus: venn-docs 4,472,832 → 4,571,136 bytes.

[All Sample processes](research/lazy-token-state-optimization-2026-10-21/resources-samples.json), [all extended processes](research/lazy-token-state-optimization-2026-10-21/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Complete workspace: **3,245 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- The two supplementary handwritten contracts pass before and after the runtime change. All existing independent error, location, parser, numeric, label, root and response assertions remain. No original expected value is generated or changed. Rejection timings are reported separately; a successful-input improvement does not excuse a rejection or original-timeout regression.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/lazy-token-state-optimization-2026-10-21/validation.json). The user stopped continuous optimization after this snapshot. The 2× API target and full original-test acceptance remain unmet; no further optimization round starts automatically.
