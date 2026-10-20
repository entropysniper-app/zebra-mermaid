# Static SVG concatenation

On all **167 unchanged Samples across all 33 diagram types**, this round reduced the equal-weight geometric mean rendering time by **8.61%** against the frozen Rust baseline. All 33 observed Sample medians were lower. The current Rust/Pele speedup is **1.67× at API return** and **1.76× for UTF-8-ready output**. The 10× target remains unmet.

The baseline is `0a3d6763cbd1d22c317d7730042425b0225e05a3`; the measured implementation is `9f1eefab1a30dc06f58df6d01eef33d029ad1e48`. Baseline binaries were built from the identical production code at `062c752`; recorded source and binary fingerprints were checked before reuse. Pele is fixed to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Runtime versions: Node v26.3.0, rustc 1.96.1.

## Evidence and change

Fresh external CPU sampling covered C4, Agentflow, Info, Block and Sequence. Repeated standard formatting frames led to a private SVG concatenation layer for known strings, coordinates and drawing adapters. After the change, `core::fmt::write` frame occurrences fell from **830 to 471** for C4, **885 to 301** for Block, and **354 to 26** for Info. The captures contain roughly 3,500 thread samples each. These are inclusive frame occurrences, with overlaps and recursive occurrences; they must not be added as exclusive CPU percentages. [Complete before/after counts and capture hashes](research/svg-concat-optimization-2026-10-20/format-frames.json) are retained.

The private `svg_format!` and `svg_write!` macros collect borrowed parts and dispatch statically to string, coordinate, label and marker writers. They evaluate all arguments before writing and preserve temporary lifetimes and drop order. Plain SVG parts avoid `fmt::Arguments` dispatch; ordinary numeric Display behavior and explicit formatting specifications retain their existing implementations. Public SVG helper return types and numeric conversion routines are unchanged.

The implementation replaces **340 formatting and 298 append calls across 38 source files**. Twelve other considered calls keep standard formatting. All input groups and test items remain in scope. Sequence and Swimlane still contain substantial standard SVG formatting and are identified as remaining work; the shared SVG layer is used by their output as well.

No parsed model, label or rendered result is cached. Source parsing, layout, measurement, precision, SVG content and the Node/Rust comparison work remain unchanged. Two supplementary Rust contract tests cover Unicode, temporary values, numeric Display, destination reborrows and argument/write/drop ordering. Existing original test bodies and independent Node fixtures were preserved.

## Timing

Three independent processes per engine/version, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, CPU sampling and allocation instrumentation ran outside timing. The same original inputs, font metrics, CSS mode and fixed UTC clock are used on both sides.

| Set / operation | After / before time | Observed change |
| --- | ---: | ---: |
| All 167 Samples / 33 types, render | 0.91387 | −8.61% |
| 33 documentation types, render | 0.91846 | −8.15% |
| 33 documentation types, parse | 0.99490 | −0.51% |
| Timeline documentation, render | 0.81516 | −18.48% |
| Mindmap documentation, render | 0.84738 | −15.26% |
| Journey documentation, render | 0.85256 | −14.74% |

Documentation rendering had lower medians in 32/33 types; Venn was **0.012% slower**. Flowchart and Sequence documentation changes were below 0.4%. Parsing algorithms were not changed, so its small observed difference is retained without attributing it to a parsing optimization. All 39 extended rows, regressions and individual process medians are in the [paired extended comparison](research/svg-concat-optimization-2026-10-20/extended-comparison.json). Small changes are not statistical-significance claims.

The [full Sample report](OUTPUT-BENCHMARK.md) retains every type and both output endpoints. API speedups range from **Info 1.18×** to **Sankey 2.79×**, with no type reaching 10×. UTF-8-ready timing includes Node's materialization, encoding and Buffer allocation. The four-engine common set remains separately reported as 110 Samples / 22 types and does not replace the full target set.

## Allocation and binary-size tradeoffs

The separate counted allocator used identical inputs/options, five warm-up batches and one measured batch. Cumulative requested bytes include full realloc requests. Allocator bytes and driver file sizes are separate from RSS.

| Sample metric | Lower | Unchanged | Higher |
| --- | ---: | ---: | ---: |
| Allocation count | 0 | 33 | 0 |
| Reallocation count | 28 | 5 | 0 |
| Cumulative requested bytes | 30 | 1 | 2 |
| Peak additional live bytes | 10 | 21 | 2 |

Notable Sample results and regressions:

- Cynefin requested bytes **273,882 → 240,547**; Mindmap **249,355 → 224,280**; Eventmodeling **327,060 → 299,926**.
- Timeline reallocations **807 → 527**, but requested bytes **389,754 → 426,621** and peak additional live bytes **29,217 → 34,399**. Capacity estimates trade more reserved memory for fewer growth operations in this batch.
- Kanban requested bytes **133,658 → 135,474**, with unchanged peak **43,078**. Gantt peak increased by **3 bytes**, from **46,184 → 46,187**.

Across all 39 extended groups, allocation counts were unchanged; reallocations fell in 30, requested bytes fell in 36 and rose in two, and peak additional live bytes fell in 13 and rose in two. Every measured Sample and extended batch ended with zero net retained bytes. This is not a general leak audit or a claim that RSS decreased. [Sample allocations](research/svg-concat-optimization-2026-10-20/allocations-samples.json) and [extended allocations](research/svg-concat-optimization-2026-10-20/allocations-extended.json) retain every row.

Static expansion also increases the measured driver sizes: the main cross-engine driver grew **9,128,416 → 9,310,832 bytes (+2.00%)**; the extended driver grew **4,833,424 → 5,017,008 bytes (+3.80%)**. The CPU and allocator drivers grew about 3.8%. These use the same release, thin LTO and single-codegen-unit settings. [All binary sizes and fingerprints](research/svg-concat-optimization-2026-10-20/summary.json) are recorded.

## Compatibility and acceptance

- Full workspace: **3,204 passed, 0 failed, 5 ignored**. No-default-features regressions: 4 passed. Clippy completed with existing warnings and no added warning categories.
- **761/761 complete reference responses, 167/167 Sample SVGs, 39/39 extended verification records matched**.
- Original inputs, fixtures, assertions, seeds, iteration counts, performance thresholds and mappings are unchanged. The two new project tests do not increase original-test acceptance.
- Original independent-timeout gate: **2,785 passed, 0 failed, 7 preserved upstream skips, 4,034 unmapped, 0 unverified**. Strict completeness exited 1 with `complete: false`; full original-test acceptance remains unfinished.

[Validation](research/svg-concat-optimization-2026-10-20/validation.json) · [Sources and binaries](research/svg-concat-optimization-2026-10-20/summary.json) · [Extended inputs](research/svg-concat-optimization-2026-10-20/cases.json) · [Sample inputs](research/output-benchmark-2026-10-20-svg-concat/cases.json). All independent-process timings and output verification records are retained in these evidence directories.

```sh
cargo test --workspace --all-features --locked -- --test-threads=1
cargo clippy --workspace --all-features --locked
node tools/test-parity/check.mjs --require-complete
```

For paired timing, freeze both versions and use `tools/bench/compare-optimization.py` with `ZM_BENCH_BEFORE`, `ZM_BENCH_AFTER` and `ZM_BENCH_OUTPUT`. Use `tools/bench/compare-output.py` with `ZM_OUTPUT_BEFORE`, `ZM_OUTPUT_NATIVE` and `ZM_OUTPUT_REPORT` for the full Sample contracts. Keep tests and profiling outside timed runs.
