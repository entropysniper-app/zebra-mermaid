# Shared SVG validation costs

Across all **167 original Samples / 33 types**, paired native render time is **0.99239× (-0.76%)**. The independently remeasured reference speedup is **API 1.81×**, **UTF-8 ready 1.92×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `97d5dabd58f1e21b73f67b3cdce4c0b363698c4b` (runtime `c1ca29b903feba11145f179a38ed587fa1b82554`), candidate `80b3c8882188bc892bbc7a324ee113ba8dd54fd9`. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/shared-svg-cost-optimization-2026-10-20/summary.json).

## Measured scope and implementation

Fresh CPU captures covered **all 33 primary types / 167 inputs**, with Arial 16, CSS output and the fixed clock. Root/font validation and coordinate writing recur across these captures. Frame occurrences are inclusive, overlap and depend on compiler inlining; they are not additive exclusive percentages. Raw captures remain ignored and are identified by SHA-256. [Before](research/shared-svg-cost-optimization-2026-10-20/cpu-before.json), [after](research/shared-svg-cost-optimization-2026-10-20/cpu-after.json), [target frames](research/shared-svg-cost-optimization-2026-10-20/hot-frames.json).

Info's named font-validator occurrences fell **457 → 36** across captures with 3,538 / 3,544 main-thread samples. Sankey's UTF-8 conversion occurrences below named coordinate-writer frames fell **168 → 133** with 3,645 / 3,648 thread samples. These observations support inspecting those paths; the full-render measurements below determine the combined result of both edits.

- Font-list validation now streams characters instead of collecting a `Vec<char>`. ASCII letters/numbers use their exact ASCII predicates; other scalars use the original Unicode property tables. Quoting, forbidden characters, whitespace, comma rules, empty quoted names and fallback output are unchanged. This does not cache or normalize user input.
- Both coordinate-writing paths still run the original rounding, decimal-digit construction and large/nonfinite fallback. Checked UTF-8 conversion now covers the fixed zero-filled 32-byte buffer, then selects the unchanged decimal suffix. All buffer bytes are ASCII; no unchecked conversion or unsafe code is introduced. Public signatures, precision and output bytes remain.

The separate 4,096-value conversion experiment uses three alternating fresh processes, five warm-ups, at least 10 ms calibration, at least nine samples and 300 ms per mode/process. It selected the implementation; it is **not a complete-render result or target evidence**. [All raw observations](research/shared-svg-cost-optimization-2026-10-20/utf8-probe.json), [exact probe source](research/shared-svg-cost-optimization-2026-10-20/utf8-probe.rs). [Changes and independent reference provenance](research/shared-svg-cost-optimization-2026-10-20/changes.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/shared-svg-cost-optimization-2026-10-20/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -1.17% | 1.67× | 1.71× |
| block | -0.10% | 1.32× | 1.38× |
| c4 | +0.17% | 1.52× | 1.57× |
| class | +0.03% | 1.55× | 1.59× |
| er | -0.23% | 1.45× | 1.51× |
| eventmodeling | -2.86% | 2.16× | 2.71× |
| gantt | +0.30% | 1.75× | 1.83× |
| git | -1.02% | 1.90× | 1.98× |
| info | -9.01% | 1.87× | 2.21× |
| journey | -1.96% | 1.96× | 2.08× |
| kanban | +0.25% | 1.45× | 1.45× |
| mindmap | +0.24% | 2.05× | 2.41× |
| packet | -0.96% | 2.02× | 2.09× |
| pie | +0.47% | 1.80× | 2.13× |
| quadrant | -1.12% | 1.65× | 1.77× |
| requirement | +0.31% | 1.63× | 1.74× |
| sankey | -1.38% | 2.78× | 3.30× |
| sequence | -0.38% | 1.76× | 1.75× |
| state | -0.51% | 1.70× | 1.70× |
| timeline | +0.79% | 2.21× | 2.36× |
| xychart | -0.66% | 1.61× | 1.65× |
| agentflow | +0.54% | 1.25× | 1.30× |
| architecture | +1.52% | 2.56× | 2.61× |
| cynefin | -0.90% | 1.90× | 2.45× |
| ishikawa | -1.45% | 1.92× | 2.01× |
| radar | -0.39% | 2.48× | 2.55× |
| railroad | +0.24% | 1.35× | 1.42× |
| swimlane | -0.59% | 1.47× | 1.47× |
| treemap | -1.05% | 2.22× | 2.23× |
| treeview | -1.14% | 2.17× | 2.32× |
| usecase | -1.31% | 1.43× | 1.49× |
| venn | -0.07% | 2.35× | 2.38× |
| wardley | -1.20% | 1.83× | 1.92× |

[Complete output-contract timings](OUTPUT-BENCHMARK.md) preserve all Sample process reports. The [four-engine comparison](BENCHMARK.md) has its own 110-input / 22-type common-success subset; it does not replace the primary collection.

Across the 33 documentation types, render is **+0.60%**, parse **-0.16%**. The parsing implementation is unchanged; the small observed parse difference is not claimed as a parsing optimization. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | +0.82% | -0.09% |
| tree: 50 nodes | +1.29% | -0.71% |
| mesh: 50 nodes, 20 extra labelled edges | +1.52% | -0.52% |
| tree: 500 nodes | +0.34% | +1.31% |
| mesh: 500 nodes, 200 extra labelled edges | +0.14% | -0.18% |
| corpus: agentflow-docs | +0.78% | +0.57% |
| corpus: agentflow-reference | +0.46% | +1.67% |
| corpus: architecture-docs | +0.62% | -0.65% |
| corpus: block-docs | -0.33% | +0.20% |
| corpus: c4-docs | +0.06% | -0.17% |
| corpus: class-docs | +4.21% | +0.43% |
| corpus: cynefin-docs | +0.16% | -2.25% |
| corpus: er-docs | +0.31% | -0.73% |
| corpus: eventmodeling-docs | -1.08% | -3.03% |
| corpus: flowchart-docs | +0.25% | +2.37% |
| corpus: gantt-docs | +0.23% | +1.44% |
| corpus: git-docs | +1.33% | -2.28% |
| corpus: info-docs | +1.37% | -0.07% |
| corpus: ishikawa-docs | -0.89% | +3.05% |
| corpus: journey-docs | +2.15% | +0.79% |
| corpus: kanban-docs | +0.68% | +2.15% |
| corpus: mindmap-docs | -0.10% | +1.54% |
| corpus: packet-docs | +3.45% | +1.43% |
| corpus: pie-docs | -0.94% | -0.93% |
| corpus: quadrant-docs | -0.99% | +0.21% |
| corpus: radar-docs | +1.54% | -4.49% |
| corpus: railroad-docs | +1.60% | -0.05% |
| corpus: requirement-docs | +1.72% | +2.82% |
| corpus: sankey-docs | +0.26% | -0.38% |
| corpus: sequence-docs | +2.23% | +0.82% |
| corpus: state-docs | -0.02% | +1.38% |
| corpus: swimlane-docs | +0.21% | -0.08% |
| corpus: timeline-docs | +1.74% | +1.09% |
| corpus: treemap-docs | -2.73% | -10.52% |
| corpus: treeview-docs | +0.08% | -2.48% |
| corpus: usecase-docs | +1.23% | +1.20% |
| corpus: venn-docs | +0.82% | -0.76% |
| corpus: wardley-docs | +1.70% | +1.19% |
| corpus: xychart-docs | -1.45% | +1.82% |

[Every extended process and verification record](research/shared-svg-cost-optimization-2026-10-20/extended-comparison.json).

## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,142,400 → 5,142,384 | -16 | 3,797,128 → 3,798,116 | +988 |
| ffi-static | 41,537,032 → 41,541,464 | +4,432 | 4,226,252 → 4,227,220 | +968 |
| ffi-dynamic | 4,749,136 → 4,749,104 | -32 | 3,513,640 → 3,514,608 | +968 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/shared-svg-cost-optimization-2026-10-20/size-comparison.json), [development drivers separately](research/shared-svg-cost-optimization-2026-10-20/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 33 / 0 / 0 |
| Primary Arial16 Samples | reallocations | 33 / 0 / 0 |
| Primary Arial16 Samples | requestedBytes | 33 / 0 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 32 / 1 / 0 |
| Default Samples | allocations | 0 / 33 / 0 |
| Default Samples | reallocations | 0 / 33 / 0 |
| Default Samples | requestedBytes | 0 / 33 / 0 |
| Default Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default extended | allocations | 0 / 39 / 0 |
| Default extended | reallocations | 0 / 39 / 0 |
| Default extended | requestedBytes | 0 / 39 / 0 |
| Default extended | peakAdditionalLiveBytes | 0 / 39 / 0 |

The initial supplemental probe read net live bytes while constructing its own JSON report. It was corrected to snapshot every counter before report construction; all rows here use the corrected probe, before any latency run began. [Preserved instrumentation diagnostic](research/shared-svg-cost-optimization-2026-10-20/primary-allocation-snapshot-correction.json).

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/shared-svg-cost-optimization-2026-10-20/allocations-primary.json), [probe source](research/shared-svg-cost-optimization-2026-10-20/primary_allocations.rs), [default Samples](research/shared-svg-cost-optimization-2026-10-20/allocations-samples.json), [default extended](research/shared-svg-cost-optimization-2026-10-20/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 15 / 4 / 14 |
| extended | 10 / 2 / 27 |

Largest samples increases: samples: mindmap 4,079,616 → 4,177,920 bytes; samples: wardley 4,390,912 → 4,472,832 bytes; samples: sankey 4,390,912 → 4,456,448 bytes.

Largest extended increases: corpus: state-docs 5,160,960 → 5,423,104 bytes; docs: all flowchart examples 5,373,952 → 5,537,792 bytes; corpus: agentflow-docs 5,292,032 → 5,455,872 bytes.

[All Sample processes](research/shared-svg-cost-optimization-2026-10-20/resources-samples.json), [all extended processes](research/shared-svg-cost-optimization-2026-10-20/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Complete workspace: **3,240 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- One new supplementary reference test checks **5,000 font lists**, with three assertions each: acceptance, exact fallback selection and complete font-style text. The pinned independent Pele function generated every expected value before the runtime edit. Both baseline and candidate pass; **2,761 accepted / 2,239 rejected**, seed `0x16f017`, 4,096 generated inputs. ASCII controls, Unicode properties/whitespace, quoted names, injection punctuation and long lists remain covered. This is not an original-test port.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/shared-svg-cost-optimization-2026-10-20/validation.json). Work toward the complete 2× API target and compiled-size objective remains active.
