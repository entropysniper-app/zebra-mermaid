# Inline storage for plain render labels

On all **167 unchanged Samples across all 33 diagram types**, this round reduced the equal-weight geometric mean rendering time by **4.49%** against the frozen Rust baseline. Observed medians fell in 27 types and rose in six. The current Rust/Pele speedup is **1.74× at API return** and **1.84× for UTF-8-ready output**. The 10× target remains unmet.

The baseline is `b202b944c1cc76116c74177ab2d5140dfde436de`; the measured implementation is `f1c5a83cede4d0cfd2a04d2a11848c9c9e128860`. Baseline binaries were built from identical production code at `9f1eefa`; their source and binary fingerprints were checked after baseline profiling and before reuse. Pele is fixed to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Runtime versions: Node v26.3.0, rustc 1.96.1.

## Evidence and implementation

Fresh external CPU captures covered Info, C4, Mindmap, Block and Sequence. Label layout and its allocation descendants appeared in each capture. The former plain-label fast path allocated three containers for a single span: the outer line vector, its span vector and the width vector. A private render label now stores that span and its line width inline. Rich labels retain owned vectors. The text itself remains owned, and no label, parsed model or rendered result is cached.

Public `Label` fields, ownership and layout helper signatures remain unchanged. Public and private storage use the same generic layout algorithm; SVG helpers share a borrowed view. Measurement callbacks retain their text, argument order and binary64 operations. Internal fitting, literal labels, restyling, titles and diagram renderers use the private representation. This removes container allocations for ordinary labels while increasing the size of a label stored in a collection; the measured memory regressions below are retained.

The [CPU captures and source hashes](research/compact-label-optimization-2026-10-20/summary.json) and [inclusive label/allocator frame counts](research/compact-label-optimization-2026-10-20/label-frames.json) are recorded. Counts include overlapping descendants and recursive occurrences; they are not additive exclusive CPU percentages. Label-layout occurrences fell from 292 to 208 in Info and 451 to 364 in Mindmap, but rose from 589 to 747 in C4 and from 140 to 144 in Sequence. These samples do not establish a universal improvement in label CPU cost.

## Independent numerical and callback checks

The new generator runs the unchanged pinned Node implementation, including Requirement's internal `restyle` function exposed by appending an export in the bundler's in-memory input. It records the upstream source hash and generates **6,048 label cases, 18,144 restyle cases and 4,108 rounding values**. Assertions compare binary64 values, full measurer callback traces and SVG both with and without an explicit icon resolver. Both public owned labels and private render labels are checked. Supplementary coverage includes custom non-additive measurement, wrapping, styled text, icons, CJK, emoji, controls, empty/absent inputs and signed zero; the seed is `0x18c0de`.

This independent reference exposed a pre-existing numerical mismatch before it was corrected: `Math.round(-0.1)` and a negative-size blank label retained negative zero in Node, while the old Rust implementation returned positive zero. The previous public label path failed the new reference before the fix. `util::js::round` now copies the input sign onto the rounded result. This is a compatibility correction, not a claim that every numeric result is unchanged. [The baseline discrepancy](research/compact-label-optimization-2026-10-20/baseline-rounding-discrepancy.json) is preserved. Existing original tests and fixture expectations were not changed.

The three supplementary project tests do not increase original-test acceptance. [Generator](../tools/compat/gen-render-label-reference.mjs) · [Independent Node fixture](../tests/fixtures/unit/render-label-reference.json.gz).

## Timing and regressions

Three independent processes per engine/version, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, CPU sampling and allocation instrumentation ran outside timing. Inputs, Arial metrics, CSS output, fixed UTC clock and complete render work were held constant.

| Set / operation | After / before time | Observed change |
| --- | ---: | ---: |
| All 167 Samples / 33 types, render | 0.95511 | −4.49% |
| 33 documentation types, render | 0.98839 | −1.16% |
| 33 documentation types, parse | 1.00351 | +0.35% |

All six Sample regressions are retained: **Treeview +4.44%, Eventmodeling +1.03%, Wardley +0.42%, Packet +0.34%, Agentflow +0.14%, Timeline +0.06%**. Documentation rendering improved in 24/33 types; its larger regressions include **Journey +12.70%, C4 +9.64% and Flowchart +5.16%**. The separate all-Flowchart-examples batch rose 9.29%. These measurements do not show that allocation reduction benefits every workload. Parsing algorithms were not changed, so its difference is not attributed to a parsing optimization. Small differences are not statistical-significance claims.

The [paired extended report](research/compact-label-optimization-2026-10-20/extended-comparison.json) retains all 39 rows and individual process medians. The [current full Sample report](OUTPUT-BENCHMARK.md) retains both endpoints and all 33 types. API speedups range from **Agentflow 1.22×** to **Sankey 3.26×**: 26 types are below 2×, seven are at least 2× and none reaches 10×. The independently measured four-engine common set remains 110 Samples / 22 types and does not replace the main target set.

## Allocation and binary-size tradeoffs

The separate counted allocator used identical inputs/options, five warm-up batches and one measured batch. Cumulative requested bytes include full realloc requests. These allocator metrics and driver file sizes are separate from RSS.

| Sample metric | Lower | Unchanged | Higher |
| --- | ---: | ---: | ---: |
| Allocation count | 30 | 3 | 0 |
| Reallocation count | 0 | 33 | 0 |
| Cumulative requested bytes | 26 | 2 | 5 |
| Peak additional live bytes | 28 | 4 | 1 |

Example Sample allocation counts are **C4 4,972 → 4,348**, **Sequence 1,132 → 988**, **Flowchart 5,556 → 5,439** and **Info 42 → 36**. Info's requested bytes fell **4,763 → 4,571**, and its peak additional live bytes fell **1,278 → 1,182**.

All increased Sample requested-byte results are retained: **Block 353,218 → 353,266; Kanban 135,474 → 135,858; State 650,565 → 652,293; Agentflow 796,526 → 797,198; Treeview 159,650 → 163,490**. Treeview's peak also rose **16,764 → 17,148 bytes**. Inline storage makes each stored label larger even when the rich variant is used.

Across 39 extended groups, allocations fell in 36 and were unchanged in three; reallocations were unchanged in all 39. Requested bytes fell in 31, were unchanged in three and rose in five; peaks fell in 33, were unchanged in five and rose in one. Every Sample and extended batch ended with zero net retained bytes. This is not a general leak audit or a claim that RSS decreased. [Every Sample allocation row](research/compact-label-optimization-2026-10-20/allocations-samples.json) and [every extended allocation row](research/compact-label-optimization-2026-10-20/allocations-extended.json) are preserved.

The cross-engine driver grew **9,310,832 → 9,360,112 bytes (+0.53%)**; the extended driver grew **5,017,008 → 5,066,064 bytes (+0.98%)**. The CPU and allocator drivers also grew by about 49 KB. All used the same release, thin LTO and single-codegen-unit settings. [Sizes, source and binary fingerprints](research/compact-label-optimization-2026-10-20/summary.json) are recorded.

## Compatibility and acceptance

- Full workspace: **3,207 passed, 0 failed, 5 ignored**. No-default-features regressions: four passed. Clippy completed with existing warnings and no added warning categories.
- **761/761 complete reference responses, 167/167 Sample SVGs, 39/39 extended verification records matched**.
- Existing original test bodies, fixtures, inputs, assertions, seeds, iteration counts, performance thresholds and mappings were preserved. Measured production-source fingerprints matched after all validation.
- Original independent-timeout gate: **2,785 passed, 0 failed, seven preserved upstream skips, 4,034 unmapped, zero unverified**. Strict completeness exited 1 with `complete: false`; full original-test acceptance remains unfinished.

[Validation](research/compact-label-optimization-2026-10-20/validation.json) · [Extended inputs](research/compact-label-optimization-2026-10-20/cases.json) · [Sample inputs and output verification](research/output-benchmark-2026-10-20-compact-label/comparison.json). All independent-process timings are retained in the associated evidence directories.

```sh
cargo test --workspace --all-features --locked -- --test-threads=1
cargo clippy --workspace --all-features --locked
node tools/test-parity/check.mjs --require-complete
```

For paired timing, freeze both versions and use `tools/bench/compare-optimization.py` with `ZM_BENCH_BEFORE`, `ZM_BENCH_AFTER` and `ZM_BENCH_OUTPUT`. Use `tools/bench/compare-output.py` with `ZM_OUTPUT_BEFORE`, `ZM_OUTPUT_NATIVE` and `ZM_OUTPUT_REPORT` for the full Sample contracts. Keep tests and profiling outside timed runs.
