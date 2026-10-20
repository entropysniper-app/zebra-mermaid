# Shared preprocessing and SVG pipeline optimization

The baseline is `a937258565ae9451db177731da57476801301c6d`; the optimized implementation is `6df32f83153c20522a124ec3c73b44ee9b327a0f`. This round optimizes common code used across all 33 diagram types: preprocessing, error-source maps, SVG coordinate output, and label wrapping. Public preprocessing results and parsed models remain owned. Runtime dependencies, layout algorithms, text measurement order, output contracts, and the pinned reference version are unchanged.

## Changes and evidence

- Ordinary sources borrow their preprocessed text until the parser needs its final buffer. A conservative guard sends carriage returns, front matter candidates, comments/directives, and quoted HTML attributes through the full preprocessing path. Both comment views, configuration merging, source limits, and error positions retain their behavior.
- Each source-map step stores its first mapping point inline. Additional points use a vector; identity and suffix steps avoid separate mark allocations. Short parser maps reserve room for the following entity step, and default/empty-step distinctions and end clamping are preserved.
- An owned preprocessed buffer is reused when entity rewriting is unnecessary. Attribute quoting is skipped only when the source cannot contain the pattern it rewrites.
- SVG coordinates are written as one validated ASCII slice instead of individual character writes. Rounding and decimal conversion are unchanged.
- Label wrapping consumes pieces through an iterator. Entity-free text and non-markdown source text are borrowed where possible; trailing whitespace is removed in place. The same piece scanner, CJK punctuation rules, host measurement calls, and floating-point accumulation order are retained.

Ten representative CPU samples identified preprocessing and label construction as shared costs. C4 and Kanban also spend substantial time wrapping labels. Samples are inclusive call stacks, so their counts cannot be added as exclusive time fractions. [Before](research/shared-pipeline-optimization-2026-10-20/cpu-before.json) · [After](research/shared-pipeline-optimization-2026-10-20/cpu-after.json).

## All-type paired timing

The unchanged extended benchmark covers **39 workload groups**, including **33 documentation types**. Both versions run in three independent processes with alternating order. Each operation uses five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms. Timings exclude verification and process startup. Compilation, tests, allocation instrumentation, and CPU stack sampling do not overlap with timing.

All six runs preserve the same verification records for **39/39 groups**, including successful SVG hashes/dimensions and rejection fields. Across the 33 documentation groups, the equal-weight geometric mean after/before ratio is **0.97335 for rendering** (2.67% less time) and **0.93186 for parsing** (6.81% less time). **33/33** types have a lower observed render median.

These are observations for this workload set, not a universal or statistical-significance claim. All rows remain visible, including any slower medians. Individual run medians and raw timing samples are retained with the exact inputs and source/binary fingerprints.

| Workload | Render before (ms) | Render after (ms) | Render change | Parse change |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.15906 | 1.13432 | -2.13% | -5.91% |
| tree: 50 nodes | 0.16420 | 0.16599 | +1.09% | +1.39% |
| mesh: 50 nodes, 20 extra labelled edges | 0.30743 | 0.30626 | -0.38% | +1.06% |
| tree: 500 nodes | 1.69003 | 1.68868 | -0.08% | +0.77% |
| mesh: 500 nodes, 200 extra labelled edges | 4.69416 | 4.64764 | -0.99% | +3.34% |
| corpus: agentflow-docs | 0.42412 | 0.41985 | -1.01% | -2.56% |
| corpus: agentflow-reference | 2.70503 | 2.65025 | -2.03% | -4.09% |
| corpus: architecture-docs | 0.09730 | 0.09519 | -2.17% | -5.02% |
| corpus: block-docs | 0.19163 | 0.18324 | -4.38% | -10.49% |
| corpus: c4-docs | 0.35296 | 0.35253 | -0.12% | +2.19% |
| corpus: class-docs | 0.55080 | 0.53167 | -3.47% | -6.47% |
| corpus: cynefin-docs | 0.06676 | 0.06566 | -1.65% | -7.39% |
| corpus: er-docs | 0.48277 | 0.47558 | -1.49% | -3.86% |
| corpus: eventmodeling-docs | 0.15157 | 0.14753 | -2.67% | -6.76% |
| corpus: flowchart-docs | 1.16625 | 1.14220 | -2.06% | -6.71% |
| corpus: gantt-docs | 0.21934 | 0.21573 | -1.65% | -4.55% |
| corpus: git-docs | 0.78656 | 0.78093 | -0.72% | -1.35% |
| corpus: info-docs | 0.00232 | 0.00195 | -15.91% | -34.41% |
| corpus: ishikawa-docs | 0.01236 | 0.01194 | -3.40% | -6.07% |
| corpus: journey-docs | 0.01039 | 0.01005 | -3.23% | -13.05% |
| corpus: kanban-docs | 0.04215 | 0.04155 | -1.42% | -3.77% |
| corpus: mindmap-docs | 0.06350 | 0.06152 | -3.11% | -20.63% |
| corpus: packet-docs | 0.03480 | 0.03441 | -1.10% | -4.84% |
| corpus: pie-docs | 0.01669 | 0.01610 | -3.55% | -6.41% |
| corpus: quadrant-docs | 0.03699 | 0.03614 | -2.31% | -2.54% |
| corpus: radar-docs | 0.03540 | 0.03401 | -3.93% | -2.28% |
| corpus: railroad-docs | 0.26545 | 0.26212 | -1.25% | -4.34% |
| corpus: requirement-docs | 0.21504 | 0.21029 | -2.21% | -3.01% |
| corpus: sankey-docs | 0.14403 | 0.14119 | -1.97% | -4.36% |
| corpus: sequence-docs | 0.43480 | 0.42513 | -2.22% | -6.78% |
| corpus: state-docs | 0.53083 | 0.51799 | -2.42% | -2.59% |
| corpus: swimlane-docs | 0.39459 | 0.39216 | -0.62% | -1.60% |
| corpus: timeline-docs | 0.22556 | 0.21980 | -2.55% | -9.79% |
| corpus: treemap-docs | 0.09849 | 0.09468 | -3.87% | -4.40% |
| corpus: treeview-docs | 0.08289 | 0.08035 | -3.06% | -7.84% |
| corpus: usecase-docs | 0.65174 | 0.64212 | -1.48% | -1.37% |
| corpus: venn-docs | 0.20410 | 0.20074 | -1.65% | -12.09% |
| corpus: wardley-docs | 0.22161 | 0.21502 | -2.97% | -5.02% |
| corpus: xychart-docs | 0.10972 | 0.10848 | -1.13% | -6.33% |

[Complete comparison](research/shared-pipeline-optimization-2026-10-20/extended-comparison.json) · [Inputs](research/shared-pipeline-optimization-2026-10-20/cases.json) · [Method, versions and fingerprints](research/shared-pipeline-optimization-2026-10-20/summary.json). Six compressed files in the evidence directory retain every timing sample and verification record.

## Allocations across all 33 sample types

Allocation probes run separately using a counted allocator. Each performs five warm-up batches and then counts one complete batch. Input and option preparation are excluded, and each render result is dropped. Requested bytes include the complete new size of each realloc; this is neither RSS nor peak memory. The additional live-byte peak is reported separately.

**33/33** sample types make fewer allocation calls, and **33/33** request fewer cumulative bytes. The additional live-byte peak is unchanged for **27** types and slightly higher for **6**, with a maximum increase of **157 bytes**. No reduction in peak memory or RSS is claimed; buffer reuse can retain spare capacity even when it reduces allocation work. All 33 sample groups and all 39 extended groups return to zero retained bytes after their measured batches. This bounded observation is not comprehensive leak testing.

| Sample type | Allocation calls before → after | Realloc calls before → after | Requested bytes before → after | Additional live-byte peak before → after |
| --- | ---: | ---: | ---: | ---: |
| flowchart | 5,613 → 5,556 | 680 → 679 | 461,442 → 457,692 | 44,573 → 44,573 |
| block | 1,240 → 1,170 | 450 → 449 | 360,564 → 354,512 | 38,975 → 38,975 |
| c4 | 5,332 → 5,111 | 2,016 → 1,937 | 1,242,548 → 1,202,099 | 112,677 → 112,833 |
| class | 3,803 → 3,740 | 587 → 591 | 533,243 → 530,305 | 44,259 → 44,259 |
| er | 3,786 → 3,722 | 552 → 550 | 501,593 → 494,413 | 53,428 → 53,428 |
| eventmodeling | 1,177 → 1,117 | 521 → 515 | 333,202 → 327,060 | 29,019 → 29,019 |
| gantt | 2,594 → 2,541 | 660 → 659 | 321,483 → 317,617 | 46,184 → 46,184 |
| git | 4,005 → 3,945 | 1,330 → 1,330 | 829,860 → 824,371 | 64,778 → 64,778 |
| info | 62 → 42 | 11 → 9 | 6,153 → 4,803 | 1,278 → 1,278 |
| journey | 114 → 104 | 48 → 47 | 46,216 → 45,164 | 24,024 → 24,024 |
| kanban | 612 → 568 | 268 → 261 | 137,175 → 133,658 | 42,958 → 43,078 |
| mindmap | 855 → 776 | 455 → 445 | 257,302 → 249,355 | 36,611 → 36,617 |
| packet | 493 → 453 | 120 → 122 | 100,789 → 99,915 | 24,655 → 24,655 |
| pie | 334 → 304 | 137 → 136 | 58,597 → 56,955 | 11,879 → 11,879 |
| quadrant | 493 → 461 | 249 → 249 | 106,008 → 102,438 | 16,984 → 16,984 |
| requirement | 3,699 → 3,633 | 860 → 859 | 665,142 → 657,094 | 113,477 → 113,477 |
| sankey | 1,359 → 1,302 | 210 → 213 | 356,149 → 355,380 | 119,850 → 119,850 |
| sequence | 1,192 → 1,132 | 505 → 501 | 400,136 → 393,002 | 37,351 → 37,351 |
| state | 4,912 → 4,846 | 826 → 823 | 689,974 → 682,661 | 59,706 → 59,706 |
| timeline | 1,593 → 1,450 | 845 → 807 | 407,267 → 389,754 | 29,060 → 29,217 |
| xychart | 1,654 → 1,595 | 512 → 512 | 258,798 → 255,962 | 29,634 → 29,634 |
| agentflow | 7,928 → 7,853 | 1,023 → 1,018 | 813,980 → 806,112 | 66,688 → 66,691 |
| architecture | 1,561 → 1,501 | 435 → 429 | 421,106 → 412,024 | 66,284 → 66,284 |
| cynefin | 662 → 632 | 501 → 498 | 277,364 → 273,882 | 42,950 → 42,950 |
| ishikawa | 159 → 149 | 74 → 73 | 59,797 → 58,265 | 21,802 → 21,802 |
| radar | 455 → 425 | 197 → 198 | 93,661 → 92,405 | 13,319 → 13,319 |
| railroad | 2,324 → 2,264 | 1,304 → 1,298 | 635,769 → 628,193 | 39,164 → 39,164 |
| swimlane | 5,135 → 5,075 | 791 → 787 | 555,482 → 548,530 | 46,553 → 46,553 |
| treemap | 985 → 923 | 432 → 434 | 189,790 → 187,198 | 15,339 → 15,339 |
| treeview | 605 → 541 | 347 → 347 | 164,519 → 160,375 | 16,764 → 16,764 |
| usecase | 5,941 → 5,872 | 1,026 → 1,027 | 739,893 → 735,548 | 79,579 → 79,580 |
| venn | 1,541 → 1,479 | 202 → 199 | 144,420 → 139,676 | 17,652 → 17,652 |
| wardley | 1,460 → 1,402 | 713 → 709 | 318,802 → 310,726 | 37,254 → 37,254 |

[All sample allocation records](research/shared-pipeline-optimization-2026-10-20/allocations-samples.json) · [Sample inputs](research/shared-pipeline-optimization-2026-10-20/sample-cases.json) · [All 39 extended allocation records](research/shared-pipeline-optimization-2026-10-20/allocations-extended.json).

## Current engine results

All **167/167** original sample SVGs match between baseline Rust, current Rust, and the independent reference. The paired sample render after/before ratio is **0.98229**; this is a different corpus from the documentation groups above. **30/33** sample groups have lower observed medians. Small increases remain visible for flowchart (+0.46%), er (+0.54%), swimlane (+0.12%); they are not removed or substituted with selected runs.

Across these 33 types, current Rust/Pele equal-weight speedup is **1.53× at API return** and **1.62× for contiguous UTF-8 output**. The latter includes Node's string materialization, encoding, and Buffer allocation; Rust already returns a UTF-8 String. See the [output-contract comparison](OUTPUT-BENCHMARK.md).

The [four-engine comparison](BENCHMARK.md) is rerun independently on the 110 samples across 22 types that all four engines support. Its smaller common cohort must not be combined with the 167-sample results. The README presents only the current engine tables.

## Correctness and acceptance

- Workspace: **3,195 passed, 0 failed, 5 ignored**. Clippy completes with existing repository warnings; no warning was introduced in the changed code.
- Current-release reference: **761/761** complete responses match, including successful output and expected errors. Sample reference: **167/167** complete SVGs match.
- Two supplementary regressions cover source-map storage/empty steps/clamping and compare borrowed preprocessing with the full path across both text views and every byte offset. They do not count as original-test mappings.
- No-default-feature regression: **4 passed**; disabled-diagram rejection is not diagram implementation coverage.
- Original-timeout acceptance: **2,785 passed, 0 failed, 7 preserved skips, 4,034 unmapped, 0 unverified**. Two original skips execute successfully but remain excluded from acceptance passes. The complete gate exits **1**, with `complete: false`.
- Original tests, fixtures, mappings, assertions, seeds, iteration counts, and thresholds are unchanged. No production source changed after the final binaries were built.

[Validation and original-test fingerprints](research/shared-pipeline-optimization-2026-10-20/validation.json). Full original-test acceptance remains incomplete.

## Reproduce

Freeze baseline binaries before editing production code, using the same release configuration as the candidate. Run timing serially, then allocation probes and correctness checks:

```sh
ZM_BENCH_BEFORE=target/all-types-optimization-3/before/bench \
ZM_BENCH_AFTER=target/all-types-optimization-3/after/bench \
ZM_BENCH_OUTPUT=target/all-types-optimization-3/final/extended \
python3 tools/bench/compare-optimization.py

ZM_OUTPUT_BEFORE=target/all-types-optimization-3/before/cross-bench \
ZM_OUTPUT_NATIVE=target/all-types-optimization-3/after/cross-bench \
ZM_OUTPUT_REPORT=target/all-types-optimization-3/final/output \
python3 tools/bench/compare-output.py

python3 tools/bench/run-cross.py
cargo test --workspace --all-features --locked -- --test-threads=1
node tools/test-parity/check.mjs --require-complete
```

The profiler binary accepts a workload name and reads `ZM_PROFILE_CASES` (default `target/bench/cases.json`). Run the frozen before/after profilers on each entry of the published input files to reproduce allocation counts. CPU sampling uses `tools/bench/profile-cpu.py`; raw machine captures remain under ignored `target/`.
