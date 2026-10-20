# Compiled code size

Compiled code size is an additional optimization objective. The performance target remains **2× at API return** on all 167 original Samples and 33 types. Size has no user-assigned numeric threshold. Preserve full features, correctness and complete workloads, and report speed, allocator and process-RSS tradeoffs independently.

Current verified source: `c1ca29b903feba11145f179a38ed587fa1b82554`. All 33 types remain enabled. Release/thin LTO/one codegen unit, locked dependencies, rustc 1.96.1; no stripping. Section reader: Apple LLVM 21.0.0 `llvm-size`. Source checkout paths are remapped consistently. This is a current-build measurement, not a cross-platform footprint claim.

## Current release artifacts

| Artifact | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| CLI | 5,142,400 | 3,797,128 |
| FFI static archive | 41,537,032 | 4,226,252 |
| FFI dynamic library | 4,749,136 | 3,513,640 |

Against the matched baseline, CLI file bytes fall **18,432**, instruction text **3,480**. FFI dynamic/static files fall **16,688 / 8,840 bytes**, text **4,184 / 4,184 bytes**. Sample native rendering time rises **0.58%**, while documentation render falls **0.72%**. This is a size/allocation tradeoff, not a demonstrated overall speedup. Current API **1.79×**, UTF-8 **1.90×**; the 2× API-return target remains unmet.

File bytes include data, symbols, metadata, embedded bitcode and archive overhead. Instruction text sums `__text` (or ELF `.text` / `.text.*`) separately from stubs, unwind tables and other sections. An unreadable archive member fails measurement instead of becoming zero-sized code. Static archive text includes every member before the host linker selects/removes unused code; neither its file nor text total is the host application's final link increment. Rust `.rlib` metadata is not substituted for a linked library.

[Current paired sections, hashes and build configuration](research/borrowed-entry-optimization-2026-10-20/size-comparison.json) · [implementation, timing and memory tradeoffs](BORROWED-ENTRY-OPTIMIZATION.md). The original [first release baseline](research/code-size-baseline-2026-10-20/summary.json) remains immutable.

## Development drivers

Drivers contain different harness/dependency code and are kept separate from shipping CLI/FFI artifacts. Their builds do not use the shipping-size source-path remapping.

| Driver | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| bench | 5,000,720 | 3,701,940 |
| profile | 4,947,456 | 3,662,632 |
| allocator | 5,448,720 | 4,149,352 |
| cross-bench | 9,295,008 | 6,738,844 |

[Both phases, every section and fingerprint](research/borrowed-entry-optimization-2026-10-20/driver-sizes.json). Driver changes are not reported as CLI or FFI changes.

## Reproduction

Keep builds and section measurements outside latency timing, tests, CPU sampling, allocator probes and process-resource checks. Create isolated source snapshots for both revisions; preserve all build parameters, features and locked dependency versions. The matched baseline can be reproduced from the repository root:

```sh
mkdir -p target/code-size/baseline-source
git archive 9497f7eda18b7029332a9f57d73546c5cb6520dd \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision 9497f7eda18b7029332a9f57d73546c5cb6520dd
python3 tools/bench/build-size.py \
  --source . --output target/code-size/after \
  --revision c1ca29b903feba11145f179a38ed587fa1b82554
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

Use the actual source revision label, or `working-tree` when dirty, and a fresh output directory. The builder fingerprints sources before/after compilation, normalizes source paths, rejects local profile/rustflag environment overrides and refuses to replace a frozen manifest. The section comparison rejects different toolchains, declared profiles, commands, normalized environments or artifact sets, and never changes/strips a binary. It describes only its own measurement, not whole-host idleness.

Both source snapshots identify the matched release builds. The preceding round’s test-only revalidation remains preserved in its own evidence; no source exceptions are needed for this round.

Before latency measurements, `python3 tools/bench/check-timing-window.py` can detect listed build/test drivers without stopping processes or storing paths/host details. Its snapshot is not proof of sustained idleness. Preserve every process result, including regressions and contention observations. Record stripping/debug/symbol changes separately from instruction reductions. Do not obtain smaller artifacts by disabling types/features, lowering precision, simplifying output, caching results, reducing inputs or weakening acceptance thresholds.
