# Stack number text

Across all **167 original Samples / 33 types**, paired native render time is **0.98905× (-1.09%)**. The independently remeasured reference speedup is **API 1.85×**, **UTF-8 ready 1.97×**. The **2× API-return target remains unmet**. Small differences are not statistical-significance claims.

Baseline `28dbcd3a1a4fc6db1c94fb4d9b13969a97f43a5e` (runtime `77810fb4aafbe4a4a6e1f8fba7b6230a9e57726b`), candidate `528010f970b8b9b54d94d88275c43e96f4b32015`. Pele 1.7.0 is fixed to `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1 (31fca3adb 2026-06-26). [Complete fingerprints](research/number-text-optimization-2026-10-20/summary.json).

## Measured scope and implementation

Fresh CPU captures cover eight selected types / 42 original inputs: Info, Packet, Sankey, Architecture, Requirement, Wardley, C4 and XYChart. They use the exact primary Arial16/CSS/fixed-clock contract. This profiling selection does not replace the full **167-input / 33-type** timing collection below. Summaries now include project trait-implementation names as well as ordinary function names; all frame counts are inclusive, overlapping and sensitive to inlining, not additive exclusive percentages. [Before](research/number-text-optimization-2026-10-20/cpu-before.json), [after](research/number-text-optimization-2026-10-20/cpu-after.json), [target frames](research/number-text-optimization-2026-10-20/hot-frames.json), [impl inventory](research/number-text-optimization-2026-10-20/trait-frames.json).

## Implementation and numeric contract

- One private `NumberText` holds complete per-call text in a checked 32-byte stack buffer. Binary64 shortest conversion needs at most 17 significant decimal digits; JavaScript's fixed/scientific cutoffs and sign fit within 25 bytes. NaN, signed infinities and both zeros retain the original spellings. No unchecked UTF-8 conversion or unsafe code is added.
- The existing `zmij` shortest-digit generator and JavaScript notation boundaries are shared unchanged. Safe-integer and exponent bytes are written directly instead of using a temporary integer `String`/formatter. Public `js::to_string(f64)` still returns an owned `String`; fixed/precision rounding and all original floating operations remain. This is not an arbitrary-precision format change.
- **120 internal call sites in 31 files** use the stack text for SVG/borrowed label inputs. Owned-return helpers retain owned strings. The root validates width/height views once and reuses them within that single output. The coordinate fast path is unchanged; its large/nonfinite fallback uses the same complete number spelling. `Part` writes text directly with its exact byte-length hint; native Rust formatting of other numeric arguments remains.

No source, token, label, parsed-model or output cache is added, and no work is moved outside the measured render call. Public model/API/FFI ownership, features, source limits, precision and output content remain. No dependency, toolchain or release-profile change is introduced. [Converted sites and source hashes](research/number-text-optimization-2026-10-20/conversion.json), [scope and invariants](research/number-text-optimization-2026-10-20/changes.json).

## Full timing results

Every original input, all 33 types and all 39 extended loads remain. Full parsing/layout/SVG is timed without parsed-model or output reuse. Three fresh processes per version/engine alternate order, with five warm-up batches, at least 10 ms calibration, at least nine samples and 300 ms per operation. Builds, tests, CPU capture, allocation counting and process-RSS checks are separate from latency. Every run and regression is retained.

[Pre/post conditions](research/number-text-optimization-2026-10-20/timing-conditions.json) are limited process snapshots, not proof of sustained host idleness. Reference measurements are renewed each round; dividing absolute reference speedups across rounds does not establish a paired native gain.

Observed native medians fall in **25 types** and rise in eight. API speedups range from **Agentflow 1.23×** to **Sankey 2.83×**; **19 types remain below 2×**, while 14 reach at least 2×. Packet is **−6.02%**, Info **−5.29%**, XYChart **−3.71%** and Radar **−3.14%**. All regressions are retained in the full table; these subsets do not replace the equal-weight primary result.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | -0.34% | 1.67× | 1.67× |
| block | -2.40% | 1.34× | 1.40× |
| c4 | +0.98% | 1.55× | 1.60× |
| class | -0.11% | 1.55× | 1.61× |
| er | +0.63% | 1.47× | 1.53× |
| eventmodeling | -0.18% | 2.30× | 2.89× |
| gantt | +0.46% | 1.79× | 1.81× |
| git | -0.62% | 2.01× | 2.01× |
| info | -5.29% | 1.94× | 2.37× |
| journey | +0.83% | 2.02× | 2.15× |
| kanban | -0.73% | 1.49× | 1.53× |
| mindmap | -0.95% | 2.13× | 2.49× |
| packet | -6.02% | 2.25× | 2.32× |
| pie | -3.14% | 1.88× | 2.11× |
| quadrant | -0.45% | 1.66× | 1.85× |
| requirement | +0.15% | 1.66× | 1.72× |
| sankey | -0.11% | 2.83× | 3.37× |
| sequence | -1.66% | 1.72× | 1.80× |
| state | -1.96% | 1.72× | 1.70× |
| timeline | -0.45% | 2.30× | 2.51× |
| xychart | -3.71% | 1.69× | 1.73× |
| agentflow | +1.26% | 1.23× | 1.32× |
| architecture | -1.15% | 2.62× | 2.66× |
| cynefin | -1.19% | 2.02× | 2.56× |
| ishikawa | -1.42% | 2.08× | 2.08× |
| radar | -3.14% | 2.52× | 2.59× |
| railroad | -1.16% | 1.42× | 1.45× |
| swimlane | +1.73% | 1.48× | 1.46× |
| treemap | +1.32% | 2.16× | 2.22× |
| treeview | -1.55% | 2.18× | 2.42× |
| usecase | -1.74% | 1.45× | 1.48× |
| venn | -2.95% | 2.38× | 2.40× |
| wardley | -0.50% | 1.85× | 1.97× |

[Complete output-contract timings](OUTPUT-BENCHMARK.md) preserve all Sample process reports. The [four-engine comparison](BENCHMARK.md) has its own 110-input / 22-type common-success subset; it does not replace the primary collection.

Across the 33 documentation types, render is **-0.66%**, parse **+0.54%**. The following table retains all 39 extended rows, including stress loads and regressions.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | +2.44% | +0.44% |
| tree: 50 nodes | +0.60% | +2.03% |
| mesh: 50 nodes, 20 extra labelled edges | +0.19% | +1.86% |
| tree: 500 nodes | +0.67% | +1.48% |
| mesh: 500 nodes, 200 extra labelled edges | -5.04% | +0.59% |
| corpus: agentflow-docs | -0.57% | -1.59% |
| corpus: agentflow-reference | -1.44% | +1.03% |
| corpus: architecture-docs | -1.26% | +2.40% |
| corpus: block-docs | +0.61% | -0.61% |
| corpus: c4-docs | +1.11% | -0.79% |
| corpus: class-docs | -2.04% | +0.36% |
| corpus: cynefin-docs | +0.95% | -1.46% |
| corpus: er-docs | -1.27% | -0.63% |
| corpus: eventmodeling-docs | -2.44% | +0.03% |
| corpus: flowchart-docs | +1.52% | +0.49% |
| corpus: gantt-docs | +2.51% | +3.52% |
| corpus: git-docs | +0.78% | +2.15% |
| corpus: info-docs | -5.09% | -0.11% |
| corpus: ishikawa-docs | -1.10% | +0.49% |
| corpus: journey-docs | -1.92% | -0.31% |
| corpus: kanban-docs | -1.33% | +0.78% |
| corpus: mindmap-docs | -1.72% | +0.60% |
| corpus: packet-docs | -6.01% | +3.16% |
| corpus: pie-docs | -0.35% | +2.75% |
| corpus: quadrant-docs | +1.68% | -0.13% |
| corpus: radar-docs | -2.50% | -0.69% |
| corpus: railroad-docs | +0.95% | -0.33% |
| corpus: requirement-docs | -1.19% | -1.04% |
| corpus: sankey-docs | -0.61% | -0.39% |
| corpus: sequence-docs | -0.88% | +0.61% |
| corpus: state-docs | +0.39% | +2.05% |
| corpus: swimlane-docs | +0.67% | -0.52% |
| corpus: timeline-docs | -0.43% | +0.82% |
| corpus: treemap-docs | -0.73% | +5.03% |
| corpus: treeview-docs | -0.23% | -0.61% |
| corpus: usecase-docs | +0.52% | +0.29% |
| corpus: venn-docs | +0.55% | +0.20% |
| corpus: wardley-docs | -0.99% | +0.68% |
| corpus: xychart-docs | -0.99% | +0.86% |

[Every extended process and verification record](research/number-text-optimization-2026-10-20/extended-comparison.json).

## Compiled size

Matched compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping. File and instruction text changes are separate.

| Artifact | File before → after (bytes) | File change | Text before → after (bytes) | Text change |
| --- | ---: | ---: | ---: | ---: |
| cli | 5,159,248 → 5,160,080 | +832 | 3,798,920 → 3,797,408 | -1,512 |
| ffi-static | 41,545,208 → 41,590,632 | +45,424 | 4,227,616 → 4,228,952 | +1,336 |
| ffi-dynamic | 4,749,184 → 4,766,240 | +17,056 | 3,515,116 → 3,516,308 | +1,192 |

File totals include data, symbols, embedded bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping and is not the host's final link increment. [Every section, configuration and artifact hash](research/number-text-optimization-2026-10-20/size-comparison.json), [development drivers separately](research/number-text-optimization-2026-10-20/driver-sizes.json).

## Allocation and process memory

The original default-CSS allocator probes are retained for all 33 Sample groups and 39 extended loads. An additional paired probe uses the **exact primary Arial16/CSS/fixed-clock contract** for all 167 inputs, with the same existing counted allocator and matched before/after core libraries. Each probe excludes input/options setup, runs five complete warm-up batches and one counted batch in a fresh process, and drops every render result. Requested bytes include full realloc requests and are not RSS. Each primary batch's SVG byte total also matches the independent Node reference.

| Set | Metric | Lower / equal / higher |
| --- | --- | ---: |
| Primary Arial16 Samples | allocations | 33 / 0 / 0 |
| Primary Arial16 Samples | reallocations | 0 / 33 / 0 |
| Primary Arial16 Samples | requestedBytes | 33 / 0 / 0 |
| Primary Arial16 Samples | peakAdditionalLiveBytes | 32 / 1 / 0 |
| Default Samples | allocations | 33 / 0 / 0 |
| Default Samples | reallocations | 0 / 33 / 0 |
| Default Samples | requestedBytes | 33 / 0 / 0 |
| Default Samples | peakAdditionalLiveBytes | 32 / 1 / 0 |
| Default extended | allocations | 39 / 0 / 0 |
| Default extended | reallocations | 0 / 39 / 0 |
| Default extended | requestedBytes | 39 / 0 / 0 |
| Default extended | peakAdditionalLiveBytes | 38 / 1 / 0 |

Every counted batch ends with zero net retained bytes; this is not a general leak audit. [Primary probe, all rows and dependency hashes](research/number-text-optimization-2026-10-20/allocations-primary.json), [probe source](research/number-text-optimization-2026-10-20/primary_allocations.rs), [default Samples](research/number-text-optimization-2026-10-20/allocations-samples.json), [default extended](research/number-text-optimization-2026-10-20/allocations-extended.json).

Separate default-CSS OS-resource checks run three fresh alternating processes per version/group, five warm-up batches and at least one second of complete rendering. Actual peak RSS and CPU totals include startup, input loading and warm-up; all batch counts and SVG byte totals match. These checks are distinct from the primary Arial16 allocation probe. Independent-process peaks are not summed.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 18 / 4 / 11 |
| extended | 22 / 2 / 15 |

Largest samples increases: samples: c4 4,521,984 → 4,718,592 bytes; samples: treemap 4,046,848 → 4,177,920 bytes; samples: radar 3,948,544 → 4,046,848 bytes.

Largest extended increases: corpus: sankey-docs 4,653,056 → 4,898,816 bytes; corpus: c4-docs 4,849,664 → 5,046,272 bytes; corpus: wardley-docs 4,800,512 → 4,997,120 bytes.

[All Sample processes](research/number-text-optimization-2026-10-20/resources-samples.json), [all extended processes](research/number-text-optimization-2026-10-20/resources-extended.json). No allocator reduction is substituted for an RSS improvement.

## Correctness and original acceptance

- Complete workspace: **3,243 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761 complete responses, 167 original Sample SVG strings, 39 extended verification records match**. Existing independent coordinate, decimal, root, token, label and semantic references remain unchanged.
- One new supplementary test reads **6,905 unchanged independent Node number strings**, seed `0x6d0d3e` / 4,096 random-bit cases. It checks stack spelling, exact Part size hints, appended bytes and string-style padding/precision. Existing public numeric tests also retain **6,905 string / 48,335 precision / 34,525 fixed** reference checks, plus the safe-integer fixture. No expected value is generated from Rust or replaced.
- The new project test is not an original-test port. Every original assertion, source, expected value, seed, loop, timeout and mapping remains.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; full original-test acceptance is incomplete.
- No original source, expected value, assertion, seed, loop count, timeout or mapping is changed. No original port is added or counted this round.

[Validation](research/number-text-optimization-2026-10-20/validation.json). Work toward the complete 2× API target and compiled-size objective remains active.
