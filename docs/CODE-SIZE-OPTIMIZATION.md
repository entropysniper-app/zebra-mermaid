# Compiled code size

Compiled code size is an additional optimization objective. The performance target remains **2× at API return** on all 167 original Samples and 33 types. Size has no user-assigned numeric threshold. Preserve full features, correctness and complete workloads, and report speed, allocator and process-RSS tradeoffs independently.

Current verified source: `9de7ca548364b364d4d0f5149af48c1eacd242bc`. All 33 types remain enabled. Release/thin LTO/one codegen unit, locked dependencies, rustc 1.96.1; no stripping. Section reader: Apple LLVM 21.0.0 `llvm-size`. Source checkout paths are remapped consistently. This is a current-build measurement, not a cross-platform footprint claim.

## Current release artifacts

| Artifact | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| CLI | 5,160,832 | 3,800,608 |
| FFI static archive | 41,545,872 | 4,230,436 |
| FFI dynamic library | 4,765,824 | 3,517,824 |

Against the matched frozen baseline, CLI file bytes fell **33,568** and instruction text **44,556**. FFI dynamic/static file bytes fell **32,960 / 165,120**, and instruction text **43,304 / 43,304 bytes**. All three shipping files and instruction-text totals decrease in this round. The current full-type API speedup is **1.78×**, UTF-8 **1.90×**; the 2× API-return target is unmet.

File bytes include data, symbols, metadata, embedded bitcode and archive overhead. Instruction text sums `__text` (or ELF `.text` / `.text.*`) separately from stubs, unwind tables and other sections. An unreadable archive member fails measurement instead of becoming zero-sized code. Static archive text includes every member before the host linker selects/removes unused code; neither its file nor text total is the host application's final link increment. Rust `.rlib` metadata is not substituted for a linked library.

[Current paired sections, hashes and build configuration](research/parser-ownership-optimization-2026-10-20/size-comparison.json) · [implementation, timing and memory tradeoffs](PARSER-OWNERSHIP-OPTIMIZATION.md). The original [first release baseline](research/code-size-baseline-2026-10-20/summary.json) remains immutable.

## Development drivers

Drivers contain different harness/dependency code and are kept separate from shipping CLI/FFI artifacts. Their builds do not use the shipping-size source-path remapping.

| Driver | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| bench | 5,002,512 | 3,705,420 |
| profile | 4,965,712 | 3,666,112 |
| allocator | 5,467,344 | 4,152,028 |
| cross-bench | 9,313,184 | 6,742,324 |

[Both phases, every section and fingerprint](research/parser-ownership-optimization-2026-10-20/driver-sizes.json). Driver changes are not reported as CLI or FFI changes.

## Reproduction

Keep builds and section measurements outside latency timing, tests, CPU sampling, allocator probes and process-resource checks. Create isolated source snapshots for both revisions; preserve all build parameters, features and locked dependency versions. The initial baseline can be reproduced from the repository root:

```sh
mkdir -p target/code-size/baseline-source
git archive 7235d3f5a7ba881a43796c82778617df54b618f7 \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision 7235d3f5a7ba881a43796c82778617df54b618f7
python3 tools/bench/build-size.py \
  --source . --output target/code-size/after \
  --revision 9de7ca548364b364d4d0f5149af48c1eacd242bc
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

Use the actual source revision label, or `working-tree` when dirty, and a fresh output directory. The builder fingerprints sources before/after compilation, normalizes source paths, rejects local profile/rustflag environment overrides and refuses to replace a frozen manifest. The section comparison rejects different toolchains, declared profiles, commands, normalized environments or artifact sets, and never changes/strips a binary. It describes only its own measurement, not whole-host idleness.

Both source snapshots identify the matched release builds. The preceding round’s test-only revalidation remains preserved in its own evidence; no source exceptions are needed for this round.

Before latency measurements, `python3 tools/bench/check-timing-window.py` can detect listed build/test drivers without stopping processes or storing paths/host details. Its snapshot is not proof of sustained idleness. Preserve every process result, including regressions and contention observations. Record stripping/debug/symbol changes separately from instruction reductions. Do not obtain smaller artifacts by disabling types/features, lowering precision, simplifying output, caching results, reducing inputs or weakening acceptance thresholds.
