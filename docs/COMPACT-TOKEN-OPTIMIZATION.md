# Prepared grammars and compact parser tokens

On all **167 unchanged Samples across all 33 diagram types**, this round reduced the equal-weight geometric mean native rendering time by **1.23%** against the frozen Rust baseline. Observed Sample medians fell in 21 types and rose in 12. The current Rust/Pele speedup is **1.73× at API return** and **1.83× for UTF-8-ready output**. The 10× target remains unmet.

The baseline is `36631d1d70c7ebd84267579c46b907f15f06450b`; the measured implementation is `35db1ee76e1921ab164f432fdf670c2927b2ef06`. Baseline binaries were built from identical production code at `f1c5a83`; source, input and binary fingerprints were checked after fresh baseline CPU sampling and before reuse. Pele is fixed to 1.7.0, `8603bf3313add9143cd417015d4a6bd6c522dada`. Runtime versions: Node v26.3.0, rustc 1.96.1.

## Evidence and implementation

Fresh external sampling covered Info, Architecture, Packet, Treemap and Railroad. The public tokenizer appeared in **780, 600, 497, 528 and 226 inclusive frame occurrences**, respectively, in captures with about 3,500 thread samples each. Inspecting this path showed three independently grown token vectors and repeated lookup of immutable grammar metadata. [Before/after captures](research/compact-token-optimization-2026-10-20/summary.json) and [complete token-related frame counts](research/compact-token-optimization-2026-10-20/token-frames.json) are retained. These counts include overlapping descendants and recursion and are not additive exclusive CPU percentages. After inlining, the candidate's tokenizer appears within `Grammar::reader`; changed function boundaries must not be interpreted as eliminated work.

The internal parser path now stores `{ start: u32, end: u32, kind: u32 }` in a single vector, with a compile-time assertion that each token is 12 bytes. The former three `usize` vectors have 24 bytes of token payload on 64-bit builds. Native-width storage remains available when the source or table exceeds the compact boundary; it preserves the original EOF sentinel and full native kind range. No new source limit or truncating fallback is introduced.

Each fixed diagram grammar prepares its ASCII candidate lists once in a `LazyLock`. Those lists contain matching-rule metadata only. Every source is scanned again, with the same ordered matchers, longer alternatives, hidden tokens, scanner state and lexical failures. Sources, tokens, parsed models and SVG results are not cached. This removes the internal path's repeated global map lookup, fingerprint calculation and `Arc` operations while retaining the existing ASCII candidate indexing.

The existing public `TokenType`, `Pattern`, scanner factories, `Tokens` vectors, `Reader` and tokenizer signatures remain unchanged. Public dynamic tables retain their existing lookup behavior. The public and private token representations share the same matching loop. The private reader preserves token access and error fields, and is used by **12 diagram types / 15 grammar tables**, including Railroad's four notations. The other 21 diagram types remain in every full-target measurement.

The compact-span idea was examined during [Ondine source research](https://github.com/wevm/ondine/blob/6a1586e5af980e67c104239593ae931837157b8b/crates/compiler/src/diagrams/common/tokens.rs). This round retains this project's fixed Pele matching loop and public API. Scanner factories still use their existing boxed closures; dedicated scanner state remains future work. No runtime dependency was added.

## Independent token and failure checks

The new [reference generator](../tools/compat/gen-token-storage-reference.mjs) loads the actual token tables from the clean pinned Node source, including `lexRadar`'s original per-source state reset. It reads the existing documentation fixture without modifying it and adds Unicode, whitespace, malformed delimiters, long digit runs, repeated unclosed descriptions and **96 seeded mutations per table**. The seed is `0x70c3a9`.

The [independent fixture](../tests/fixtures/unit/token-storage-reference.json.gz) contains **2,008 cases across all 15 tables**: 934 successful streams, 1,074 expected lexical rejections, and 2,153 Reader error records. Each case is checked against the public owned vectors, internal compact tokens and a forced native-width fallback. Assertions retain token kind, EOF, text, UTF-16 start/end positions and complete lexical/Reader error fields. No expectation was generated from Rust.

Fifteen supplementary table tests and two storage-boundary tests were added. They do not increase original-test acceptance. Existing original test bodies, fixture expectations, seeds, iterations, timeouts, thresholds and mappings were preserved.

## Timing and regressions

Three independent processes per version/engine, alternating order; five warm-up batches, at least 10 ms calibration, at least nine samples and at least 300 ms per operation. Builds, tests, CPU sampling, allocation instrumentation and process-resource checks ran separately from latency timing. Original inputs, Arial metrics, CSS output, fixed UTC clock and complete parse/layout/SVG work were retained.

| Set / operation | After / before time | Observed change |
| --- | ---: | ---: |
| All 167 Samples / 33 types, render | 0.98768 | −1.23% |
| All 33 documentation types, render | 0.98108 | −1.89% |
| All 33 documentation types, parse | 0.96773 | −3.23% |
| Affected 12 documentation types, parse | 0.90408 | −9.59% |

The last row is a supplementary component view; it does not replace the full 33-type target. All 12 affected documentation parse medians were lower: Info **−28.77%**, Wardley **−13.31%**, Cynefin **−11.56%**, Pie **−9.46%**, Eventmodeling **−9.27%**, Treemap **−8.17%**, Railroad **−7.57%**, Packet **−7.42%**, Git **−5.00%**, Architecture **−4.18%**, Treeview **−4.01%**, Radar **−3.19%**.

All 12 Sample render regressions are retained: **Treemap +2.21%, Ishikawa +2.09%, XYChart +1.32%, Venn +0.61%, State +0.56%, Journey +0.51%, Usecase +0.41%, C4 +0.41%, Timeline +0.26%, Sequence +0.20%, ER +0.08%, Gantt +0.01%**. Info Samples improved **10.53%**, Pie **5.85%**, Packet **4.33%** and Wardley **4.12%**. Small differences are not statistical-significance claims, and changes in the other diagram implementations are not attributed to their parsing algorithms.

Documentation render medians were lower in 25/33 types. Regressions include **Block +4.71%, Requirement +4.08%, XYChart +1.02%, Swimlane +0.98%**. The 500-node tree rose **2.11%**, the 50-node tree **1.67%**, the Agentflow reference batch **3.59%**, and the large labelled mesh **0.17%**. Every row, including smaller regressions, is retained in the [39-group paired report](research/compact-token-optimization-2026-10-20/extended-comparison.json).

The current [full Sample contract report](OUTPUT-BENCHMARK.md) separately measures API return and UTF-8-ready output. API speedups range from **Agentflow 1.21×** to **Sankey 2.86×**: 25 types are below 2×, eight are at least 2×, and none reaches 10×. Node is remeasured each round; dividing absolute Rust/Pele ratios from different rounds does not establish this round's native improvement. The native gain above comes from the paired frozen before/after comparison. The four-engine common set remains separately reported as 110 Samples / 22 types.

## Allocations and driver sizes

The counted allocator uses identical inputs/options, five warm-up batches and one measured batch. Inputs/options are excluded and results are dropped each iteration. Cumulative requested bytes include full realloc requests. These metrics are separate from OS peak RSS.

| Sample metric | Lower | Unchanged | Higher |
| --- | ---: | ---: | ---: |
| Allocation count | 12 | 21 | 0 |
| Reallocation count | 12 | 21 | 0 |
| Cumulative requested bytes | 12 | 21 | 0 |
| Peak additional live bytes | 0 | 33 | 0 |

Examples from the complete Sample batches:

- Info allocations **36 → 32**, reallocations **9 → 7**, requested bytes **4,571 → 4,379**; peak additional live bytes remains **1,182**.
- Packet reallocations **122 → 92**, requested bytes **99,693 → 93,741**; peak remains **24,559**.
- Wardley reallocations **647 → 591**, requested bytes **295,698 → 278,322**; peak remains **36,770**.
- Radar requested bytes **88,731 → 81,195**; peak remains **13,103**.

Across 39 extended groups, allocations, reallocations and requested bytes each fell in 12 and were unchanged in 27; peaks were unchanged in all 39. Every Sample and extended batch ended with zero net retained bytes. This is not a general leak audit. [All Sample allocation rows](research/compact-token-optimization-2026-10-20/allocations-samples.json) and [all extended allocation rows](research/compact-token-optimization-2026-10-20/allocations-extended.json) are preserved.

The main cross-engine driver grew **9,360,112 → 9,362,432 bytes (+0.025%)**; the extended driver grew **5,066,064 → 5,068,128 bytes (+0.041%)**. The CPU driver grew **5,012,768 → 5,014,832 bytes**, and the counted-allocator driver **5,513,952 → 5,532,336 bytes**. Release, thin LTO and single-codegen-unit settings were unchanged. [Source, input and binary fingerprints](research/compact-token-optimization-2026-10-20/summary.json) identify the measured programs.

<!-- process-resources:start -->
## OS process resources

Separately from all latency runs, tests, CPU sampling and allocation probes, **three fresh processes per version and group** ran with alternating order. Each performed five warm-up batches and a complete-render loop of at least one second. OS peak RSS and process CPU totals include startup, input-manifest loading and warm-up. These are process-wide diagnostics, not the render-only allocator peak or the API latency benchmark. Every batch's success/rejection counts and SVG byte total matched.

| Set | Peak RSS lower | Unchanged | Higher |
| --- | ---: | ---: | ---: |
| All 33 Sample types | 8 | 0 | 25 |
| All 39 extended groups | 11 | 4 | 24 |

For Samples, the range of per-type median peak RSS was **3,293,184–5,013,504 bytes before** and **3,342,336–5,013,504 bytes after**. The largest observed Sample increases were Cynefin 4,030,464 → 4,243,456 bytes; Architecture 4,358,144 → 4,538,368 bytes; Sequence 4,390,912 → 4,538,368 bytes. These results do not support an overall RSS reduction claim. Small changes may include page and allocator variability; they are retained rather than explained away. Per-case RSS medians are not summed.

[Every Sample process](research/compact-token-optimization-2026-10-20/resources-samples.json) and [every extended process](research/compact-token-optimization-2026-10-20/resources-extended.json) retain peak RSS, CPU seconds, actual loop counts, inputs and binary fingerprints.
<!-- process-resources:end -->

## Compatibility and acceptance

- Full workspace: **3,224 passed, 0 failed, 5 ignored**. No-default-features regressions: four passed. Clippy completed with unchanged warning categories and counts.
- **761/761 complete reference responses, 167/167 Sample SVGs, 39/39 extended verification records matched**.
- Measured production-source and binary fingerprints matched after validation. The two frozen input manifests also match the prior verified complete sets.
- Original independent-timeout gate: **2,785 passed, 0 failed, seven preserved upstream skips, 4,034 unmapped, zero unverified**. Strict completeness exited 1 with `complete: false`; full original-test acceptance remains unfinished.

[Validation](research/compact-token-optimization-2026-10-20/validation.json) · [Extended inputs](research/compact-token-optimization-2026-10-20/cases.json) · [Sample inputs and output verification](research/output-benchmark-2026-10-20-compact-token/comparison.json). All independent-process timings are retained in the associated evidence directories.

```sh
cargo test --workspace --all-features --locked -- --test-threads=1
cargo clippy --workspace --all-features --locked
node tools/test-parity/check.mjs --require-complete
```

For paired latency, freeze both versions and use `tools/bench/compare-optimization.py` with `ZM_BENCH_BEFORE`, `ZM_BENCH_AFTER` and `ZM_BENCH_OUTPUT`. Use `tools/bench/compare-output.py` with `ZM_OUTPUT_BEFORE`, `ZM_OUTPUT_NATIVE` and `ZM_OUTPUT_REPORT` for the full Sample contracts. For separate OS resource checks, use `tools/bench/compare-resources.py` with `ZM_RESOURCE_BEFORE`, `ZM_RESOURCE_AFTER`, `ZM_RESOURCE_CASES` and `ZM_RESOURCE_REPORT`. Keep builds, tests and all probes outside latency timing.
