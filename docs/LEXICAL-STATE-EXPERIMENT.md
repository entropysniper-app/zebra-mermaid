# Tokenizer bit-set scratch experiment

**This candidate was not retained.** Allocation counts and requested bytes decrease for 12 types, but the observed native-time reduction is small, all three shipping instruction-text totals increase, and OS RSS rises for 29 of 33 Sample groups. The Sample median RSS delta is +81,920 bytes, maximum +196,608 bytes; these process measurements can vary with page/layout/allocator effects and are not a proven causal attribution to the bit-set. Production remains `528010f970b8b9b54d94d88275c43e96f4b32015`, API 1.85× / UTF-8 ready 1.97×. The 2× API-return and compiled-size objectives remain active.

Across all **167 original Samples / 33 types**, paired native render time is **0.99650× (-0.35%)**. The independently remeasured reference speedup is **API 1.86×**, **UTF-8 ready 1.98×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `542f6cd265d0b775b130a626ccd994c2b9e9d006` (runtime `528010f970b8b9b54d94d88275c43e96f4b32015`), candidate preserved in an [exact source/test patch](research/lexical-state-experiment-2026-10-21/candidate.patch.gz), not a production revision. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/lexical-state-experiment-2026-10-21/summary.json).

## Measured scope and implementation

Fresh before/after CPU captures cover eight selected types / 36 original inputs: Info, Packet, Radar, Pie, GitGraph, Railroad, TreeView and Wardley. The full timing collection remains **all 167 original Samples / 33 types**. Before captures show token writing/growth and allocation descendants under the shared tokenizer. Counts are inclusive, overlapping and dependent on inlining, not additive exclusive CPU percentages. [Before](research/lexical-state-experiment-2026-10-21/cpu-before.json), [after](research/lexical-state-experiment-2026-10-21/cpu-after.json), [target frames](research/lexical-state-experiment-2026-10-21/hot-frames.json), [trait inventory](research/lexical-state-experiment-2026-10-21/trait-frames.json). [Baseline allocator descendants under tokenizer subtrees](research/lexical-state-experiment-2026-10-21/tokenizer-allocation-frames-before.json) preserve the measured reason for targeting scan-local state; they do not assign every allocation exclusively to that state.

Baseline shipping CLI and dynamic-library function intervals were also attributed from their frozen Mach-O function-start tables and symbols, and their total text bytes matched the independent section reader. Intervals include alignment, aliases and unnamed locals; they are not disassembled exclusive instruction sizes or exclusive CPU costs. [CLI](research/lexical-state-experiment-2026-10-21/text-attribution-cli.json), [dynamic library](research/lexical-state-experiment-2026-10-21/text-attribution-ffi-dynamic.json), [reader source](research/lexical-state-experiment-2026-10-21/attribute_text.py).

## Local state and public dynamic tables

Only `diagrams/common/tokens.rs` changes at runtime. The first 64 disabled-pattern flags use one local `u64`; token kinds beyond that range keep lazily allocated overflow flags. All 15 fixed grammar tables currently have at most 47 kinds. Public dynamic tables retain the complete native kind range. Overflow storage is allocated when an opener first disables a kind beyond 63.

The scanner-slot vector is created only when a `Pattern::Scanner` is actually tried. Its slots still use the original kind indices, and each factory is called on its original first attempt. State is owned by that scan and dropped at its end. It is never retained for another source or operation. Existing matcher functions, hidden-token behavior, first-match priority, LONGER_ALT, opener suppression, EOF, output vectors and precise errors remain unchanged.

Two supplementary public-API tests cover 38 handwritten tokenization calls / 12 expected rejections: direct and longer-alternative disabling around 64/128/256 boundaries through kind 1,023; exact tokens, byte spans and error positions; lazy factories, state reuse within one scan and reset across scans. The unmodified production implementation passed these tests before the runtime edit. The existing independent Node token reference retains all **2,008 cases / 15 tables**, seed and mutation counts unchanged. These are not new original-test ports. [Scope and fingerprints](research/lexical-state-experiment-2026-10-21/changes.json), [baseline handwritten-contract validation](research/lexical-state-experiment-2026-10-21/baseline-supplementary.json), [supplementary source](research/lexical-state-experiment-2026-10-21/token_workspace.rs).

Full parsing, layout and SVG output, workloads, precision and diagram features remain. No dependency, toolchain/release-profile change, unsafe code, preparse or returned cache is introduced. Baseline artifacts and inputs were frozen and all source/binary hashes checked before editing. [Baseline check](research/lexical-state-experiment-2026-10-21/baseline-check.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/lexical-state-experiment-2026-10-21/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -0.20% | 1.68× | 1.71× |
| block | -0.71% | 1.38× | 1.47× |
| c4 | +0.72% | 1.57× | 1.60× |
| class | +0.84% | 1.55× | 1.59× |
| er | -0.51% | 1.50× | 1.54× |
| eventmodeling | -1.67% | 2.35× | 2.90× |
| gantt | -0.15% | 1.81× | 1.86× |
| git | -0.24% | 1.98× | 2.07× |
| info | -3.03% | 2.06× | 2.39× |
| journey | -0.88% | 2.04× | 2.16× |
| kanban | +0.51% | 1.46× | 1.52× |
| mindmap | -2.07% | 2.27× | 2.50× |
| packet | -1.23% | 2.26× | 2.35× |
| pie | -0.14% | 1.89× | 2.16× |
| quadrant | +2.42% | 1.69× | 1.78× |
| requirement | -0.24% | 1.68× | 1.74× |
| sankey | +0.23% | 2.83× | 3.42× |
| sequence | -0.60% | 1.71× | 1.76× |
| state | -0.28% | 1.70× | 1.73× |
| timeline | +0.27% | 2.31× | 2.53× |
| xychart | +0.14% | 1.70× | 1.71× |
| agentflow | -1.30% | 1.29× | 1.30× |
| architecture | -1.60% | 2.65× | 2.71× |
| cynefin | +0.22% | 1.99× | 2.53× |
| ishikawa | +1.43% | 1.95× | 2.03× |
| radar | -0.98% | 2.50× | 2.60× |
| railroad | -1.17% | 1.38× | 1.44× |
| swimlane | +2.01% | 1.45× | 1.45× |
| treemap | -0.80% | 2.28× | 2.31× |
| treeview | +0.10% | 2.17× | 2.37× |
| usecase | -0.78% | 1.44× | 1.47× |
| venn | -0.33% | 2.36× | 2.39× |
| wardley | -1.35% | 1.89× | 1.95× |

API speedup ranges from 1.29× (agentflow, slowest relative to Pele) to 2.83× (sankey); 21 of 33 types remain below 2×. An aggregate target does not imply every type reaches it.

[Candidate output-contract timings](research/lexical-state-experiment-2026-10-21/output/comparison.json) preserve all Sample process reports in the same directory. The [candidate four-engine comparison](research/lexical-state-experiment-2026-10-21/four-engine/comparison.json) has its own 110-input / 22-type common-success subset; it does not replace the primary collection. Published production comparisons and README remain unchanged.

Across the 33 documentation types, render is **-0.21%**, parse **-0.45%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | +0.45% | -1.51% |
| tree: 50 nodes | -0.82% | -0.70% |
| mesh: 50 nodes, 20 extra labelled edges | +0.82% | +1.02% |
| tree: 500 nodes | +0.86% | +0.32% |
| mesh: 500 nodes, 200 extra labelled edges | -1.26% | -0.27% |
| corpus: agentflow-docs | +0.11% | +0.16% |
| corpus: agentflow-reference | +0.11% | -0.41% |
| corpus: architecture-docs | -0.10% | -0.32% |
| corpus: block-docs | -0.37% | +0.19% |
| corpus: c4-docs | -0.01% | -0.03% |
| corpus: class-docs | +0.06% | +1.04% |
| corpus: cynefin-docs | -1.66% | -2.64% |
| corpus: er-docs | -1.36% | +0.66% |
| corpus: eventmodeling-docs | +0.23% | -1.34% |
| corpus: flowchart-docs | -1.86% | -1.55% |
| corpus: gantt-docs | -0.20% | -0.77% |
| corpus: git-docs | +0.35% | -0.58% |
| corpus: info-docs | -4.53% | -9.98% |
| corpus: ishikawa-docs | -0.09% | -1.41% |
| corpus: journey-docs | +2.80% | -1.37% |
| corpus: kanban-docs | +0.52% | -0.21% |
| corpus: mindmap-docs | -0.41% | +1.46% |
| corpus: packet-docs | -0.63% | -1.43% |
| corpus: pie-docs | -0.19% | +0.64% |
| corpus: quadrant-docs | -1.04% | +0.17% |
| corpus: radar-docs | +1.45% | -0.87% |
| corpus: railroad-docs | -0.19% | +0.49% |
| corpus: requirement-docs | -0.15% | +0.79% |
| corpus: sankey-docs | +0.62% | -0.46% |
| corpus: sequence-docs | -0.79% | +0.05% |
| corpus: state-docs | +0.59% | +0.88% |
| corpus: swimlane-docs | +0.93% | +0.93% |
| corpus: timeline-docs | +0.26% | +0.90% |
| corpus: treemap-docs | +1.06% | +4.56% |
| corpus: treeview-docs | -1.09% | -2.28% |
| corpus: usecase-docs | -0.19% | +0.65% |
| corpus: venn-docs | -1.27% | -1.16% |
| corpus: wardley-docs | +0.93% | -0.58% |
| corpus: xychart-docs | -0.43% | -0.56% |

[Every extended process and verification record](research/lexical-state-experiment-2026-10-21/extended-comparison.json).

| Rejection workload | Rejection time change |
| --- | ---: |
| corpus: agentflow-reference | -1.13% |


## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,160,080 → 5,160,080 | +0 | 3,797,408 → 3,797,976 | +568 |
| ffi-static | 41,590,632 → 41,593,856 | +3,224 | 4,228,952 → 4,229,336 | +384 |
| ffi-dynamic | 4,766,240 → 4,766,624 | +384 | 3,516,308 → 3,516,744 | +436 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/lexical-state-experiment-2026-10-21/size-comparison.json), [development drivers separately](research/lexical-state-experiment-2026-10-21/driver-sizes.json).

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

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/lexical-state-experiment-2026-10-21/allocations-primary.json), [probe source](research/lexical-state-experiment-2026-10-21/primary_allocations.rs), [default Samples](research/lexical-state-experiment-2026-10-21/allocations-samples.json), [default extended](research/lexical-state-experiment-2026-10-21/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 2 / 2 / 29 |
| extended | 5 / 0 / 34 |

Largest samples increases: samples: er 4,554,752 → 4,751,360 bytes; samples: gantt 4,423,680 → 4,603,904 bytes; samples: agentflow 4,833,280 → 5,013,504 bytes.

Largest extended increases: corpus: sequence-docs 4,980,736 → 5,341,184 bytes; mesh: 500 nodes, 200 extra labelled edges 10,878,976 → 11,206,656 bytes; corpus: er-docs 5,062,656 → 5,373,952 bytes.

[All Sample processes](research/lexical-state-experiment-2026-10-21/resources-samples.json), [all extended processes](research/lexical-state-experiment-2026-10-21/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Complete workspace: **3,245 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- The two supplementary handwritten contracts pass before and after the runtime change. All existing independent error, location, parser, numeric, label, root and response assertions remain. No original expected value is generated or changed. Rejection timings are reported separately; a successful-input improvement does not excuse a rejection or original-timeout regression.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Candidate validation](research/lexical-state-experiment-2026-10-21/validation.json), [restored-production revalidation](research/lexical-state-experiment-2026-10-21/production-revalidation.json). After preserving the complete candidate patch and supplementary source, only the one modified runtime file is restored and the two-test candidate-only file is removed from the production checkout. No original test or expectation is removed. Restored production has 3,243 passed, zero failed, five ignored. Work toward the complete 2× API target and compiled-size objective remains active.
