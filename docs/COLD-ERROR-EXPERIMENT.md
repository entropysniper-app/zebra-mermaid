# Cold error hint experiment

**This candidate was not retained.** The observed paired native-time reduction is small, all allocation metrics are unchanged, and instruction text grows in all three shipping artifacts. Production remains `528010f970b8b9b54d94d88275c43e96f4b32015`, with API 1.85× / UTF-8 ready 1.97×. The 2× API-return target and compiled-size objective remain active.

Across all **167 original Samples / 33 types**, paired native render time is **0.99570× (-0.43%)**. The independently remeasured reference speedup is **API 1.86×**, **UTF-8 ready 1.98×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `32d553241b2fc2f42813ef00afc41454abc92f45` (runtime `528010f970b8b9b54d94d88275c43e96f4b32015`), candidate preserved as an [exact source patch](research/cold-error-experiment-2026-10-21/candidate.patch.gz), not a production revision. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/cold-error-experiment-2026-10-21/summary.json).

## Measured scope and implementation

Fresh CPU captures cover eight selected types / 36 original inputs: Info, Packet, Radar, Pie, Flowchart, Sequence, Agentflow and TreeView. The full timing collection below remains **all 167 original Samples / 33 types**. The successful-input captures contain no sampled error-construction frames; normal parser/renderer frames remain. Counts are inclusive, overlapping and dependent on inlining, not additive exclusive CPU percentages. [Before](research/cold-error-experiment-2026-10-21/cpu-before.json), [after](research/cold-error-experiment-2026-10-21/cpu-after.json), [target frames](research/cold-error-experiment-2026-10-21/hot-frames.json), [trait inventory](research/cold-error-experiment-2026-10-21/trait-frames.json).

## Code-layout hints and failure contract

Only **23 entrypoints across 16 source files** receive `#[cold]` and `#[inline(never)]`: central error constructors/location and parser failure helpers. Removing exactly the 46 added attribute lines reproduces the baseline source byte for byte, with no function body, literal, assertion or control-flow change. The hints ask the compiler to separate failure-only code and avoid copying it into normal callers. They do not skip constructing an error or alter its work.

All public APIs, codes/messages, UTF-16 locations/snippets, source byte spans, type tags, first-error ordering and original limits remain. Success and rejection timings are both retained, along with every regression. Full parsing, layout and SVG output, workloads, precision and diagram features remain. No dependency, toolchain/release-profile change, unsafe code, preparse or returned cache is introduced. [Every attributed function](research/cold-error-experiment-2026-10-21/conversion.json), [scope and invariants](research/cold-error-experiment-2026-10-21/changes.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/cold-error-experiment-2026-10-21/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -2.04% | 1.66× | 1.75× |
| block | +0.30% | 1.39× | 1.42× |
| c4 | +0.12% | 1.56× | 1.61× |
| class | -1.21% | 1.59× | 1.65× |
| er | -0.06% | 1.48× | 1.55× |
| eventmodeling | -1.07% | 2.37× | 2.95× |
| gantt | -0.23% | 1.79× | 1.81× |
| git | -0.39% | 2.01× | 2.07× |
| info | +0.90% | 1.96× | 2.43× |
| journey | +0.87% | 2.05× | 2.16× |
| kanban | +1.50% | 1.51× | 1.53× |
| mindmap | +1.88% | 2.13× | 2.48× |
| packet | -0.54% | 2.25× | 2.36× |
| pie | -0.30% | 1.87× | 2.12× |
| quadrant | -0.92% | 1.67× | 1.80× |
| requirement | -0.08% | 1.66× | 1.76× |
| sankey | +0.73% | 2.84× | 3.41× |
| sequence | +0.20% | 1.70× | 1.79× |
| state | -0.34% | 1.63× | 1.72× |
| timeline | +0.99% | 2.28× | 2.53× |
| xychart | +0.69% | 1.66× | 1.71× |
| agentflow | +1.47% | 1.24× | 1.26× |
| architecture | -1.20% | 2.69× | 2.69× |
| cynefin | -0.62% | 2.01× | 2.53× |
| ishikawa | -0.25% | 1.98× | 2.05× |
| radar | -4.47% | 2.59× | 2.66× |
| railroad | +1.47% | 1.37× | 1.42× |
| swimlane | -0.14% | 1.44× | 1.50× |
| treemap | -4.92% | 2.28× | 2.35× |
| treeview | -3.88% | 2.21× | 2.49× |
| usecase | -3.18% | 1.52× | 1.48× |
| venn | +0.11% | 2.35× | 2.39× |
| wardley | +0.88% | 1.83× | 1.91× |

API speedup ranges from 1.24× (agentflow, slowest relative to Pele) to 2.84× (sankey); 20 of 33 types remain below 2×. An aggregate target does not imply every type reaches it.

[Candidate output-contract timings](research/cold-error-experiment-2026-10-21/output/comparison.json) preserve all Sample process reports in the same folder. The [candidate four-engine comparison](research/cold-error-experiment-2026-10-21/four-engine/comparison.json) has its own 110-input / 22-type common-success subset; it does not replace the primary collection. Published terminal production reports and README remain unchanged.

Across the 33 documentation types, render is **-0.79%**, parse **-2.05%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -0.26% | +1.43% |
| tree: 50 nodes | -1.41% | -2.64% |
| mesh: 50 nodes, 20 extra labelled edges | +1.14% | -0.51% |
| tree: 500 nodes | -0.68% | -2.30% |
| mesh: 500 nodes, 200 extra labelled edges | -1.26% | -3.34% |
| corpus: agentflow-docs | -1.45% | -0.05% |
| corpus: agentflow-reference | +0.33% | -2.33% |
| corpus: architecture-docs | -1.60% | -6.88% |
| corpus: block-docs | -1.64% | +2.99% |
| corpus: c4-docs | +0.43% | +0.83% |
| corpus: class-docs | +0.49% | +1.02% |
| corpus: cynefin-docs | -1.65% | -4.42% |
| corpus: er-docs | +0.65% | +0.63% |
| corpus: eventmodeling-docs | -1.38% | -4.83% |
| corpus: flowchart-docs | -0.46% | -0.25% |
| corpus: gantt-docs | +0.36% | -1.94% |
| corpus: git-docs | +0.27% | -1.29% |
| corpus: info-docs | +0.73% | -1.58% |
| corpus: ishikawa-docs | -0.05% | -4.88% |
| corpus: journey-docs | -0.24% | -1.92% |
| corpus: kanban-docs | -1.81% | -1.46% |
| corpus: mindmap-docs | +0.61% | +0.88% |
| corpus: packet-docs | -1.63% | -2.47% |
| corpus: pie-docs | -1.17% | -3.14% |
| corpus: quadrant-docs | +0.33% | -0.08% |
| corpus: radar-docs | -4.85% | -8.77% |
| corpus: railroad-docs | -1.85% | -0.53% |
| corpus: requirement-docs | -1.58% | -2.27% |
| corpus: sankey-docs | -0.19% | -0.10% |
| corpus: sequence-docs | +0.84% | -0.97% |
| corpus: state-docs | +0.61% | +0.04% |
| corpus: swimlane-docs | +0.40% | +0.61% |
| corpus: timeline-docs | +2.04% | +1.18% |
| corpus: treemap-docs | -2.78% | -15.15% |
| corpus: treeview-docs | -3.05% | -5.48% |
| corpus: usecase-docs | -2.03% | -0.35% |
| corpus: venn-docs | -0.99% | -1.78% |
| corpus: wardley-docs | -0.69% | -2.34% |
| corpus: xychart-docs | -2.27% | -0.81% |

[Every extended process and verification record](research/cold-error-experiment-2026-10-21/extended-comparison.json).

| Rejection workload | Rejection time change |
| --- | ---: |
| corpus: agentflow-reference | -1.48% |


## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,160,080 → 5,176,912 | +16,832 | 3,797,408 → 3,804,020 | +6,612 |
| ffi-static | 41,590,632 → 41,597,512 | +6,880 | 4,228,952 → 4,233,872 | +4,920 |
| ffi-dynamic | 4,766,240 → 4,766,432 | +192 | 3,516,308 → 3,520,664 | +4,356 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/cold-error-experiment-2026-10-21/size-comparison.json), [development drivers separately](research/cold-error-experiment-2026-10-21/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 0 / 33 / 0 |
| Primary Arial16 Samples | reallocations | 0 / 33 / 0 |
| Primary Arial16 Samples | requestedBytes | 0 / 33 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default Samples | allocations | 0 / 33 / 0 |
| Default Samples | reallocations | 0 / 33 / 0 |
| Default Samples | requestedBytes | 0 / 33 / 0 |
| Default Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default extended | allocations | 0 / 39 / 0 |
| Default extended | reallocations | 0 / 39 / 0 |
| Default extended | requestedBytes | 0 / 39 / 0 |
| Default extended | peakAdditionalLiveBytes | 0 / 39 / 0 |

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/cold-error-experiment-2026-10-21/allocations-primary.json), [probe source](research/cold-error-experiment-2026-10-21/primary_allocations.rs), [default Samples](research/cold-error-experiment-2026-10-21/allocations-samples.json), [default extended](research/cold-error-experiment-2026-10-21/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 13 / 3 / 17 |
| extended | 21 / 5 / 13 |

Largest samples increases: samples: class 4,521,984 → 4,816,896 bytes; samples: sequence 4,374,528 → 4,571,136 bytes; samples: kanban 4,014,080 → 4,177,920 bytes.

Largest extended increases: mesh: 500 nodes, 200 extra labelled edges 10,682,368 → 11,091,968 bytes; corpus: sequence-docs 4,980,736 → 5,308,416 bytes; corpus: usecase-docs 5,242,880 → 5,439,488 bytes.

[All Sample processes](research/cold-error-experiment-2026-10-21/resources-samples.json), [all extended processes](research/cold-error-experiment-2026-10-21/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness, restoration and original acceptance

- Complete workspace: **3,243 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- No new test or expected value is introduced. All existing independent error, location, parser, numeric, label, root and response assertions remain. Rejection timings are reported separately; fast success paths do not excuse a rejection or original-timeout regression.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Candidate validation](research/cold-error-experiment-2026-10-21/validation.json), [restored-production revalidation](research/cold-error-experiment-2026-10-21/production-revalidation.json). Only the 16 attributed source files are restored after preserving the complete patch and all observations. No test or expectation is removed. Work toward the complete 2× API target and compiled-size objective remains active.
