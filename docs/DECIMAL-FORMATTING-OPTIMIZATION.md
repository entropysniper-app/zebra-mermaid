# Decimal conversion, allocation and JavaScript rounding

Baseline: `97b74882ff6ec26e336945f83063405d1cd4f167`. Implementation: `87baab21861ae54d22a11de9035019ba505876d7`. This round removes intermediate decimal strings and corrects existing rounding differences exposed by additional independent Node references. The runtime remains pure Rust.

## Implementation and compatibility

Shortest decimal conversion now uses **zmij 1.0.23**, with JavaScript notation applied to its stack-buffer output. The version was already present in the lockfile through the development JSON tools; it is now also a pinned direct library dependency. Its MIT license is recorded in THIRD_PARTY_NOTICES.md. Precision formatting uses bounded stack scratch storage, with heap spill for larger requests, and allocates the final result once.

Fixed and significant-digit rounding test the binary significand exactly. Previously the tie correction parsed the rounded decimal back into an f64; at high precision that can collapse the lower candidate onto the input and hide a required increment. The exact test also handles rounding at negative decimal places, such as `250.toPrecision(1)`.

JavaScript shortest output chooses the closest decimal that round-trips, with an even significand when the closest candidates tie. Fixed/significant precision chooses the larger positive candidate at an exact midpoint before applying the sign. See [Number::toString](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring), [toPrecision](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.prototype.toprecision), and [toFixed](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.prototype.tofixed).

Examples independently reproduced in the baseline implementation and corrected against Node:

| Operation | Baseline output | Node / corrected output |
| --- | --- | --- |
| `toPrecision(250, 1)` | `2e+2` | `3e+2` |
| String conversion, bits `4308b14af7ae4a12` | `868791872768322.3` | `868791872768322.2` |
| `toPrecision`, bits `42d6bcc41e900001`, 20 digits | `100000000000000.01562` | `100000000000000.01563` |
| `toFixed`, same bits, 5 places | `100000000000000.01562` | `100000000000000.01563` |

The added fixture is generated only by Node v26.3.0: **6,905** values, **48,335** precision outputs and **34,525** fixed-place outputs, totaling **89,765** comparisons. It covers decimal-exponent neighbors, powers of two, exact ties, subnormals, nonfinite values, negative zero, and 4,096 seeded random bit patterns (`0x6d0d3e`). Selected precisions reach 100 digits. Existing fixtures and original assertions are unchanged; these are three supplementary project tests, not additional original-test mappings.

[Baseline discrepancies](research/decimal-formatting-optimization-2026-10-20/baseline-discrepancies.json) · [Reference generator](../tools/bench/generate-decimal-reference.mjs).

## Paired performance

The unchanged 39-group benchmark runs three independent processes per version in alternating order. It uses five warm-ups, at least 10 ms calibration, and at least nine samples plus 300 ms per operation. Verification and startup are excluded from timing. Compilation, tests, allocation probes, and CPU sampling do not overlap with timing.

All **39/39** groups retain identical verification records across all six runs. The 33-documentation-type render geometric-mean after/before ratio is **0.99778** (time change **-0.22%**); parse is **0.99957** (**-0.04%**). Only **16/33** documentation groups have lower observed render medians. Sankey rendering decreases 5.22% and Treemap 3.70%; the overall change is effectively flat at this sampling scale.

The separate 167-sample paired render ratio is **1.00703** (**+0.70%** time). Current Rust/Pele API speedup is **1.52×**, and UTF-8-ready speedup is **1.62×**. These measurements do **not establish a general acceleration**. The compatibility corrections and local allocation reductions are retained, and all slower rows remain visible. The **10× API target remains unmet**.

| Workload | Before render (ms) | After render (ms) | Render change | Parse change |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.12570 | 1.13218 | +0.58% | -0.80% |
| tree: 50 nodes | 0.16490 | 0.16318 | -1.05% | -1.61% |
| mesh: 50 nodes, 20 extra labelled edges | 0.30330 | 0.30685 | +1.17% | -0.98% |
| tree: 500 nodes | 1.68624 | 1.67163 | -0.87% | -0.83% |
| mesh: 500 nodes, 200 extra labelled edges | 4.64247 | 4.62110 | -0.46% | -1.58% |
| corpus: agentflow-docs | 0.41401 | 0.41700 | +0.72% | +0.51% |
| corpus: agentflow-reference | 2.61896 | 2.62673 | +0.30% | +0.31% |
| corpus: architecture-docs | 0.09536 | 0.09564 | +0.29% | +0.19% |
| corpus: block-docs | 0.18235 | 0.18096 | -0.76% | +0.62% |
| corpus: c4-docs | 0.34811 | 0.34819 | +0.02% | -0.23% |
| corpus: class-docs | 0.52929 | 0.53142 | +0.40% | -0.30% |
| corpus: cynefin-docs | 0.06497 | 0.06466 | -0.49% | +0.28% |
| corpus: er-docs | 0.46935 | 0.47641 | +1.50% | +0.37% |
| corpus: eventmodeling-docs | 0.14700 | 0.14677 | -0.16% | +0.38% |
| corpus: flowchart-docs | 1.13379 | 1.12523 | -0.75% | -0.80% |
| corpus: gantt-docs | 0.21429 | 0.21297 | -0.61% | +0.73% |
| corpus: git-docs | 0.77232 | 0.77219 | -0.02% | -0.95% |
| corpus: info-docs | 0.00202 | 0.00208 | +2.92% | +0.78% |
| corpus: ishikawa-docs | 0.01180 | 0.01191 | +0.93% | -0.34% |
| corpus: journey-docs | 0.01000 | 0.01004 | +0.46% | +1.33% |
| corpus: kanban-docs | 0.04134 | 0.04142 | +0.21% | +0.40% |
| corpus: mindmap-docs | 0.06130 | 0.06020 | -1.80% | -0.42% |
| corpus: packet-docs | 0.03412 | 0.03438 | +0.77% | +0.94% |
| corpus: pie-docs | 0.01603 | 0.01571 | -1.99% | -0.34% |
| corpus: quadrant-docs | 0.03617 | 0.03580 | -1.02% | +0.07% |
| corpus: radar-docs | 0.03391 | 0.03381 | -0.30% | -0.94% |
| corpus: railroad-docs | 0.25854 | 0.25764 | -0.35% | +0.33% |
| corpus: requirement-docs | 0.20790 | 0.20878 | +0.42% | -0.31% |
| corpus: sankey-docs | 0.14101 | 0.13365 | -5.22% | -0.06% |
| corpus: sequence-docs | 0.42065 | 0.42188 | +0.29% | +0.56% |
| corpus: state-docs | 0.51672 | 0.51749 | +0.15% | +1.16% |
| corpus: swimlane-docs | 0.39058 | 0.38970 | -0.23% | -1.91% |
| corpus: timeline-docs | 0.21636 | 0.21652 | +0.07% | -0.42% |
| corpus: treemap-docs | 0.09420 | 0.09072 | -3.70% | -2.80% |
| corpus: treeview-docs | 0.07910 | 0.07889 | -0.26% | -0.48% |
| corpus: usecase-docs | 0.63210 | 0.64110 | +1.42% | +0.62% |
| corpus: venn-docs | 0.20144 | 0.20278 | +0.66% | +0.15% |
| corpus: wardley-docs | 0.21285 | 0.21126 | -0.75% | -1.60% |
| corpus: xychart-docs | 0.10655 | 0.10669 | +0.13% | +1.19% |

[All run medians](research/decimal-formatting-optimization-2026-10-20/extended-comparison.json) · [Exact inputs](research/decimal-formatting-optimization-2026-10-20/cases.json) · [Source, binary and runtime fingerprints](research/decimal-formatting-optimization-2026-10-20/summary.json). Six compressed reports retain all individual samples.

## Allocation measurements

Allocation probes run separately: five warm-up batches followed by one counted batch, with input/options preparation excluded and results dropped. Requested bytes include full realloc requests; they are not RSS. Four of 33 sample types make fewer allocation calls and request fewer bytes; the other 29 are unchanged. No sample type increases either measure. One live-byte peak is lower and 32 are unchanged. Every sample and extended batch returns to zero retained bytes; this is a bounded observation, not complete leak testing.

| Sample type | Allocations before → after | Reallocations before → after | Requested bytes before → after | Additional live-byte peak before → after |
| --- | ---: | ---: | ---: | ---: |
| pie | 304 → 295 | 136 → 136 | 56,955 → 56,886 | 11,879 → 11,879 |
| sankey | 1,302 → 1,044 | 213 → 213 | 355,380 → 353,405 | 119,850 → 119,850 |
| treemap | 923 → 851 | 434 → 326 | 187,198 → 185,074 | 15,339 → 15,339 |
| usecase | 5,872 → 5,869 | 1,027 → 1,027 | 735,548 → 735,525 | 79,580 → 79,577 |

[All 33 sample allocation groups](research/decimal-formatting-optimization-2026-10-20/allocations-samples.json) · [Sample inputs](research/decimal-formatting-optimization-2026-10-20/sample-cases.json) · [All 39 extended allocation groups](research/decimal-formatting-optimization-2026-10-20/allocations-extended.json).

The baseline CPU evidence is reused from the fingerprint-verified previous implementation. New CPU sampling covers Treemap, XY chart, and Flowchart. Inclusive stack counts include descendants and must not be added as exclusive time percentages. [Baseline](research/decimal-formatting-optimization-2026-10-20/cpu-before.json) · [Current](research/decimal-formatting-optimization-2026-10-20/cpu-after.json).

## Validation and reproduction

- Workspace: **3,198 passed, 0 failed, 5 ignored**. Clippy completes with existing repository warnings.
- **761/761** current-release complete responses, **167/167** sample SVGs, and **39/39** extended verification groups match. The normal diagram corpus is unchanged by the numerical edge-case corrections.
- No-default-feature regression: **4 passed**; unsupported-type rejection is not implementation coverage.
- Original-timeout acceptance: **2,785 passed, 0 failed, 7 preserved skips, 4,034 unmapped, 0 unverified**. Two original skips execute successfully but remain excluded from acceptance passes. The complete gate exits **1**, with `complete: false`.
- Original test inputs, assertions, fixtures, seeds, iterations, mappings and thresholds are unchanged. No production source changed after the final builds.

[Validation evidence](research/decimal-formatting-optimization-2026-10-20/validation.json). Full original-test acceptance remains incomplete, independently of the performance target.

```sh
node tools/bench/generate-decimal-reference.mjs
cargo test --workspace --all-features --locked -- --test-threads=1
node tools/test-parity/check.mjs --require-complete

ZM_BENCH_BEFORE=target/numeric-optimization-4/before/bench \
ZM_BENCH_AFTER=target/numeric-optimization-4/after/bench \
ZM_BENCH_OUTPUT=target/numeric-optimization-4/final/extended \
python3 tools/bench/compare-optimization.py

ZM_OUTPUT_BEFORE=target/numeric-optimization-4/before/cross-bench \
ZM_OUTPUT_NATIVE=target/numeric-optimization-4/after/cross-bench \
ZM_OUTPUT_REPORT=target/numeric-optimization-4/final/output \
python3 tools/bench/compare-output.py
```

Freeze baseline programs before editing, and run timing separately from compilation and tests. The [output-contract report](OUTPUT-BENCHMARK.md) and [four-engine comparison](BENCHMARK.md) show current results on their respective fixed cohorts.
