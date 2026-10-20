# Shared preprocessing-view experiment

Across all **167 original Samples / 33 types**, paired native render time is **1.00368× (+0.37%)**. The independently remeasured reference speedup is **API 1.84×**, **UTF-8 ready 1.97×**. This variant is **not retained as production runtime**: fewer allocations did not establish a complete-render speedup, and instruction text grew. The preceding verified production reports remain current; the **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `b963d10ccbae6495f42b432c22284b57a7ccee70` (runtime `528010f970b8b9b54d94d88275c43e96f4b32015`), experimental working-tree patch over that baseline. No production commit is assigned; [exact runtime, generator and Node fixture patch](research/preprocess-view-experiment-2026-10-20/candidate.patch.gz). Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/preprocess-view-experiment-2026-10-20/summary.json).

## Measured scope and implementation

Fresh primary-contract CPU captures cover eight selected types / 40 original inputs: Pie, Packet, TreeView, Radar, Gantt, Flowchart, Class and State. This profiling selection does not replace **all 167 original Samples / 33 types** in the primary timings. Counts are inclusive, overlap and depend on inlining; they are not additive exclusive CPU percentages. [Before](research/preprocess-view-experiment-2026-10-20/cpu-before.json), [after](research/preprocess-view-experiment-2026-10-20/cpu-after.json), [target frames](research/preprocess-view-experiment-2026-10-20/hot-frames.json), [trait inventory](research/preprocess-view-experiment-2026-10-20/trait-frames.json).

## Implementation and source mapping

- A source-map builder borrows contiguous copied source ranges until a rewrite or disjoint copy requires an owned buffer. Marks are recorded in the same order. Its public `finish` still returns owned text and preserves the original initial capacity; private consumers can retain a borrowed copied range. No input or result is retained between calls.
- Private parser preparation keeps one underlying comment-preserving text plus an optional changed text or explicit copied range. It avoids duplicating an unchanged view, including middle/prefix/suffix ranges after comment removal. Front-matter suffixes and unchanged directive phases can stay borrowed. Public preprocessing still returns both fully owned views with their original metadata/configuration.
- Type detection uses the same cleaned view. Only after the ordinary detection/feature checks does the parser move its selected text/map, with both comment modes preserved. Unused map clones are avoided; no semantic preprocessing or parsing work is moved outside rendering. Original normalization, quote rewrites, YAML/directive merging, error precedence, entity encoding, original byte origins and EOF mapping remain.

No public model/FFI ownership or feature change, dependency, toolchain/profile change, unsafe code, precision reduction, preparse or returned source/model/output cache is introduced. [Implementation scope](research/preprocess-view-experiment-2026-10-20/changes.json). The frozen committed public preprocessing API separately matches all 276 independent Node reference cases. [Baseline verifier and binary/dependency fingerprints](research/preprocess-view-experiment-2026-10-20/baseline-reference.json), [verifier source](research/preprocess-view-experiment-2026-10-20/baseline_reference.rs).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/preprocess-view-experiment-2026-10-20/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -1.26% | 1.67× | 1.68× |
| block | -0.30% | 1.39× | 1.44× |
| c4 | -0.33% | 1.59× | 1.62× |
| class | -1.21% | 1.57× | 1.61× |
| er | -0.01% | 1.49× | 1.57× |
| eventmodeling | -1.27% | 2.32× | 2.87× |
| gantt | +1.22% | 1.76× | 1.86× |
| git | +1.49% | 1.94× | 1.99× |
| info | +0.80% | 1.96× | 2.38× |
| journey | +1.04% | 2.00× | 2.15× |
| kanban | +0.65% | 1.44× | 1.51× |
| mindmap | -0.53% | 2.12× | 2.50× |
| packet | -0.64% | 2.22× | 2.32× |
| pie | +0.34% | 1.86× | 2.11× |
| quadrant | +0.75% | 1.67× | 1.79× |
| requirement | +2.04% | 1.63× | 1.74× |
| sankey | +0.05% | 3.04× | 3.38× |
| sequence | +1.12% | 1.74× | 1.80× |
| state | +0.44% | 1.68× | 1.69× |
| timeline | -0.22% | 2.29× | 2.53× |
| xychart | -1.08% | 1.66× | 1.80× |
| agentflow | +0.83% | 1.29× | 1.31× |
| architecture | +2.11% | 2.56× | 2.63× |
| cynefin | +1.68% | 1.92× | 2.49× |
| ishikawa | +0.92% | 1.93× | 2.04× |
| radar | -1.47% | 2.45× | 2.62× |
| railroad | +1.06% | 1.36× | 1.42× |
| swimlane | +1.11% | 1.46× | 1.48× |
| treemap | +0.24% | 2.20× | 2.25× |
| treeview | -0.35% | 2.17× | 2.39× |
| usecase | -0.20% | 1.47× | 1.52× |
| venn | +1.75% | 2.36× | 2.34× |
| wardley | +1.56% | 1.88× | 1.91× |

[All experimental output-contract timings](research/preprocess-view-experiment-2026-10-20/output/comparison.json) retain the original full sample collection and every process. The [experimental four-engine comparison](research/preprocess-view-experiment-2026-10-20/four-engine/comparison.json) uses the separate 110-input / 22-type common-success subset; neither replaces the production reports.

Across the 33 documentation types, render is **-0.27%**, parse **-1.02%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -1.08% | -1.21% |
| tree: 50 nodes | -0.22% | +0.89% |
| mesh: 50 nodes, 20 extra labelled edges | +0.61% | +0.67% |
| tree: 500 nodes | +0.25% | +0.67% |
| mesh: 500 nodes, 200 extra labelled edges | +3.31% | +0.79% |
| corpus: agentflow-docs | +3.33% | -0.63% |
| corpus: agentflow-reference | +2.88% | +0.61% |
| corpus: architecture-docs | -0.11% | -0.01% |
| corpus: block-docs | -0.20% | -0.24% |
| corpus: c4-docs | +1.73% | +0.38% |
| corpus: class-docs | -0.04% | -0.91% |
| corpus: cynefin-docs | +0.67% | -1.42% |
| corpus: er-docs | +1.55% | +0.44% |
| corpus: eventmodeling-docs | -2.48% | -3.36% |
| corpus: flowchart-docs | +0.65% | -0.60% |
| corpus: gantt-docs | -0.10% | -2.59% |
| corpus: git-docs | -0.31% | -1.13% |
| corpus: info-docs | +0.11% | -5.39% |
| corpus: ishikawa-docs | +1.71% | -0.13% |
| corpus: journey-docs | -0.63% | -0.11% |
| corpus: kanban-docs | -0.50% | -0.05% |
| corpus: mindmap-docs | -2.59% | -1.75% |
| corpus: packet-docs | -0.89% | -2.01% |
| corpus: pie-docs | -2.95% | -2.62% |
| corpus: quadrant-docs | +0.21% | -0.88% |
| corpus: radar-docs | -0.29% | -3.92% |
| corpus: railroad-docs | -0.99% | -0.57% |
| corpus: requirement-docs | +1.46% | +2.01% |
| corpus: sankey-docs | -1.81% | +0.08% |
| corpus: sequence-docs | -0.05% | -1.52% |
| corpus: state-docs | +0.85% | -1.67% |
| corpus: swimlane-docs | -1.33% | -0.13% |
| corpus: timeline-docs | -0.52% | -2.69% |
| corpus: treemap-docs | -2.82% | +0.12% |
| corpus: treeview-docs | +0.03% | +2.46% |
| corpus: usecase-docs | +0.86% | +1.22% |
| corpus: venn-docs | -1.65% | -0.84% |
| corpus: wardley-docs | -0.22% | -3.71% |
| corpus: xychart-docs | -1.29% | -1.16% |

[Every extended process and verification record](research/preprocess-view-experiment-2026-10-20/extended-comparison.json).

## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,160,080 → 5,177,552 | +17,472 | 3,797,408 → 3,799,804 | +2,396 |
| ffi-static | 41,590,632 → 41,612,400 | +21,768 | 4,228,952 → 4,231,060 | +2,108 |
| ffi-dynamic | 4,766,240 → 4,766,864 | +624 | 3,516,308 → 3,516,472 | +164 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/preprocess-view-experiment-2026-10-20/size-comparison.json), [development drivers separately](research/preprocess-view-experiment-2026-10-20/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 33 / 0 / 0 |
| Primary Arial16 Samples | reallocations | 21 / 12 / 0 |
| Primary Arial16 Samples | requestedBytes | 33 / 0 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default Samples | allocations | 33 / 0 / 0 |
| Default Samples | reallocations | 21 / 12 / 0 |
| Default Samples | requestedBytes | 33 / 0 / 0 |
| Default Samples | peakAdditionalLiveBytes | 0 / 33 / 0 |
| Default extended | allocations | 39 / 0 / 0 |
| Default extended | reallocations | 24 / 15 / 0 |
| Default extended | requestedBytes | 39 / 0 / 0 |
| Default extended | peakAdditionalLiveBytes | 0 / 39 / 0 |

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/preprocess-view-experiment-2026-10-20/allocations-primary.json), [probe source](research/preprocess-view-experiment-2026-10-20/primary_allocations.rs), [default Samples](research/preprocess-view-experiment-2026-10-20/allocations-samples.json), [default extended](research/preprocess-view-experiment-2026-10-20/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 5 / 2 / 26 |
| extended | 5 / 2 / 32 |

Largest samples increases: samples: er 4,538,368 → 4,767,744 bytes; samples: mindmap 4,096,000 → 4,259,840 bytes; samples: agentflow 4,833,280 → 4,997,120 bytes.

Largest extended increases: tree: 500 nodes 8,110,080 → 10,584,064 bytes; mesh: 500 nodes, 200 extra labelled edges 10,895,360 → 11,780,096 bytes; corpus: er-docs 5,062,656 → 5,341,184 bytes.

[All Sample processes](research/preprocess-view-experiment-2026-10-20/resources-samples.json), [all extended processes](research/preprocess-view-experiment-2026-10-20/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Experimental candidate workspace: **3,246 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- Three new project tests check **276 independently generated Node cases** (275 successful views/configuration/metadata and one expected failure), selected kept/cleaned inputs and entity encoding, **4,096 dense byte-origin map cases / 16 operations each**, and four handwritten original-byte/CRLF/EOF cases. Node generation uses seed `0x20a11e`, 256 generated inputs; dense maps use `0x20b11d`. Values are not generated from Rust or substituted for old expectations.
- JSON configuration numbers are compared as exact binary64 domain values, matching JavaScript's number type, rather than serde_json's integer/float storage tags. Existing preprocessing/entity fuzz loops, stress limits, offsets, expected values and original mapping are unchanged. These three supplementary tests are not original-test ports.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/preprocess-view-experiment-2026-10-20/validation.json). The runtime and new candidate test files are restored/removed only after their exact patch is preserved. [Restored production validation](research/preprocess-view-experiment-2026-10-20/production-revalidation.json) records the independently repeated checks. Work toward the complete 2× API target and compiled-size objective remains active.
