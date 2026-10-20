# Style resolution and Sequence token allocation optimization

The baseline is `488bdee08ea9d6c19fe034c6c1596ee9c90cd349`. This round reduces temporary allocations in author styles, Block hierarchy construction, and Sequence parsing. The production runtime remains pure Rust, with the same public model ownership and pinned reference behavior. The README contains current engine comparisons only.

## Implementation

- Author-style resolution uses fixed slots for the existing property allowlists, preserving first-insertion order when a declaration is replaced. Property names and values borrow the input where possible. ASCII case-insensitive searches avoid allocating lowercase copies, and escaped values are written directly into the final attributes. Unicode property lowercasing, `!important` removal, unsafe-value rejection, and declaration order retain their existing behavior.
- The Sequence lexer records UTF-16 spans. The parser borrows complete UTF-8 ranges and retains lossy UTF-16 decoding for a span that splits a surrogate pair. Actor references remain borrowed through the private statement list; public models still store owned strings. Original token positions, statement order, repeated participant applications, and errors are preserved.
- Block construction appends child indices directly to the current hierarchy frame and avoids redundant identifier copies. Rendering looks up the default class directly instead of allocating its name for each block.

The baseline CPU samples identified style resolution and hierarchy construction in Block, and token emission and parsing in Sequence. These are sampled inclusive stacks, not additive time fractions. [Sanitized baseline profile](research/render-optimization-round2-2026-10-20/cpu-before.json).

## Paired timing

The unchanged extended benchmark runs 39 workload groups, including 33 documentation groups. Three independent processes per version alternate order. Each process uses five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per operation. Timing runs do not overlap with allocation instrumentation or correctness tests. All six runs have identical verification records for **39/39 groups**, including successful SVG hashes and dimensions and expected rejection fields.

| Documentation group | Render before → after (ms/batch) | Render time change | Parse before → after (ms/batch) | Parse time change |
| --- | ---: | ---: | ---: | ---: |
| Block | 0.20221 → 0.18968 | -6.20% | 0.07244 → 0.06833 | -5.68% |
| Sequence | 0.48065 → 0.42913 | -10.72% | 0.15826 → 0.11402 | -27.96% |

Across all 33 documentation groups, the equal-weight geometric mean after/before ratio is **0.99070 for rendering** (0.93% less time) and **0.99053 for parsing** (0.95% less time). Twenty-one groups have a lower observed render median. This small aggregate change does not establish a statistically significant general speedup; the clear gains are concentrated in the targeted paths.

All slower rows remain in the evidence. The largest observed render increases are swimlane documentation (+2.47%), the 50-node tree (+2.33%), and ER documentation (+1.98%). Three process medians are insufficient to distinguish these small differences from run variation. The full results are retained without substituting selected runs or changing the workload set.

[All 39 groups and run medians](research/render-optimization-round2-2026-10-20/extended-comparison.json) · [Exact inputs](research/render-optimization-round2-2026-10-20/cases.json) · [Runtime, source and binary fingerprints](research/render-optimization-round2-2026-10-20/summary.json). The directory also contains all six compressed reports with individual timing samples.

## Allocation measurements

The allocation counter runs separately from timing. Inputs and options are prepared before tracking; five batches warm up the implementation, then one complete batch is measured. Requested bytes include complete realloc requests. They are not RSS, and the allocation-count reduction is not a proportional reduction in peak memory.

| Sample group | Allocation calls before → after | Realloc calls before → after | Requested bytes before → after | Additional live-byte peak before → after |
| --- | ---: | ---: | ---: | ---: |
| Block | 1,563 → 1,240 (-20.67%) | 466 → 450 | 367,201 → 360,564 | 38,993 → 38,975 |
| Sequence | 1,114 → 810 (-27.29%) | 384 → 340 | 288,907 → 279,642 | 37,351 → 37,351 |

The larger documentation batches reduce Block allocation calls from 3,981 to 3,340 and Sequence from 7,158 to 5,544. Their live-byte peaks remain unchanged. All six sample batches and all 39 extended batches return to zero retained bytes after dropping results; this is a bounded probe, not comprehensive leak testing.

[Six sample groups](research/render-optimization-round2-2026-10-20/allocations-samples.json) · [Sample inputs](research/render-optimization-round2-2026-10-20/sample-cases.json) · [All extended allocation groups](research/render-optimization-round2-2026-10-20/allocations-extended.json).

## Current engine comparison

The independent 167-sample comparison verifies the complete SVG from baseline Rust, current Rust, and Pele before timing. All **167/167** match. On this sample set, the paired Rust render after/before ratio is **0.98614** (1.39% less time). These inputs differ from the documentation corpus, so the percentages are not combined.

Across its 33 types, the current equal-weight geometric mean Rust/Pele speedup is **1.50× at API return** and **1.60× for contiguous UTF-8 output**. The second Node endpoint includes string materialization, UTF-8 encoding, and Buffer allocation; Rust already returns an owned UTF-8 String. See the [output-contract report](OUTPUT-BENCHMARK.md).

The [four-engine comparison](BENCHMARK.md) was also rerun with the final candidate. It uses the smaller common set of 110 samples across 22 types. Every engine attempts all 167 inputs; unsupported or misidentified outputs remain in its coverage report.

## Correctness and acceptance

- Workspace: **3,193 passed, 0 failed, 5 ignored**.
- Current-release reference: **761/761** complete responses match, including expected rejections. Sample reference: **167/167** complete SVG strings match.
- A new independent fixture contains **2,902 author-style cases**, including 2,048 seeded cases (`0x57a1e`), property/value combinations, duplicate declarations, Unicode, escaping, and unsafe values formed by `!important` removal. Expectations come only from the pinned reference implementation.
- A new Sequence regression compares borrowed token decoding with UTF-16 lossy decoding at every span boundary in a source containing supplementary characters, CJK, combining marks, and a BOM. Existing Sequence lexer/parser/stress assertions remain unchanged.
- No-default-feature regression: **4 passed**. Disabled-diagram rejection is not counted as implementation coverage.
- Original-test acceptance: **2,785 passed, 0 failed, 7 preserved skips, 4,034 unmapped, 0 unverified**. Two original skips execute successfully but remain excluded from acceptance passes. The complete gate exits **1**, with `complete: false`.
- Original test inputs, assertions, seeds, limits, mappings, and frozen fixtures are unchanged. The new reference fixture and boundary regression are supplementary project tests, not additional original-test mappings.
- Clippy completes with existing repository warnings; the Block guard that rejects NaN width is preserved. No production source changes were made after the final release builds and measurements.

[Validation evidence and original-test fingerprints](research/render-optimization-round2-2026-10-20/validation.json).

## Reproduce

Build and freeze the baseline drivers before editing production code. Build the candidate with the same release configuration, then run timing serially before allocation probes and tests:

```sh
ZM_BENCH_BEFORE=target/render-optimization-2/before/bench \
ZM_BENCH_OUTPUT=target/render-optimization-2/final/extended \
python3 tools/bench/compare-optimization.py

ZM_OUTPUT_BEFORE=target/render-optimization-2/before/cross-bench \
ZM_OUTPUT_REPORT=target/render-optimization-2/final/output \
python3 tools/bench/compare-output.py

python3 tools/bench/run-cross.py
cargo test --workspace --all-features --locked -- --test-threads=1
cargo test -p zebra-mermaid --no-default-features \
  --test mermaid_type --test svg_root --test style_regression
node tools/test-parity/check.mjs --require-complete
```

Generate the additional style reference with `node tools/compat/gen-style-regression.mjs` only from the pinned, clean upstream source. The complete gate remains nonzero while original tests are unmapped or preserve skip/todo status.
