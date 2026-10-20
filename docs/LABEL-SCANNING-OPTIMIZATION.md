# Label scanning and substring search

Across all **167 original Samples / 33 types**, paired native render time is **0.98332× (-1.67%)**. The independently remeasured reference speedup is **API 1.83×**, **UTF-8 ready 1.95×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `3678cf20455d0af31679e60fd3fbbdc48c112a3f` (runtime `80b3c8882188bc892bbc7a324ee113ba8dd54fd9`), candidate `77810fb4aafbe4a4a6e1f8fba7b6230a9e57726b`. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/label-scanning-optimization-2026-10-20/summary.json).

## Measured scope and implementation

Fresh CPU captures cover eight selected types / 36 original inputs: Timeline, C4, Mindmap, Ishikawa, Requirement, Eventmodeling, Packet and Journey. They use the exact primary Arial16/CSS/fixed-clock contract. The latest full 33-type capture guided this selection; the primary timing collection below still includes **all 167 inputs / 33 types**. Inclusive frame occurrences overlap and depend on compiler inlining; they are not additive exclusive percentages. Raw host captures remain ignored and SHA-256 identified. [Before](research/label-scanning-optimization-2026-10-20/cpu-before.json), [after](research/label-scanning-optimization-2026-10-20/cpu-after.json), [target frames](research/label-scanning-optimization-2026-10-20/hot-frames.json).

- Inline text now borrows continuous source ranges. Existing ASCII markup delimiters are located by byte search. Text is joined only where a skipped non-style tag removes a gap, before the original icon/entity recognition. Original style/emphasis flush boundaries, marker lookahead, spans and combined callback order remain. Recognized tag names use equivalent ASCII-insensitive comparisons instead of allocating lowercase copies; the original tag grammar admits ASCII names.
- Line-break splitting yields the same source slices lazily, including final empty fragments, instead of allocating a fragment vector. The original break matcher, line order, measurements, wrapping, floating operations and output values remain.
- Short substring search locates the first needle byte and compares the complete remainder. A nonempty valid UTF-8 needle starts with a non-continuation byte, so a complete match in valid UTF-8 starts on a character boundary. Empty and oversized needles, first-match order, and the existing **96-byte memmem cutoff** remain. There is no character-boundary slicing in the byte-search loop.

The compact CPU summaries omit some Rust trait-implementation/drop symbols that do not start with `zebra_mermaid::`. An [additional impl-frame inventory](research/label-scanning-optimization-2026-10-20/trait-frames.json) preserves those names from the same raw captures. Its overlapping entries guide subsequent profiling; they are not exclusive cost measurements.

No source, token, label, parsed model or result cache is added. Public ownership/API types, precision, diagram features, original inputs and thresholds are unchanged. No dependency or unsafe code is added. [Changes and independent reference provenance](research/label-scanning-optimization-2026-10-20/changes.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/label-scanning-optimization-2026-10-20/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

Observed native medians fall in **25 types** and rise in eight. API speedups range from **Agentflow 1.29×** to **Sankey 2.81×**; **22 types remain below 2×**, and 11 reach at least 2×. Eventmodeling is **−6.52%**, Timeline **−6.01%**, Mindmap **−5.74%** and Packet **−5.72%**. All regressions remain in the full table; these subsets do not replace the equal-weight primary result.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | +0.60% | 1.65× | 1.68× |
| block | -0.79% | 1.34× | 1.40× |
| c4 | -0.80% | 1.57× | 1.62× |
| class | -0.54% | 1.53× | 1.57× |
| er | -0.15% | 1.46× | 1.50× |
| eventmodeling | -6.52% | 2.27× | 2.81× |
| gantt | -0.26% | 1.79× | 1.81× |
| git | -2.24% | 1.94× | 1.99× |
| info | +0.50% | 1.79× | 2.19× |
| journey | -2.78% | 2.04× | 2.16× |
| kanban | -1.64% | 1.44× | 1.48× |
| mindmap | -5.74% | 2.09× | 2.52× |
| packet | -5.72% | 2.11× | 2.20× |
| pie | -0.81% | 1.85× | 2.16× |
| quadrant | -1.01% | 1.65× | 1.78× |
| requirement | -3.65% | 1.67× | 1.74× |
| sankey | -1.44% | 2.81× | 3.40× |
| sequence | -1.98% | 1.70× | 1.79× |
| state | +1.12% | 1.67× | 1.73× |
| timeline | -6.01% | 2.30× | 2.53× |
| xychart | -0.15% | 1.73× | 1.67× |
| agentflow | +0.28% | 1.29× | 1.31× |
| architecture | -1.67% | 2.61× | 2.66× |
| cynefin | -2.47% | 1.96× | 2.51× |
| ishikawa | -1.75% | 1.97× | 2.05× |
| radar | +0.73% | 2.45× | 2.52× |
| railroad | -1.35% | 1.35× | 1.41× |
| swimlane | -4.15% | 1.43× | 1.49× |
| treemap | -2.32% | 2.27× | 2.25× |
| treeview | -3.66% | 2.16× | 2.35× |
| usecase | +1.47% | 1.44× | 1.45× |
| venn | +0.02% | 2.33× | 2.39× |
| wardley | +0.59% | 1.82× | 1.91× |

[Complete output-contract timings](OUTPUT-BENCHMARK.md) preserve all Sample process reports. The [four-engine comparison](BENCHMARK.md) has its own 110-input / 22-type common-success subset; it does not replace the primary collection.

Across the 33 documentation types, render is **-2.39%**, parse **+0.15%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -2.31% | -0.85% |
| tree: 50 nodes | -2.22% | +0.23% |
| mesh: 50 nodes, 20 extra labelled edges | +0.47% | -0.22% |
| tree: 500 nodes | +0.17% | +0.75% |
| mesh: 500 nodes, 200 extra labelled edges | -0.61% | +1.11% |
| corpus: agentflow-docs | -0.97% | -0.22% |
| corpus: agentflow-reference | -3.12% | +0.91% |
| corpus: architecture-docs | -2.45% | +1.08% |
| corpus: block-docs | -1.86% | +4.22% |
| corpus: c4-docs | -2.74% | -0.14% |
| corpus: class-docs | -0.25% | +0.31% |
| corpus: cynefin-docs | -3.53% | +1.73% |
| corpus: er-docs | -4.51% | -1.88% |
| corpus: eventmodeling-docs | -6.47% | -1.18% |
| corpus: flowchart-docs | -2.62% | -0.05% |
| corpus: gantt-docs | -1.16% | -2.00% |
| corpus: git-docs | -0.81% | +1.39% |
| corpus: info-docs | +0.70% | -1.51% |
| corpus: ishikawa-docs | -0.80% | +4.53% |
| corpus: journey-docs | -5.59% | +0.89% |
| corpus: kanban-docs | -1.79% | +0.83% |
| corpus: mindmap-docs | -1.44% | +1.07% |
| corpus: packet-docs | -4.85% | -0.63% |
| corpus: pie-docs | -0.34% | +1.65% |
| corpus: quadrant-docs | -2.54% | +1.14% |
| corpus: radar-docs | -1.13% | -0.57% |
| corpus: railroad-docs | -0.67% | +0.91% |
| corpus: requirement-docs | -6.64% | -3.02% |
| corpus: sankey-docs | -3.97% | -0.94% |
| corpus: sequence-docs | -4.64% | -1.38% |
| corpus: state-docs | -1.80% | -0.87% |
| corpus: swimlane-docs | -1.80% | -0.21% |
| corpus: timeline-docs | -3.40% | +0.03% |
| corpus: treemap-docs | -2.79% | +0.70% |
| corpus: treeview-docs | -1.75% | -1.10% |
| corpus: usecase-docs | -2.60% | -2.34% |
| corpus: venn-docs | -1.65% | -0.48% |
| corpus: wardley-docs | -2.19% | -0.81% |
| corpus: xychart-docs | +0.68% | +4.34% |

[Every extended process and verification record](research/label-scanning-optimization-2026-10-20/extended-comparison.json).

## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,142,384 → 5,159,248 | +16,864 | 3,798,116 → 3,798,920 | +804 |
| ffi-static | 41,541,464 → 41,545,208 | +3,744 | 4,227,220 → 4,227,616 | +396 |
| ffi-dynamic | 4,749,104 → 4,749,184 | +80 | 3,514,608 → 3,515,116 | +508 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/label-scanning-optimization-2026-10-20/size-comparison.json), [development drivers separately](research/label-scanning-optimization-2026-10-20/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 10 / 23 / 0 |
| Primary Arial16 Samples | reallocations | 10 / 23 / 0 |
| Primary Arial16 Samples | requestedBytes | 10 / 23 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default Samples | allocations | 10 / 23 / 0 |
| Default Samples | reallocations | 10 / 23 / 0 |
| Default Samples | requestedBytes | 10 / 23 / 0 |
| Default Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default extended | allocations | 15 / 24 / 0 |
| Default extended | reallocations | 15 / 24 / 0 |
| Default extended | requestedBytes | 15 / 24 / 0 |
| Default extended | peakAdditionalLiveBytes | 0 / 39 / 0 |

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/label-scanning-optimization-2026-10-20/allocations-primary.json), [probe source](research/label-scanning-optimization-2026-10-20/primary_allocations.rs), [default Samples](research/label-scanning-optimization-2026-10-20/allocations-samples.json), [default extended](research/label-scanning-optimization-2026-10-20/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 14 / 3 / 16 |
| extended | 22 / 2 / 15 |

Largest samples increases: samples: wardley 4,358,144 → 4,587,520 bytes; samples: gantt 4,358,144 → 4,489,216 bytes; samples: block 4,243,456 → 4,341,760 bytes.

Largest extended increases: tree: 500 nodes 8,126,464 → 8,585,216 bytes; corpus: sequence-docs 5,111,808 → 5,292,032 bytes; corpus: block-docs 4,718,592 → 4,816,896 bytes.

[All Sample processes](research/label-scanning-optimization-2026-10-20/resources-samples.json), [all extended processes](research/label-scanning-optimization-2026-10-20/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Complete workspace: **3,242 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- One new supplementary Node reference checks **8,448 label cases** through both the public and private storage paths: exact label f64 bits and the combined measurement/icon callback sequence. The independent pinned reference generated expected values before runtime edits; baseline and candidate pass. It covers 88 sources (64 seeded generated sources), seed `0x1abe17`, both markdown/icon modes, styles, negative/positive sizes and narrow/infinite widths. Skipped-tag joins that form entity codes or icon names, malformed tags, emphasis boundaries and trailing empty lines are included.
- One new standard-library differential test checks **4,096 seeded Unicode cases**, eight needles and both find/contains per case, seed `0x5e17f1`. Empty/missing needles, one- through four-byte scalars, overlapping prefixes and lengths around the unchanged cutoff remain covered. These two project tests are not original-test ports.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/label-scanning-optimization-2026-10-20/validation.json). Work toward the complete 2× API target and compiled-size objective remains active.
