# Text metrics experiment

The integer-advance and incremental-prefix candidate was **rejected**. It passed compatibility checks but did not improve full rendering on the fixed input sets. Production code remains at the verified decimal-formatting implementation. The independent Node metrics fixture and its supplementary test are retained for future work.

The baseline is `377cfc8fbdfa501934217152e91672ff69675a8e`; Pele is pinned to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Node v26.3.0 and rustc 1.96.1 were used. [The candidate patch](research/text-metrics-experiment-2026-10-20/candidate.patch) and [source/binary fingerprints](research/text-metrics-experiment-2026-10-20/summary.json) make the rejected implementation identifiable separately from the current library.

## Hypothesis and candidate

Fresh baseline CPU sampling found label layout in 677/3,603 C4 samples and 490/3,598 Kanban samples; wrapping appeared in 430 and 333, respectively. These are inclusive sampled frames, not exclusive CPU percentages, and cannot be added together. [Baseline sampling](research/text-metrics-experiment-2026-10-20/cpu-before.json) and [candidate sampling](research/text-metrics-experiment-2026-10-20/cpu-after.json) retain the four workloads.

The candidate accumulated default font advances as integers before scaling, and reused exact advance units when extending a same-style wrapping prefix. It selected this path only for the built-in measurer. Public custom-measurer construction kept the original callback path, including nonadditive widths and callback order. A very-large-input fallback retained floating-point accumulation outside the proven exact-integer bound. No parsed model or SVG was cached.

## Results

All original input groups and all regressions are retained in the [39-group comparison](research/text-metrics-experiment-2026-10-20/extended-comparison.json). The benchmark used three independent processes per version, alternating order, five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, allocation probes and CPU sampling ran outside timed measurements.

| Input set / operation | Candidate / baseline time | Observed change |
| --- | ---: | ---: |
| 33 documentation types, render | 1.00253 | +0.25% |
| 33 documentation types, parse | 1.00807 | +0.81% |
| All 167 original Samples, render | 1.00677 | +0.68% |

Only 15/33 documentation render medians were lower. Treemap documentation rendering was 4.80% slower; Venn was 1.64% faster. Small observed differences are not statistical-significance claims. The candidate's complete 33-type Sample comparison was API **1.51907×**, UTF-8-ready **1.61625×** versus Pele; these rejected-candidate measurements do not replace the [current production results](OUTPUT-BENCHMARK.md).

Allocation counts, reallocations, cumulative requested bytes and peak additional live bytes were identical for every one of the 33 Sample groups and all 39 extended groups. Every measured batch ended with zero net retained bytes. These allocator measurements are separate from RSS and do not establish general absence of leaks. See [Sample allocations](research/text-metrics-experiment-2026-10-20/allocations-samples.json) and [extended allocations](research/text-metrics-experiment-2026-10-20/allocations-extended.json).

The additional control paths and state did not earn their complexity. The entire production candidate was restored to the baseline after the frozen-binary measurements and validation. Subsequent work will target allocations and copying exposed by the call stacks.

## Compatibility and acceptance

- Candidate workspace: **3,201 passed, 0 failed, 5 ignored**; no-default-features regressions: 4 passed. Clippy completed with existing repository warnings.
- **761/761 complete reference responses, 167/167 Sample SVGs and 39/39 extended verification records matched**.
- Original independent-timeout acceptance: **2,785 passed, 0 failed, 7 preserved upstream skips, 4,034 unmapped, 0 unverified**. Strict completeness gate exited 1 with `complete: false`. Full original-test acceptance remains unfinished.
- The independent metrics reference contains **4,427 valid Rust strings × 10 styles × 7 font sizes = 309,890 exact binary64 comparisons**, including fixed-seed random scalars, mixed strings, ASCII controls, CJK, combining marks, supplementary characters and width-boundary neighbors. Seed: `0x51a710`. The generator verifies the pinned clean upstream revision before recording expectations. This supplementary test does not increase the original-test mapping count and does not waive isolated UTF-16 surrogate coverage.
- The retained metrics test also passes against the restored production implementation. Restored workspace results are recorded separately in the [validation evidence](research/text-metrics-experiment-2026-10-20/validation.json).

[Validation record](research/text-metrics-experiment-2026-10-20/validation.json), [exact extended inputs](research/text-metrics-experiment-2026-10-20/extended-cases.json), [Sample inputs and all output-contract timings](research/text-metrics-experiment-2026-10-20/output/cases.json), and [four-engine outcomes](research/text-metrics-experiment-2026-10-20/four-engine/comparison.json) remain available. The common four-engine subset is separate from the full 33-type target.

```sh
node tools/compat/gen-metrics-regression.mjs
cargo test -p zebra-mermaid --test metrics_reference --locked
cargo test --workspace --all-features --locked -- --test-threads=1
node tools/test-parity/check.mjs --require-complete
```

To reproduce the rejected candidate, apply its patch to the baseline and add the retained fixture, generator and supplementary integration test. Build and freeze the baseline/candidate drivers before running `tools/bench/compare-optimization.py` and `tools/bench/compare-output.py` with their respective binary-path environment variables. Never compare a restored source tree to the candidate fingerprint as though they were identical.
