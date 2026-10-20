# Borrowed SVG adapters and direct appends

On all **167 unchanged Samples across all 33 diagram types**, this round reduced the equal-weight geometric mean native rendering time by **3.25%** against the frozen Rust baseline. Observed Sample medians fell in 26 types and rose in seven. Current Rust/Pele speedup is **1.79× at API return** and **1.89× for UTF-8-ready output**. The 10× target remains unmet.

The baseline is `2b1fda61446bdc0f95ab6b8534cefb6a409f6056`. Candidate revision: `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. Baseline binaries were built from identical production code at `35db1ee`; source, input and binary fingerprints were checked after fresh CPU sampling and before reuse. Pele remains fixed to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Runtime versions: Node v26.3.0, rustc 1.96.1.

## Profiling and implementation

Fresh external CPU captures covered Info, C4, Block, Sequence and Swimlane. Sequence had **969 inclusive `core::fmt::write` occurrences** and **634 `alloc::fmt::format_inner` occurrences** in about 3,559 thread samples. Shared label writing also appeared throughout the captures. These overlapping inclusive counts include descendants and recursion; they are not additive exclusive CPU percentages. [Before/after counts and capture hashes](research/svg-adapter-optimization-2026-10-20/format-frames.json) are retained.

Internal label adapters now borrow their render label or an existing view, constructing the view at the write boundary. Private centered, edge and left-aligned writers pass that view by reference within the shared output implementation. Public label/edge helper signatures, owned models and public adapter fields remain unchanged. Text measurement, layout, floating-point operations and output bytes retain their existing implementations. No parsed model, label or rendered result is cached.

The measured build's diagnostics show **72 bytes for `LabelView` and eight for the label reference**. Current internal centered/view adapters are 48 bytes, the edge adapter 80, and the left-aligned adapter 40. The unchanged public edge adapter is 144 bytes. These are target-specific representation diagnostics, not public layout assertions or isolated speed measurements. [All recorded sizes](research/svg-adapter-optimization-2026-10-20/adapter-sizes.json) are retained.

Sequence now uses the existing static SVG concatenation layer for **16 string constructions and 25 appends**. Another **47 single-adapter appends across 21 render files** use direct `Part` writing for known labels and markers. Formatting flags and non-SVG constructions retain standard formatting. Arguments are still evaluated before writing, and temporary lifetimes and drop order retain the existing concatenation contract. The parser and all original work limits are unchanged.

After the change, Sequence’s inclusive `core::fmt::write` occurrences fell **969 → 346**, while Info’s shared-label occurrences rose **685 → 802** and Block’s **97 → 122**. These captures do not establish a universal reduction in label CPU cost.

The combined implementation is measured as a whole. Its results do not isolate the contribution of borrowed views from direct appends, and the regressions below prevent a claim that every workload benefits. [Sequence conversion inventory](research/svg-adapter-optimization-2026-10-20/conversion.json) and [direct-adapter inventory](research/svg-adapter-optimization-2026-10-20/wrapper-conversion.json) identify the affected calls.

## Timing and regressions

Three independent processes per version/engine, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, CPU sampling, allocation probes and process-resource checks ran separately from latency timing. Original inputs, Arial metrics, CSS output, fixed UTC clock and complete parse/layout/SVG work were retained.

| Set / operation | After / before time | Observed change |
| --- | ---: | ---: |
| All 167 Samples / 33 types, render | 0.96753 | −3.25% |
| All 33 documentation types, render | 0.98471 | −1.53% |
| All 33 documentation types, parse | 0.99252 | −0.75% |
| Sequence documentation, render | 0.90008 | −9.99% |

Parsing algorithms were not changed, so its observed difference is not attributed to a parsing optimization. Small differences are not statistical-significance claims.

All seven Sample regressions are retained: **Pie +7.92%, State +4.97%, Packet +2.72%, Railroad +2.31%, Flowchart +2.31%, Swimlane +1.70%, Radar +0.39%**. Lower medians include **Class −15.55%, Block −15.36%, C4 −14.10%, Git −14.01%, Ishikawa −9.66%, Sequence −7.23%**. Every type and individual process median is in the [complete Sample report](research/output-benchmark-2026-10-20-svg-adapters/comparison.json).

Extended results include substantial regressions: the **114-example Flowchart batch +30.73%**, **50-node tree +10.04%**, **small labelled mesh +7.41%**, **500-node tree +7.03%**, and **large labelled mesh +18.65%**. Documentation regressions include **Venn +20.02%, Timeline +13.35%, Treemap +9.57%, Flowchart +8.09%, Wardley +7.93%, Swimlane +4.25%**; all smaller regressions are also retained. Some unchanged parsing paths varied materially, and process runs contain outliers. No run was discarded or repeated to replace these results. The evidence does not resolve every regression's cause. [All 39 paired rows and raw runs](research/svg-adapter-optimization-2026-10-20/extended-comparison.json) remain available for follow-up.

The current [full output-contract report](OUTPUT-BENCHMARK.md) separately measures API return and UTF-8-ready output. API speedups range from **Agentflow 1.21×** to **Sankey 2.86×**: 23 types are below 2×, ten are at least 2×, and none reaches 10×. Node is remeasured each round; cross-round absolute ratios do not establish the native gain. The four-engine common set remains separately reported as 110 Samples / 22 types and does not replace the main target set.

## Allocations and driver sizes

The counted allocator uses identical inputs/options, five warm-up batches and one measured batch. Inputs/options are excluded and results are dropped each iteration. Requested bytes include full realloc requests and are separate from OS peak RSS.

All 33 Sample allocation counts are unchanged. **Sequence alone** has fewer reallocations and lower requested/live-peak bytes; the other 32 types are unchanged:

| Sequence set | Metric | Before | After |
| --- | --- | ---: | ---: |
| Samples | Allocations | 988 | 988 |
| Samples | Reallocations | 501 | 433 |
| Samples | Requested bytes | 390,194 | 363,097 |
| Documentation | Allocations | 4,493 | 4,493 |
| Documentation | Reallocations | 2,652 | 2,291 |
| Documentation | Requested bytes | 1,661,097 | 1,508,089 |
| Both | Peak additional live bytes | 36,631 | 36,487 |

Across all 39 extended groups, the other 38 groups' allocator metrics are unchanged. Every Sample and extended batch ended with zero net retained bytes. This is not a general leak audit. [Sample allocations](research/svg-adapter-optimization-2026-10-20/allocations-samples.json) and [extended allocations](research/svg-adapter-optimization-2026-10-20/allocations-extended.json) retain every row.

The main cross-engine driver changed **9,362,432 → 9,363,088 bytes**; the extended driver **5,068,128 → 5,068,880 bytes**; the CPU driver **5,014,832 → 5,015,584 bytes**; the counted-allocator driver **5,532,336 → 5,516,912 bytes**. Release, thin LTO and single-codegen-unit settings were unchanged. [Source, input and binary fingerprints](research/svg-adapter-optimization-2026-10-20/summary.json) identify the measured programs.

<!-- process-resources:start -->
## OS process resources

Separately from all latency runs, tests, CPU sampling and allocation probes, **three fresh processes per version and group** ran with alternating order. Each performed five warm-up batches and a complete-render loop of at least one second. OS peak RSS and process CPU totals include startup, input-manifest loading and warm-up. These are process-wide diagnostics, not the render-only allocator peak or the API latency benchmark. Every batch's success/rejection counts and SVG byte total matched.

| Set | Peak RSS lower | Unchanged | Higher |
| --- | ---: | ---: | ---: |
| All 33 Sample types | 30 | 1 | 2 |
| All 39 extended groups | 34 | 1 | 4 |

For Samples, the range of per-type median peak RSS was **3,342,336–4,947,968 bytes before** and **3,276,800–4,947,968 bytes after**. The largest observed Sample increases were Agentflow 4,767,744 → 4,833,280 bytes; Usecase 4,915,200 → 4,947,968 bytes. These process measurements are separate from render-only allocator peaks. Small changes may include page and allocator variability; every regression is retained. Per-case RSS medians are not summed.

The extended range was **3,883,008–11,386,880 bytes before** and **3,768,320–11,960,320 bytes after**. All four extended RSS increases are retained: the large labelled mesh **11,386,880 → 11,960,320 bytes (+560 KiB)**, Agentflow documentation **5,242,880 → 5,308,416**, Ishikawa **4,030,464 → 4,063,232**, and Packet **4,276,224 → 4,308,992 bytes**.

[Every Sample process](research/svg-adapter-optimization-2026-10-20/resources-samples.json) and [every extended process](research/svg-adapter-optimization-2026-10-20/resources-extended.json) retain peak RSS, CPU seconds, actual loop counts, inputs and binary fingerprints.
<!-- process-resources:end -->

## Compatibility and acceptance

- Full workspace: **3,226 passed, 0 failed, 5 ignored**. No-default-features regressions: four passed. Clippy completed with unchanged warning categories and counts.
- **761/761 complete reference responses, 167/167 Sample SVGs, 39/39 extended verification records matched**.
- Two supplementary direct-part tests use unchanged independent Node fixtures: **6,048 centered-label cases**, both with and without icons, through render-label and borrowed-view parts; **444 left-aligned cases** through both public and internal label storage. Existing temporary/evaluation/drop-order tests also remain enabled. These additions do not increase original-test acceptance.
- Original test bodies, fixture expectations, assertions, random seeds, iteration counts, performance thresholds and mappings are unchanged. Measured production-source, input and binary fingerprints are checked again before publication.
- Original independent-timeout gate: **2,785 passed, 0 failed, seven preserved upstream skips, 4,034 unmapped, zero unverified**. Strict completeness exited 1 with `complete: false`. Full original-test acceptance remains unfinished.

[Validation](research/svg-adapter-optimization-2026-10-20/validation.json) · [Extended inputs](research/svg-adapter-optimization-2026-10-20/cases.json) · [Sample inputs and SVG verification](research/output-benchmark-2026-10-20-svg-adapters/comparison.json). Every independent-process timing is retained in the associated evidence directories.

```sh
cargo test --workspace --all-features --locked -- --test-threads=1
cargo clippy --workspace --all-features --locked
node tools/test-parity/check.mjs --require-complete
```

For paired latency, freeze both versions and use `tools/bench/compare-optimization.py` with `ZM_BENCH_BEFORE`, `ZM_BENCH_AFTER` and `ZM_BENCH_OUTPUT`. Use `tools/bench/compare-output.py` with `ZM_OUTPUT_BEFORE`, `ZM_OUTPUT_NATIVE` and `ZM_OUTPUT_REPORT` for the full Sample contracts. For separate OS resource checks, use `tools/bench/compare-resources.py` with `ZM_RESOURCE_BEFORE`, `ZM_RESOURCE_AFTER`, `ZM_RESOURCE_CASES` and `ZM_RESOURCE_REPORT`. Keep builds, tests and all probes outside latency timing.
