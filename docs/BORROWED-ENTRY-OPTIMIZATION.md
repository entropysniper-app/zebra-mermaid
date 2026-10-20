# Borrowed insertion keys

This round reduces compiled artifact sizes and temporary key allocations, **not a demonstrated overall speedup**. On all 167 unchanged Samples / 33 types, paired native render time is **1.00584× (+0.58%)**. Documentation render is **0.99275× (−0.72%)**, parse **0.99782× (−0.22%)**. Current reference speedup is **API 1.79×, UTF-8 ready 1.90×**; the **2× API-return target remains unmet**. Small changes are not statistical-significance claims.

Baseline `9497f7eda18b7029332a9f57d73546c5cb6520dd`, production code `9de7ca548364b364d4d0f5149af48c1eacd242bc`; candidate `c1ca29b903feba11145f179a38ed587fa1b82554`. Pele is fixed to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`; Node v26.3.0, rustc 1.96.1. [Full fingerprints](research/borrowed-entry-optimization-2026-10-20/summary.json).

## Implementation and invariants

Fresh primary-contract CPU captures covered Agentflow, Flowchart, State, ER and Class. Source inspection confirmed `entry(key.to_owned())` allocated keys even for occupied entries. **24 sites in 12 files** now use one private borrowed-entry helper. It hashes the borrowed `str` with that exact map's existing builder, looks up using normal equality, and constructs a `String` plus the original value factory only on a vacant entry. String/str use the same hash representation; the provided hash is used consistently for lookup and insertion. No hasher/seed, ID normalization, insertion order, factory semantics or public map type is changed. No hashes or outputs are cached.

The helper uses the safe `IndexMap` raw-entry v1 API of the already locked dependency; no dependency/version or unsafe code was added. Tests mix normal owned entries with borrowed entries, remove/reinsert keys, verify all lookup forms/order, force collisions, and check lazy factories plus panic behavior. Seed `0x51e17`, 512 operations for each normal/collision builder. These two supplemental tests are not upstream ports. [Every converted site](research/borrowed-entry-optimization-2026-10-20/conversion.json).

## Timing and all regressions

Three fresh alternating processes per engine/version; five warm-ups, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. All 167 original samples, 33 types, 39 extended loads, full parsing/layout/SVG, options, precision and original limits remain. No raw run is discarded/replaced. Builds, tests, CPU sampling, allocation probes and RSS checks are separate from latency.

The actual preflight and postflight found no listed build/test driver; these limited snapshots do not prove sustained host idleness. [Conditions](research/borrowed-entry-optimization-2026-10-20/timing-conditions.json). CPU capture source/input/options and SVG byte totals are matched. [Before](research/borrowed-entry-optimization-2026-10-20/cpu-before.json), [after](research/borrowed-entry-optimization-2026-10-20/cpu-after.json).

Sample medians fell in nine types and rose in 24. Agentflow **−2.38%**, Swimlane **−2.86%**, ER **−0.64%** coexist with Flowchart **+1.53%**, Treemap **+3.12%**, Usecase **+2.34%**. Neither this subset nor fewer allocations replaces the full-type result. The independent reference is remeasured every round; cross-round absolute ratios do not establish a native gain.

| Type | Native time change | API speedup | UTF-8 speedup |
| --- | ---: | ---: | ---: |
| flowchart | +1.53% | 1.61× | 1.68× |
| block | -0.11% | 1.32× | 1.38× |
| c4 | +0.27% | 1.52× | 1.57× |
| class | -0.34% | 1.50× | 1.56× |
| er | -0.64% | 1.47× | 1.48× |
| eventmodeling | +0.89% | 2.16× | 2.68× |
| gantt | +0.71% | 1.79× | 1.79× |
| git | +0.38% | 2.02× | 1.96× |
| info | -0.85% | 1.68× | 2.08× |
| journey | +0.36% | 1.98× | 2.06× |
| kanban | +2.32% | 1.37× | 1.43× |
| mindmap | +0.80% | 2.06× | 2.47× |
| packet | +1.31% | 2.03× | 2.11× |
| pie | +0.20% | 1.85× | 2.08× |
| quadrant | +0.28% | 1.67× | 1.74× |
| requirement | +0.46% | 1.62× | 1.65× |
| sankey | +1.35% | 2.76× | 3.33× |
| sequence | +1.77% | 1.65× | 1.69× |
| state | +0.01% | 1.67× | 1.70× |
| timeline | -0.02% | 2.23× | 2.37× |
| xychart | +0.48% | 1.61× | 1.66× |
| agentflow | -2.38% | 1.25× | 1.28× |
| architecture | +0.63% | 2.54× | 2.63× |
| cynefin | +3.69% | 1.83× | 2.37× |
| ishikawa | +0.60% | 1.88× | 1.95× |
| radar | -0.02% | 2.47× | 2.55× |
| railroad | -0.96% | 1.38× | 1.41× |
| swimlane | -2.86% | 1.45× | 1.45× |
| treemap | +3.12% | 2.12× | 2.18× |
| treeview | +3.14% | 2.11× | 2.29× |
| usecase | +2.34% | 1.39× | 1.45× |
| venn | +0.14% | 2.33× | 2.37× |
| wardley | +0.98% | 1.81× | 1.86× |

[Current full output-contract report](OUTPUT-BENCHMARK.md) retains all raw Sample processes; [four-engine results](BENCHMARK.md) use a separate 110-input / 22-type common set and do not substitute for the primary target.

| Extended workload | Render change | Parse change |
| --- | ---: | ---: |
| docs: all flowchart examples | +1.14% | +1.14% |
| tree: 50 nodes | +2.38% | +0.01% |
| mesh: 50 nodes, 20 extra labelled edges | -0.40% | -3.66% |
| tree: 500 nodes | -1.35% | -2.02% |
| mesh: 500 nodes, 200 extra labelled edges | +0.63% | -3.74% |
| corpus: agentflow-docs | +0.00% | -2.03% |
| corpus: agentflow-reference | +0.11% | -1.24% |
| corpus: architecture-docs | -1.35% | +0.01% |
| corpus: block-docs | -1.07% | -1.02% |
| corpus: c4-docs | -0.09% | -4.60% |
| corpus: class-docs | -2.25% | -2.31% |
| corpus: cynefin-docs | -2.07% | -0.23% |
| corpus: er-docs | +0.68% | -1.13% |
| corpus: eventmodeling-docs | -0.30% | +4.77% |
| corpus: flowchart-docs | +1.37% | -0.58% |
| corpus: gantt-docs | +0.62% | +0.53% |
| corpus: git-docs | -0.43% | +1.17% |
| corpus: info-docs | -0.13% | +1.25% |
| corpus: ishikawa-docs | -3.49% | +0.13% |
| corpus: journey-docs | -1.30% | +0.58% |
| corpus: kanban-docs | -1.24% | -0.37% |
| corpus: mindmap-docs | -0.11% | +1.28% |
| corpus: packet-docs | -3.91% | -0.45% |
| corpus: pie-docs | -0.06% | -0.97% |
| corpus: quadrant-docs | -1.42% | -4.66% |
| corpus: radar-docs | -1.29% | +0.90% |
| corpus: railroad-docs | -0.98% | -0.67% |
| corpus: requirement-docs | -0.18% | +1.23% |
| corpus: sankey-docs | -0.84% | -1.15% |
| corpus: sequence-docs | -1.00% | +1.09% |
| corpus: state-docs | -0.64% | -2.56% |
| corpus: swimlane-docs | -0.27% | -0.83% |
| corpus: timeline-docs | -1.67% | +0.59% |
| corpus: treemap-docs | +0.60% | +0.94% |
| corpus: treeview-docs | +0.72% | +0.88% |
| corpus: usecase-docs | -0.45% | +0.21% |
| corpus: venn-docs | -0.91% | +0.89% |
| corpus: wardley-docs | +0.41% | +0.93% |
| corpus: xychart-docs | -0.64% | -0.54% |

[Every extended row and independent process](research/borrowed-entry-optimization-2026-10-20/extended-comparison.json) remains, including all regressions.

## Compiled size and allocation tradeoffs

Same compiler, locked dependencies, release/thin LTO/one codegen unit, normalized source paths and all 33 types; no stripping.

| Artifact | File before → after (bytes) | Instruction text before → after (bytes) |
| --- | ---: | ---: |
| cli | 5,160,832 → 5,142,400 | 3,800,608 → 3,797,128 |
| ffi-static | 41,545,872 → 41,537,032 | 4,230,436 → 4,226,252 |
| ffi-dynamic | 4,765,824 → 4,749,136 | 3,517,824 → 3,513,640 |

CLI file bytes fall **18,432**, instruction text **3,480**. FFI dynamic/static file bytes fall **16,688 / 8,840**, text **4,184 / 4,184 bytes**. File totals include data, bitcode, symbols and archive overhead. Static-archive text sums all members before host selection/dead stripping, not the final host link increment. [All sections/hashes](research/borrowed-entry-optimization-2026-10-20/size-comparison.json), [separate driver sizes](research/borrowed-entry-optimization-2026-10-20/driver-sizes.json).

Sample allocation counts and requested bytes fall in **8 types**, with 25 unchanged; across extended loads they fall in **14 groups**, with 25 unchanged. Reallocations and additional-live peaks are unchanged everywhere. Examples: Agentflow allocations **7,175 → 7,026**, Flowchart **5,073 → 5,042**, ER **3,311 → 3,261**, State **4,744 → 4,663**. Every batch ends with zero net retained bytes; this is not a general leak audit. Requested allocator bytes are not RSS. [All Sample rows](research/borrowed-entry-optimization-2026-10-20/allocations-samples.json), [all extended rows](research/borrowed-entry-optimization-2026-10-20/allocations-extended.json).

## OS process resources

Separately, three fresh alternating processes per group/version use five warm-up batches and at least one second of complete rendering. Peak RSS and CPU totals include startup, input loading and warm-up. Batch success/rejection counts and SVG byte totals match. Peaks are not summed across independent processes.

| Set | Peak RSS lower / equal / higher |
| --- | ---: |
| samples | 17 / 4 / 12 |
| extended | 29 / 4 / 6 |

Largest observed increases: samples: c4 4,587,520 → 4,751,360 bytes; samples: venn 4,096,000 → 4,161,536 bytes; samples: wardley 4,472,832 → 4,538,368 bytes; corpus: agentflow-reference 5,521,408 → 5,685,248 bytes; mesh: 50 nodes, 20 extra labelled edges 5,570,560 → 5,652,480 bytes; corpus: c4-docs 4,997,120 → 5,046,272 bytes. This does not turn allocator reductions into a general RSS improvement claim. [Every Sample process](research/borrowed-entry-optimization-2026-10-20/resources-samples.json), [every extended process](research/borrowed-entry-optimization-2026-10-20/resources-extended.json).

## Correctness and original acceptance

- Full workspace: **3,239 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories unchanged.
- **761/761 complete responses, 167/167 original Sample SVGs, 39/39 extended verification records match**.
- Existing independent Node token/label/number/semantic fixtures remain unchanged. Both new map contracts pass with ordinary owned-entry interoperation, collision chains, order, lazy factories and panic recovery.
- Original isolated-timeout gate: **2,791 passed, zero failed, seven preserved skips, 4,028 unmapped, zero unverified**. Strict completeness exits one, `complete: false`; original-test acceptance remains incomplete.
- Original source, fixtures, assertions, expected values, seeds, loop counts, thresholds and test mappings remain unchanged. No new original port is counted this round.

[Validation](research/borrowed-entry-optimization-2026-10-20/validation.json). The measured size/allocation gains and latency tradeoff are recorded together while work toward the full 2× target continues.
