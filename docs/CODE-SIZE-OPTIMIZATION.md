# Compiled code size

Current verified source: `2f2c3b097dcaa1e9d15a448f9cda0822049301a3`, aligned with Pele 1.9.0 and using RaTeX 0.1.15 by default. All 33 types and built-in math remain enabled. Continuous optimization remains stopped at the user's request; this is a finite measurement after the version upgrade. The **2× API-return performance target remains unmet**. Size has no user-assigned numeric threshold.

## Current shipping artifacts

Release/thin LTO/one codegen unit, locked dependencies, rustc 1.96.1; no stripping. Section reader: LLVM 21.0.0 `llvm-size`. Source checkout paths are remapped consistently. This describes the recorded build and does not claim a cross-platform footprint.

| Artifact | Current file bytes | Current instruction text bytes |
| --- | ---: | ---: |
| CLI | 9,237,040 | 5,649,348 |
| FFI dynamic library | 8,811,776 | 5,326,912 |
| FFI static archive | 62,027,464 | 6,400,400 |

The version upgrade adds functionality, including the RaTeX parser/layout/SVG pipeline and embedded math fonts. All three shipping artifacts grew relative to the preceding full-feature production baseline. These are measured feature/size tradeoffs, not code-size optimization results; the comparison does not isolate one dependency's contribution.

| Artifact | File increase, bytes | Instruction text increase, bytes |
| --- | ---: | ---: |
| CLI | +4,076,960 | +1,851,432 |
| FFI dynamic library | +4,045,232 | +1,810,320 |
| FFI static archive | +20,434,384 | +2,171,144 |

The frozen baseline binaries match the fingerprints in the preceding [production snapshot](research/lazy-token-state-optimization-2026-10-21/size-comparison.json), whose runtime source was `b85c7c10bcc0db77d4758f7ad0c4a029257a56fe`. The original frozen manifest label (`working-tree-over-68a20d8`) is preserved rather than relabeled; artifact SHA-256 values establish which binaries were compared. The new manifest identifies `2f2c3b0`. Both revisions use the same toolchain, release configuration, artifact set and normalized build environment; each uses its own locked dependencies. Dependencies change intentionally with the math integration.

[Current paired sections, source hashes, artifact hashes and build configuration](research/pele-1.9.0-alignment-2026-10-23/size-comparison.json) · [validation](research/pele-1.9.0-alignment-2026-10-23/validation.json). Previous raw data and the [first release baseline](research/code-size-baseline-2026-10-20/summary.json) retain their original meaning.

File bytes include data, symbols, metadata, embedded bitcode and archive overhead. Instruction text sums `__text` (or ELF `.text` / `.text.*`) separately from stubs, unwind tables and other sections. An unreadable archive member fails measurement rather than becoming zero-sized code. Static archive text includes every member before the host linker selects/removes unused code; neither its file nor text total is the host application's final link increment. Rust `.rlib` metadata is not substituted for a linked library.

## Timing and development drivers

The current full 167-sample/33-type API and UTF-8 speedups are **1.76× / 1.89×**. The original samples contain no formulas; this does not measure math throughput. See the [complete timing table](OUTPUT-BENCHMARK.md). Allocation counts, requested bytes, peak live allocation and actual process RSS were not remeasured in this alignment run; older memory data are not current-build claims.

Development drivers contain different harness/dependency code and are separate from shipping CLI/FFI artifacts. Their builds do not use the shipping-size source-path remapping. For example, the benchmark-run CLI file is 9,237,136 bytes, while the remapped shipping CLI above is 9,237,040 bytes. Only the shipping build is used in the paired section comparison. The previous [driver-size snapshot](research/lazy-token-state-optimization-2026-10-21/driver-sizes.json) is historical and was not refreshed as a current four-driver report.

## Reproduction

Keep builds and section measurements outside latency timing, tests, CPU sampling, allocator probes and process-resource checks. Create isolated source snapshots; preserve build parameters, features and each revision's locked dependencies. From the repository root:

```sh
mkdir -p target/code-size/baseline-source target/code-size/current-source
git archive b85c7c10bcc0db77d4758f7ad0c4a029257a56fe \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
git archive 2f2c3b097dcaa1e9d15a448f9cda0822049301a3 \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/current-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision b85c7c10bcc0db77d4758f7ad0c4a029257a56fe
python3 tools/bench/build-size.py \
  --source target/code-size/current-source \
  --output target/code-size/after \
  --revision 2f2c3b097dcaa1e9d15a448f9cda0822049301a3
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

Use an accurate revision label, or `working-tree` when dirty, and a fresh output directory. The builder fingerprints sources before/after compilation, normalizes source paths, rejects local profile/rustflag overrides and refuses to replace a frozen manifest. The section comparison rejects mismatched toolchains, declared profiles, commands, normalized environments or artifact sets, and never changes/strips a binary. It describes only its own measurement, not whole-host idleness.

Before latency measurements, `python3 tools/bench/check-timing-window.py` can detect listed build/test drivers without stopping processes or storing paths/host details. Its snapshot does not prove sustained idleness. Preserve every process result and regression. Record stripping/debug/symbol changes separately from instruction reductions. Do not shrink artifacts by disabling diagram types or built-in math, changing workloads, lowering precision, simplifying output, caching results or weakening acceptance thresholds.
