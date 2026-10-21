# Compiled code size

Compiled code size is an additional optimization objective. The performance target remains **2× at API return** on all 167 original Samples and 33 types. Size has no user-assigned numeric threshold. Preserve full features, correctness and complete workloads, and report speed, allocator and process-RSS tradeoffs independently.

Current verified source: `528010f970b8b9b54d94d88275c43e96f4b32015`. All 33 types remain enabled. Release/thin LTO/one codegen unit, locked dependencies, rustc 1.96.1; no stripping. Section reader: Apple LLVM 21.0.0 `llvm-size`. Source checkout paths are remapped consistently. This is a current-build measurement, not a cross-platform footprint claim.

## Current release artifacts

| Artifact | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| CLI | 5,160,080 | 3,797,408 |
| FFI static archive | 41,590,632 | 4,228,952 |
| FFI dynamic library | 4,766,240 | 3,516,308 |

Against the matched baseline: CLI file **+832**, instruction text **-1,512 bytes**; FFI static archive file **+45,424**, instruction text **+1,336 bytes**; FFI dynamic library file **+17,056**, instruction text **+1,192 bytes**.

Full-type native render time changes **-1.09%**; documentation render **-0.66%**. All increases are retained as speed/size tradeoffs. Current API **1.85×**, UTF-8 **1.97×**; the 2× API-return target remains unmet.

A subsequent [cold-error hint experiment](COLD-ERROR-EXPERIMENT.md) was rejected: native time changed −0.43% while CLI / FFI static / FFI dynamic instruction text grew by 6,612 / 4,920 / 4,356 bytes. All allocator metrics stayed equal. The candidate and every regression are preserved separately; the current release sizes above remain the verified production artifacts.

File bytes include data, symbols, metadata, embedded bitcode and archive overhead. Instruction text sums `__text` (or ELF `.text` / `.text.*`) separately from stubs, unwind tables and other sections. An unreadable archive member fails measurement instead of becoming zero-sized code. Static archive text includes every member before the host linker selects/removes unused code; neither its file nor text total is the host application's final link increment. Rust `.rlib` metadata is not substituted for a linked library.

[Current paired sections, hashes and build configuration](research/number-text-optimization-2026-10-20/size-comparison.json) · [implementation, timing and memory tradeoffs](NUMBER-TEXT-OPTIMIZATION.md). The original [first release baseline](research/code-size-baseline-2026-10-20/summary.json) remains immutable.

## Development drivers

Drivers contain different harness/dependency code and are kept separate from shipping CLI/FFI artifacts. Their builds do not use the shipping-size source-path remapping.

| Driver | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| bench | 5,001,776 | 3,702,220 |
| profile | 4,948,464 | 3,662,912 |
| allocator | 5,449,856 | 4,147,776 |
| cross-bench | 9,296,464 | 6,739,376 |

[Both phases, every section and fingerprint](research/number-text-optimization-2026-10-20/driver-sizes.json). Driver changes are not reported as CLI or FFI changes.

## Reproduction

Keep builds and section measurements outside latency timing, tests, CPU sampling, allocator probes and process-resource checks. Create isolated source snapshots for both revisions; preserve all build parameters, features and locked dependency versions. The matched baseline can be reproduced from the repository root:

```sh
mkdir -p target/code-size/baseline-source
git archive 28dbcd3a1a4fc6db1c94fb4d9b13969a97f43a5e \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision 28dbcd3a1a4fc6db1c94fb4d9b13969a97f43a5e
python3 tools/bench/build-size.py \
  --source . --output target/code-size/after \
  --revision 528010f970b8b9b54d94d88275c43e96f4b32015
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

Use the actual source revision label, or `working-tree` when dirty, and a fresh output directory. The builder fingerprints sources before/after compilation, normalizes source paths, rejects local profile/rustflag environment overrides and refuses to replace a frozen manifest. The section comparison rejects different toolchains, declared profiles, commands, normalized environments or artifact sets, and never changes/strips a binary. It describes only its own measurement, not whole-host idleness.

Both source snapshots identify the matched release builds. The preceding round’s test-only revalidation remains preserved in its own evidence; no source exceptions are needed for this round.

Before latency measurements, `python3 tools/bench/check-timing-window.py` can detect listed build/test drivers without stopping processes or storing paths/host details. Its snapshot is not proof of sustained idleness. Preserve every process result, including regressions and contention observations. Record stripping/debug/symbol changes separately from instruction reductions. Do not obtain smaller artifacts by disabling types/features, lowering precision, simplifying output, caching results, reducing inputs or weakening acceptance thresholds.
