# SVG byte-staging experiment

Across all **167 original Samples / 33 types**, paired native render time is **0.99415× (-0.59%)**. The independently remeasured reference speedup is **API 1.83×**, **UTF-8 ready 1.94×**. This broad variant is **not retained as production code**: the small observed gain, unchanged allocator results, code growth and added abstraction do not justify replacing the current implementation. The production performance report remains at the preceding verified runtime; the **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `57384aa34c572776103f1348e4b84e296e23ade0` (runtime `528010f970b8b9b54d94d88275c43e96f4b32015`), experimental working-tree patch over that baseline. No production commit is assigned; [exact patch and fingerprints](research/svg-byte-staging-experiment-2026-10-20/candidate.patch.gz). Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/svg-byte-staging-experiment-2026-10-20/summary.json).

## Measured scope and implementation

Fresh CPU captures cover nine selected types / 52 original inputs: Packet, Sankey, XYChart, Flowchart, State, Sequence, Architecture, Wardley and C4. They use the primary Arial16/CSS/fixed-clock options. The full timing collection remains **167 original inputs / all 33 types**. Frame counts include trait impls, overlap and depend on inlining; they are not additive exclusive percentages. Raw host captures remain ignored and SHA-256 identified. [Before](research/svg-byte-staging-experiment-2026-10-20/cpu-before.json), [after](research/svg-byte-staging-experiment-2026-10-20/cpu-after.json), [target frames](research/svg-byte-staging-experiment-2026-10-20/hot-frames.json), [trait inventory](research/svg-byte-staging-experiment-2026-10-20/trait-frames.json).

## Output contract and implementation

One private static `Output` abstraction now serves both the existing String path and a byte-staging path. The same parts evaluate arguments before writing and keep their original write/callback/drop order. Public String, Display, model, formatting and FFI interfaces remain.

- **133 internal SVG buffers across 29 render files** stage valid UTF-8 fragments as bytes. Text inputs are valid `str` or encoded Rust `char`; coordinate/raw-number bytes come only from the same checked numeric algorithms. The byte vector is private and is not exposed for arbitrary mutation.
- Numeric parts retain the exact original rounding, digits and large/nonfinite fallback. The byte path appends those generated ASCII digits without converting each coordinate to `str` first. Original String and public numeric formatting paths retain their checked conversion. Existing generic formatting writers inside some label/marker helpers still perform their original checks; this does not remove every UTF-8 check.
- Byte buffers remain checked when exposed as string views or finalized into an owned String. Nested byte buffers append their validated-by-construction fragments directly. Rendering, Unicode content, complete SVG bytes, geometry, options and errors are preserved. No unsafe code or unchecked UTF-8 conversion is added.

No new dependency, toolchain/release-profile change, feature removal, precision reduction, preparse or source/model/output cache is introduced. All work remains within complete parse/layout/SVG rendering. [Converted buffers](research/svg-byte-staging-experiment-2026-10-20/conversion.json), [scope and invariants](research/svg-byte-staging-experiment-2026-10-20/changes.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/svg-byte-staging-experiment-2026-10-20/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -0.26% | 1.63× | 1.64× |
| block | -0.66% | 1.32× | 1.38× |
| c4 | +0.10% | 1.55× | 1.56× |
| class | -1.29% | 1.56× | 1.57× |
| er | -0.15% | 1.46× | 1.48× |
| eventmodeling | +0.28% | 2.27× | 2.79× |
| gantt | +0.02% | 1.73× | 1.79× |
| git | -1.84% | 1.95× | 1.98× |
| info | +2.35% | 1.89× | 2.28× |
| journey | +1.94% | 2.06× | 2.14× |
| kanban | -0.17% | 1.46× | 1.50× |
| mindmap | -2.86% | 2.10× | 2.50× |
| packet | -4.64% | 2.26× | 2.41× |
| pie | -1.48% | 1.82× | 2.12× |
| quadrant | -1.91% | 1.70× | 1.76× |
| requirement | -0.43% | 1.63× | 1.67× |
| sankey | -4.60% | 2.98× | 3.56× |
| sequence | +0.42% | 1.70× | 1.77× |
| state | -1.54% | 1.92× | 1.70× |
| timeline | +0.45% | 2.29× | 2.48× |
| xychart | -0.61% | 1.72× | 1.72× |
| agentflow | -0.40% | 1.25× | 1.29× |
| architecture | +0.83% | 2.53× | 2.61× |
| cynefin | -0.77% | 1.89× | 2.45× |
| ishikawa | -1.83% | 1.97× | 2.03× |
| radar | +0.82% | 2.41× | 2.46× |
| railroad | +0.13% | 1.36× | 1.41× |
| swimlane | -1.32% | 1.48× | 1.48× |
| treemap | +3.65% | 2.13× | 2.16× |
| treeview | +1.10% | 2.06× | 2.33× |
| usecase | -0.40% | 1.41× | 1.46× |
| venn | -0.86% | 2.36× | 2.36× |
| wardley | -2.90% | 1.87× | 1.95× |

[All experimental output-contract timings](research/svg-byte-staging-experiment-2026-10-20/output/comparison.json) preserve the full Sample collection and independent raw processes. The [experimental four-engine comparison](research/svg-byte-staging-experiment-2026-10-20/four-engine/comparison.json) uses its separate 110-input / 22-type common-success subset. Neither replaces the production reports.

Across the 33 documentation types, render is **-0.70%**, parse **-0.22%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | -1.81% | +0.24% |
| tree: 50 nodes | +0.39% | -0.06% |
| mesh: 50 nodes, 20 extra labelled edges | +0.93% | +0.57% |
| tree: 500 nodes | -0.08% | +1.04% |
| mesh: 500 nodes, 200 extra labelled edges | -1.25% | +1.02% |
| corpus: agentflow-docs | -0.66% | +0.06% |
| corpus: agentflow-reference | +2.84% | -1.58% |
| corpus: architecture-docs | +0.06% | -3.73% |
| corpus: block-docs | -2.33% | +0.12% |
| corpus: c4-docs | +1.27% | -0.30% |
| corpus: class-docs | -2.04% | -0.94% |
| corpus: cynefin-docs | -0.30% | -2.19% |
| corpus: er-docs | -0.36% | -0.20% |
| corpus: eventmodeling-docs | +0.41% | -1.53% |
| corpus: flowchart-docs | +1.09% | +0.05% |
| corpus: gantt-docs | +1.95% | -0.86% |
| corpus: git-docs | -0.10% | -1.61% |
| corpus: info-docs | +0.53% | +0.74% |
| corpus: ishikawa-docs | -1.04% | +0.62% |
| corpus: journey-docs | -1.02% | -0.25% |
| corpus: kanban-docs | -0.89% | +2.24% |
| corpus: mindmap-docs | -2.02% | +3.10% |
| corpus: packet-docs | -5.19% | +0.35% |
| corpus: pie-docs | -1.40% | -0.85% |
| corpus: quadrant-docs | -0.93% | +0.05% |
| corpus: radar-docs | -1.70% | -1.95% |
| corpus: railroad-docs | -1.77% | -2.18% |
| corpus: requirement-docs | +0.51% | -1.04% |
| corpus: sankey-docs | -3.40% | +0.28% |
| corpus: sequence-docs | +0.44% | +0.07% |
| corpus: state-docs | +2.81% | -0.88% |
| corpus: swimlane-docs | +0.37% | +1.55% |
| corpus: timeline-docs | +0.75% | +1.26% |
| corpus: treemap-docs | -0.27% | -1.69% |
| corpus: treeview-docs | -0.31% | -0.29% |
| corpus: usecase-docs | -0.19% | -0.60% |
| corpus: venn-docs | -0.85% | +2.43% |
| corpus: wardley-docs | +0.15% | -0.49% |
| corpus: xychart-docs | -6.23% | +1.73% |

[Every extended process and verification record](research/svg-byte-staging-experiment-2026-10-20/extended-comparison.json).

## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,160,080 → 5,195,136 | +35,056 | 3,797,408 → 3,829,184 | +31,776 |
| ffi-static | 41,590,632 → 42,056,992 | +466,360 | 4,228,952 → 4,257,056 | +28,104 |
| ffi-dynamic | 4,766,240 → 4,784,720 | +18,480 | 3,516,308 → 3,543,652 | +27,344 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/svg-byte-staging-experiment-2026-10-20/size-comparison.json), [development drivers separately](research/svg-byte-staging-experiment-2026-10-20/driver-sizes.json).

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

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/svg-byte-staging-experiment-2026-10-20/allocations-primary.json), [probe source](research/svg-byte-staging-experiment-2026-10-20/primary_allocations.rs), [default Samples](research/svg-byte-staging-experiment-2026-10-20/allocations-samples.json), [default extended](research/svg-byte-staging-experiment-2026-10-20/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 12 / 1 / 20 |
| extended | 20 / 3 / 16 |

Largest samples increases: samples: gantt 4,423,680 → 4,603,904 bytes; samples: sequence 4,407,296 → 4,571,136 bytes; samples: class 4,587,520 → 4,734,976 bytes.

Largest extended increases: corpus: venn-docs 4,505,600 → 4,718,592 bytes; corpus: class-docs 5,357,568 → 5,554,176 bytes; corpus: kanban-docs 4,456,448 → 4,620,288 bytes.

[All Sample processes](research/svg-byte-staging-experiment-2026-10-20/resources-samples.json), [all extended processes](research/svg-byte-staging-experiment-2026-10-20/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Experimental candidate workspace: **3,247 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- Four new project tests check **2,520 unchanged independent Node coordinate values**, **6,048 unchanged independent Node label SVG cases** through public/private adapters with/without icons, standard-equivalent argument evaluation/write/drop order, and rejection of invalid UTF-8 at finalization. Expected values are not generated from Rust or replaced. Existing independent numeric, label/callback, root and semantic references remain.
- These supplementary tests are not original-test ports. No original input, assertion, expected value, seed, loop count, failure or timeout threshold is changed.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/svg-byte-staging-experiment-2026-10-20/validation.json). The candidate patch and every observation remain available for review. The production runtime is restored and separately revalidated before this experiment is published; see [restored-runtime validation](research/svg-byte-staging-experiment-2026-10-20/production-revalidation.json). Work toward the complete 2× API target and compiled-size objective remains active.
