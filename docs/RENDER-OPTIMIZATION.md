# SVG construction and parser allocation optimization

The baseline is `v0.1.0-pele1.7.0` (`0bbe183`). Both versions retain the pinned upstream behavior and all original tests. This report records paired implementation changes; the README displays current engine results only.

## Changes

- Prepare the SVG root's fixed substring searches once with memchr 2.8.3. Search for the first use of each required declaration instead of visiting every repeated fill/stroke variable. Byte searching is also used for longer arbitrary needles; short-label matching keeps its existing path.
- Write root font and weight declarations directly into the final output. Escape the caller's font lists individually; preserve the exact declaration order and content.
- Write Journey faces, actor dots, cards, and legend items into shared buffers. Section ranges reference byte offsets in one card buffer instead of concatenating a vector of card strings. Borrow actor names and CSS fill values, and reuse the body directly when there is no title.
- Borrow Journey and Timeline token text from the preprocessed source while parsing. Final public models still own their strings. Journey task parsing reads only the first two colon-separated fields, retaining the original behavior for additional fields.
- Separate API-return and contiguous UTF-8 output benchmarks. No output caching, reduced iteration counts, or relaxed correctness checks are introduced.

The production code remains pure Rust. memchr is an additional direct dependency already present in the lockfile; its version is pinned and its license is included in THIRD_PARTY_NOTICES.md.

## Paired extended benchmark

The unchanged 39-workload protocol runs three independent processes per version with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per operation. All six verification arrays match, including successful SVG hashes/dimensions and rejection fields.

Across 33 documentation groups, the equal-weight geometric mean after/before ratio is **0.87447 for full rendering** (12.55% lower time) and **0.93572 for parsing** (6.43% lower time). All 33 measured render medians are lower, but small changes near zero are observations rather than evidence of statistically significant improvement.

| Workload | Before (ms/batch) | After (ms/batch) | Render time change |
| --- | ---: | ---: | ---: |
| tree: 500 nodes | 1.83934 | 1.66039 | -9.73% |
| mesh: 500 nodes, 200 extra labelled edges | 5.00194 | 4.66302 | -6.78% |
| corpus: block-docs | 0.23274 | 0.20081 | -13.72% |
| corpus: journey-docs | 0.01613 | 0.01021 | -36.73% |
| corpus: sequence-docs | 0.56846 | 0.47218 | -16.94% |
| corpus: timeline-docs | 0.28324 | 0.22240 | -21.48% |

[All 39 groups and individual run medians](research/render-optimization-2026-10-20/extended-comparison.json) · [Inputs](research/render-optimization-2026-10-20/cases.json) · [Runtime, source and binary fingerprints](research/render-optimization-2026-10-20/summary.json). Six compressed reports retain every timing sample and verification record.

## Allocations

The allocator probe runs separately from timing on six unchanged sample groups. Inputs and options are prepared before tracking; five batches warm up the implementation, then one complete batch is measured. Requested bytes include complete realloc requests and are not RSS or peak memory. All measured batches return to zero retained bytes.

| Group | Allocations before / after | Reallocations before / after | Requested bytes before / after |
| --- | ---: | ---: | ---: |
| samples: flowchart | 5625 / 5613 | 687 / 680 | 463592 / 461442 |
| samples: block | 1573 / 1563 | 470 / 466 | 369209 / 367201 |
| samples: journey | 188 / 114 | 68 / 48 | 55658 / 46216 |
| samples: sankey | 1392 / 1386 | 210 / 210 | 358807 / 356959 |
| samples: sequence | 1122 / 1114 | 388 / 384 | 290299 / 288907 |
| samples: radar | 460 / 455 | 197 / 197 | 94657 / 93661 |

Journey's single sample reduces allocation calls by 39.36%, realloc calls by 29.41%, and requested bytes by 16.96%. The other groups primarily benefit from faster scanning and root construction; their allocation-count reductions are smaller. [Complete allocation data](research/render-optimization-2026-10-20/allocations.json).

## Current output contracts

All 167 Playground samples across 33 types are checked against independent reference SVG before timing. The paired all-sample render ratio is **0.90036** (9.96% lower time); this uses a different input set from the documentation corpus above, so the percentages are not combined.

The current equal-weight geometric mean Rust/Pele speedup across these 33 types is **1.48× at API return** and **1.57× for UTF-8-ready output**. Node's second endpoint includes materialization, encoding, and Buffer allocation. Rust already returns an owned UTF-8 String. See the [complete output-contract report](OUTPUT-BENCHMARK.md).

The [four-engine report](BENCHMARK.md) independently measures the 110 samples supported by all four engines, across 22 types. Its smaller common set must not be mixed with the 167-sample comparison.

A final lint cleanup removed three redundant references to borrowed token text. The rebuilt native driver has **identical executable instruction bytes** to the measured driver, and all 167 complete SVG strings were rechecked. The measured source hashes remain intact; [cleanup provenance](research/render-optimization-2026-10-20/lint-cleanup.json) records the final source and binary hashes.

## Correctness and acceptance

- Existing original inputs, assertions, seeds, limits, and mappings are unchanged.
- 761/761 complete current-release reference responses match.
- 167/167 sample SVG strings match for baseline Rust, current Rust, and the independent reference.
- A new independent reference covers 512 SVG-root cases: variable flags, repeated/malformed rectangle tags, user styles, Unicode, quoted fonts, and accessibility metadata. Expectations are generated only from the pinned upstream SVG root.
- Workspace: **3,191 passed, 0 failed, 5 ignored**. No-default-feature regression: **3 passed**. Original-timeout acceptance: **2,785 passed, 0 failed, 7 preserved skips, 4,034 unmapped, 0 unverified**. The complete gate exits 1 as required. [Validation evidence](research/render-optimization-2026-10-20/validation.json).

## Reproduce

Before editing production code, build and freeze the baseline drivers under an ignored directory. Build the candidate with the same release configuration, then run timing serially before allocation probes and tests:

```sh
ZM_BENCH_BEFORE=target/render-optimization/before/bench \
ZM_BENCH_OUTPUT=target/render-optimization/extended \
python3 tools/bench/compare-optimization.py

ZM_OUTPUT_BEFORE=target/render-optimization/before/cross-bench \
ZM_OUTPUT_REPORT=target/render-optimization/output \
python3 tools/bench/compare-output.py

cargo test --workspace --all-features --locked -- --test-threads=1
node tools/test-parity/check.mjs --require-complete
```

The complete gate remains nonzero while original tests are unmapped or preserve skip/todo status. Render equality and performance results do not replace full original-test acceptance.
