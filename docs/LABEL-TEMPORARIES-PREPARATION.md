# Label string construction candidate

This candidate has completed correctness checks, release-size measurement and allocator probes. **Latency timing and OS RSS comparison are still pending.** A read-only preflight found an unrelated build/test driver, so the timing pipeline stopped before producing any latency samples. The current committed performance result remains API **1.79×** and UTF-8 **1.89×**; the **2× API-return target is not yet proven**.

Baseline: `4a03a25c2ef74b393b283751cb04086edd3c2a81`, with the same production code as `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. The uncommitted candidate is captured over `cab1800` in an [exact gzipped patch](research/label-temporaries-prepared-2026-10-20/candidate.patch.gz). Its reverse application was checked against the working candidate without modifying any file. [Source/input/program fingerprints and status](research/label-temporaries-prepared-2026-10-20/summary.json) identify this preparation snapshot; it is not a performance-completion report.

## Implementation

Fresh CPU captures covered Info, all Flowchart documentation, C4, Venn and Treeview. Info still constructed two temporary label strings through `Display::to_string`, with repeated growth visible below label writing. The candidate converts **62 known adapter calls across 31 files** to the existing private SVG `Part` implementation: 57 centered labels, three markers and two coordinate strings. [Every converted call and source hash](research/label-temporaries-prepared-2026-10-20/conversion.json) is recorded.

Centered-label parts provide an initial capacity hint of attribute bytes plus 96 for a nonempty label, and zero for an empty one. This only reserves storage; larger markup grows normally. It does not resolve icons, measure text, cache output or skip any rendering. Info constructs its two labels in one output buffer. Both have no icon resolver, and text measurement still happens beforehand. Public `Display` helpers and signatures are unchanged.

Parsing, layout, numerical formatting, text measurement, output/error limits and original work budgets retain their existing implementations. The two previous scanner/counter runtime experiments have been withdrawn. Four public scanner lifecycle/boundary contract tests are retained independently; they do not count as upstream ports.

## Release artifacts

Same compiler, locked dependencies, full features, release/thin LTO/one codegen unit and remapped source paths; no stripping. All 33 types remain enabled.

| Artifact | File before → candidate (bytes) | Instruction text before → candidate (bytes) |
| --- | ---: | ---: |
| CLI | 5,210,752 → 5,194,400 | 3,853,944 → 3,845,164 |
| FFI dynamic library | 4,798,544 → 4,798,784 | 3,561,420 → 3,561,128 |
| FFI static archive | 41,708,256 → 41,710,992 | 4,273,968 → 4,273,740 |

CLI file bytes fell by **16,352** and instruction text by **8,780**. FFI instruction text fell by **292** and **228 bytes**, respectively, while its file totals increased. These small FFI differences are not a universal size reduction claim. File bytes include symbols, metadata, bitcode and archive overhead. Static-archive text sums all members before host selection/dead stripping; it is not the host application's final link increment. [Every section, configuration and artifact fingerprint](research/label-temporaries-prepared-2026-10-20/size-comparison.json) is retained.

## Allocations and tradeoffs

Allocator probes run separately from latency, after five warm-up batches, for one complete counted batch. Inputs/options are excluded and each render result is dropped. Requested bytes include complete realloc requests; they are not RSS.

| Metric | Samples: lower / equal / higher | Extended groups: lower / equal / higher |
| --- | ---: | ---: |
| Allocations | 1 / 32 / 0 | 1 / 38 / 0 |
| Reallocations | 23 / 10 / 0 | 27 / 12 / 0 |
| Requested bytes | 20 / 10 / 3 | 24 / 12 / 3 |
| Additional-live peak | 3 / 29 / 1 | 2 / 36 / 1 |

Info Samples and documentation changed from **32 → 30 allocations**, **7 → 1 reallocations**, and **4,379 → 3,827 requested bytes**, but additional-live peak rose **1,182 → 1,215 bytes**. Flowchart Samples' reallocations fell **663 → 610** and requested bytes **454,370 → 451,884**, with peak unchanged.

All three requested-byte regressions remain recorded: Journey **44,389 → 44,458**, Radar **81,195 → 82,259**, and Venn Samples **138,057 → 138,411**; Venn documentation **165,743 → 166,097 bytes**. Every batch ended with zero net retained bytes; this is not a general leak audit. [All Sample allocator rows](research/label-temporaries-prepared-2026-10-20/allocations-samples.json), [all extended rows](research/label-temporaries-prepared-2026-10-20/allocations-extended.json).

## Completed correctness checks

- Full candidate workspace: **3,230 passed, zero failed, five ignored**. No-default-feature regressions: four passed. Clippy warning counts/categories are unchanged.
- Existing complete-response fixture tests remain **761/761**. The independent pinned Node label/SVG fixtures and evaluation/drop-order tests also pass.
- Non-timed checks rendered every original Sample: **167/167** SVGs match the pinned reference. Frozen before/after native verification records are byte-identical.
- An additional check rendered **879 input executions across all 39 extended groups** with the primary driver's Arial 16/CSS/fixed-clock options: **820 successes and 59 expected rejections**. Every before/after native verification record matches, and every reference success/rejection outcome and successful SVG matches. These groups contain overlapping inputs, use an additional font configuration, and supplement the original extended benchmark verification; they do not increase original-test acceptance.
- Original isolated-timeout gate: **2,785 passed, zero failed, seven preserved skips, 4,034 unmapped, zero unverified**. Strict completeness exits one with `complete: false`. Full original-test acceptance is unfinished. Assertions, expected values, seeds, loop counts, thresholds and mappings remain unchanged.

[Functional scope and result hashes](research/label-temporaries-prepared-2026-10-20/functional-summary.json), [correctness gate records](research/label-temporaries-prepared-2026-10-20/correctness-gates.json). Native verification records contain SVG/type or error text; reference comparisons check success/rejection outcomes and successful SVG bytes, not identical exception serialization between languages. The full-response checks above are counted separately.

The next measurement uses the already frozen programs and unchanged 167/33 and 39-group inputs. A preflight checks for listed build/test drivers before timing starts, and a postflight records the same limited snapshot after latency runs. These checks do not stop other tasks or prove sustained host idleness. [Observed preflights](research/label-temporaries-prepared-2026-10-20/timing-preflights.json) explain the pending measurement. The runtime candidate will be assessed after the paired timings and separate process-resource checks complete.
