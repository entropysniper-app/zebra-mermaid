# Literal label layout and SVG output

This round reduces temporary strings in literal-label helpers and left-aligned SVG output. The measured benefit is local to **C4** in the fixed corpus: Sample rendering time fell **2.79%**, documentation rendering fell **3.45%**, and allocator calls decreased. The all-type Sample result remained essentially flat. The 10× target is not met.

The baseline is `2e81b44c20ee5799acc021dddd28a3533df05474`; the verified implementation is `062c752bd2dbdd560e214ae5ad680409ea6529ae`. Baseline binaries were originally built from the identical production source at `87baab2`; every recorded source and binary hash was checked before reuse. Pele remains pinned to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Runtime versions: Node v26.3.0 and rustc 1.96.1.

## Change and evidence

Fresh CPU sampling covered Packet, Treeview, Eventmodeling and C4. C4's left-aligned SVG helper appeared in 152/3,548 thread samples. The candidate's append helper appeared in 70/3,607. These are inclusive sampled frames for differently compiled helpers, not exclusive CPU percentages or a statistical significance test. [Before](research/literal-label-optimization-2026-10-20/cpu-before.json) and [after](research/literal-label-optimization-2026-10-20/cpu-after.json) aggregates retain the four workloads.

- Left-aligned labels write escaped spans and coordinates directly into the destination. C4 and Treeview append into their existing buffers; Eventmodeling can format the label directly into its enclosing SVG. The public `left_label_svg` function still returns an owned `String`, and icon fallback output is preserved.
- Literal wrapping probes borrow source slices. ASCII uses byte offsets directly; Unicode uses a code-point boundary table. Only final line text is copied.
- Character wrapping measures the mutable candidate prefix and removes the last candidate character before committing an overflowing line. It keeps the original floating-point operations and the exact strings, sizes, styles and order seen by custom measurers, including nonadditive widths.

The latter wrapping paths serve optional narrow layouts. Their lack of allocator changes in the default corpus is reported below; no throughput improvement is claimed for unexercised optional branches. Independent references cover these helpers separately. The work does not cache parsed models, labels or rendered SVGs across requests.

## End-to-end timing

The protocol is unchanged: three independent processes per engine/version, alternating order, five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Compilation, tests, CPU sampling and allocator instrumentation ran outside timed measurements. Every original input is retained.

| Input set / operation | After / before time | Observed change |
| --- | ---: | ---: |
| 33 documentation types, render | 0.99536 | −0.46% |
| 33 documentation types, parse | 0.98770 | −1.23% |
| All 167 original Samples, render | 1.00112 | +0.11% |
| C4 documentation, render | 0.96553 | −3.45% |
| C4 Samples, render | 0.97213 | −2.79% |

Documentation render medians were lower for 22/33 types, while Sample medians were lower for 9/33. Documentation Timeline rendering was 1.54% slower and Info was 1.01% slower; Sample XYChart was 1.25% slower. All rows and individual process medians, including regressions, are preserved in the [extended comparison](research/literal-label-optimization-2026-10-20/extended-comparison.json) and [Sample comparison](research/output-benchmark-2026-10-20-literal-label/comparison.json). Parsing code did not change; its observed difference is retained without attributing it to this rendering change. Small differences are not statistical significance claims.

Across all 167 unchanged Samples and all 33 types, the current Rust/Pele equal-weight geometric mean is **API return 1.52207×**, **UTF-8 ready 1.61074×**. The API distribution ranges from **Info 1.13×** to **Sankey 2.37×**: 29 types are below 2×, four are at least 2×, and none reaches 10×. UTF-8-ready timing includes Node's materialization, encoding and Buffer allocation. See the [complete current table](OUTPUT-BENCHMARK.md). The four-engine comparison separately retains its 110-Sample/22-type common subset; it is not the full target set.

## Allocator measurements

A separate counted allocator processed one batch after five warm-up batches, with identical source, options, clock and output mode. Inputs and options are excluded. Cumulative requested bytes include full realloc requests and are not RSS.

| C4 batch metric | Before | After |
| --- | ---: | ---: |
| Allocations | 5,111 | 4,972 |
| Reallocations | 1,937 | 1,844 |
| Cumulative requested bytes | 1,202,099 | 1,187,061 |
| Peak additional live bytes | 112,833 | 112,833 |
| Net retained bytes after batch | 0 | 0 |

Both C4 batches produced the allocator totals shown above; their exact sources and order are retained in separate input manifests. The other 32 Sample types and 38 extended groups had identical allocation counts, reallocation counts, requested bytes and peak additional live bytes. All measured batches ended with zero net retained bytes. This is a batch measurement, not general leak detection or a claim that RSS fell. [All 33 Sample groups](research/literal-label-optimization-2026-10-20/allocations-samples.json) and [all 39 extended groups](research/literal-label-optimization-2026-10-20/allocations-extended.json) are included.

## Compatibility and original-test acceptance

- Workspace: **3,202 passed, 0 failed, 5 ignored**. No-default-features regressions: 4 passed. Clippy completed with existing repository warnings.
- **761/761 complete reference responses, 167/167 Sample SVGs and 39/39 extended verification records matched**.
- New independent pinned Node reference: **1,008 literal layouts, 1,184 character-wrapping cases, and 444 left-aligned SVG cases**. All 2,192 layout cases compare complete measurement callback traces; label dimensions and span widths are checked by exact binary64 bits. It includes ASCII/Unicode, combining marks, supplementary characters, controls, blank/empty spans, icons, style mixtures, finite/infinite room and custom nonadditive measurement. Seed: `0x17e2b6`.
- These three supplementary tests do not increase the original-test mapping count. Existing original tests, fixtures, assertions, thresholds, loops, seeds and mappings are unchanged.
- Original independent-timeout acceptance: **2,785 passed, 0 failed, 7 preserved upstream skips, 4,034 unmapped, 0 unverified**. `--require-complete` exited 1 with `complete: false`. Full original-test acceptance remains incomplete.

[Summary and fingerprints](research/literal-label-optimization-2026-10-20/summary.json) · [Validation record](research/literal-label-optimization-2026-10-20/validation.json) · [Extended inputs](research/literal-label-optimization-2026-10-20/cases.json) · [Sample inputs](research/output-benchmark-2026-10-20-literal-label/cases.json). The evidence directories retain all process timing samples and verification records.

```sh
node tools/compat/gen-literal-label-regression.mjs
cargo test -p zebra-mermaid --all-features --locked --test literal_label_reference
cargo test --workspace --all-features --locked -- --test-threads=1
node tools/test-parity/check.mjs --require-complete
```

For paired timing, build and freeze both versions, then use `ZM_BENCH_BEFORE`, `ZM_BENCH_AFTER` and `ZM_BENCH_OUTPUT` with `tools/bench/compare-optimization.py`. Use `ZM_OUTPUT_BEFORE`, `ZM_OUTPUT_NATIVE` and `ZM_OUTPUT_REPORT` with `tools/bench/compare-output.py` for the full Sample output contracts. Keep instrumentation and tests separate from timing.
