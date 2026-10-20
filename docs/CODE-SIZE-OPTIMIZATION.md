# Compiled code size

Compiled code size is an additional optimization objective. The performance target remains **2× at API return** on all 167 original Samples and 33 types. Size has no user-assigned numeric threshold. Preserve full features, correctness and complete workloads, and report speed, allocator and process-RSS tradeoffs independently.

Current verified source: `2a3b5e01b99b96581194f433d637c6c9a5e3e3c7`. All 33 types remain enabled. Release/thin LTO/one codegen unit, locked dependencies, rustc 1.96.1; no stripping. Section reader: Apple LLVM 21.0.0 `llvm-size`. Source checkout paths are remapped consistently. This is a current-build measurement, not a cross-platform footprint claim.

## Current release artifacts

| Artifact | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| cli | 5,194,400 | 3,845,164 |
| ffi-static | 41,710,992 | 4,273,740 |
| ffi-dynamic | 4,798,784 | 3,561,128 |

Against the matched frozen production baseline, CLI file bytes fell **16,352**, instruction text **8,780**. FFI dynamic/static instruction text fell **292 / 228 bytes**, while their full file sizes rose **240 / 2,736 bytes**. These values are reported separately; the result does not show every artifact's file getting smaller. Current full-type reference speedup is API **1.77×**, UTF-8 **1.87×**, and paired native render time fell **2.04%**. The 2× target is not met.

File bytes include data, symbols, metadata, embedded bitcode and archive overhead. Instruction text sums `__text` (or ELF `.text` / `.text.*`) separately from stubs, unwind tables and other sections. An unreadable archive member fails measurement instead of becoming zero-sized code. Static archive text includes every member before the host linker selects/removes unused code; neither its file nor text total is the host application's final link increment. Rust `.rlib` metadata is not substituted for a linked library.

[Current paired sections, hashes and build configuration](research/label-temporaries-optimization-2026-10-20/size-comparison.json) · [implementation, timing and memory tradeoffs](LABEL-TEMPORARIES-OPTIMIZATION.md). The original [release baseline](research/code-size-baseline-2026-10-20/summary.json) remains immutable.

## Development drivers

Drivers contain different harness/dependency code and are kept separate from shipping CLI/FFI artifacts. Their builds do not use the shipping-size source-path remapping.

| Driver | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| bench | 5,052,704 | 3,749,976 |
| profile | 4,999,328 | 3,710,668 |
| allocator | 5,517,024 | 4,201,640 |
| cross-bench | 9,346,784 | 6,786,880 |

[Both phases, every section and fingerprint](research/label-temporaries-optimization-2026-10-20/driver-sizes.json). Driver changes are not reported as CLI or FFI changes.

## Reproduction

Keep builds and section measurements outside latency timing, tests, CPU sampling, allocator probes and process-resource checks. Create isolated source snapshots for both revisions; preserve all build parameters, features and locked dependency versions. The initial baseline can be reproduced from the repository root:

```sh
mkdir -p target/code-size/baseline-source
git archive 4a03a25c2ef74b393b283751cb04086edd3c2a81 \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision 4a03a25c2ef74b393b283751cb04086edd3c2a81
python3 tools/bench/build-size.py \
  --source . --output target/code-size/after \
  --revision 2a3b5e01b99b96581194f433d637c6c9a5e3e3c7
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

Use the actual source revision label, or `working-tree` when dirty, and a fresh output directory. The builder fingerprints sources before/after compilation, normalizes source paths, rejects local profile/rustflag environment overrides and refuses to replace a frozen manifest. The section comparison rejects different toolchains, declared profiles, commands, normalized environments or artifact sets, and never changes/strips a binary. It describes only its own measurement, not whole-host idleness.

After test-only ports, rebuilding the exact targets produced identical bytes for all four drivers and all three shipping artifacts. The original source snapshots remain preserved, and [the explicit test-only supplement](research/label-temporaries-optimization-2026-10-20/release-revalidation.json) records the later validation sources separately.

Before latency measurements, `python3 tools/bench/check-timing-window.py` can detect listed build/test drivers without stopping processes or storing paths/host details. Its snapshot is not proof of sustained idleness. Preserve every process result, including regressions and contention observations. Record stripping/debug/symbol changes separately from instruction reductions. Do not obtain smaller artifacts by disabling types/features, lowering precision, simplifying output, caching results, reducing inputs or weakening acceptance thresholds.
