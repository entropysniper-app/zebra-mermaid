# Compiled code size

Compiled code size is an additional optimization objective. The existing target is **2× relative to the fixed reference at API return**, using all 167 original Samples and all 33 diagram types. Size has no numeric threshold assigned by the user. Each optimization round must report its size, speed, allocation and process-RSS tradeoffs while preserving the original correctness requirements and full workloads.

This report establishes a release baseline; it does not claim a size reduction. Source revision: `4a03a25c2ef74b393b283751cb04086edd3c2a81`, with production code identical to the already validated `e0a9e608a2c19b0066b6710b76e4f95dc81c1926`. The in-progress scanner/layout candidate is excluded from this baseline.

## Release artifacts

The CLI and FFI were built together from an isolated snapshot of the fixed revision, with locked dependencies, all features, release optimization, thin LTO and one codegen unit. All 33 diagram types remain enabled. Compiler: rustc 1.96.1. Section reader: Apple LLVM 21.0.0 `llvm-size`. Checkout paths are remapped identically in both versions. Artifacts are not stripped.

| Artifact | Entire file bytes | Instruction text bytes |
| --- | ---: | ---: |
| CLI | 5,210,752 | 3,853,944 |
| FFI dynamic library | 4,798,544 | 3,561,420 |
| FFI static archive | 41,708,256 | 4,273,968 |

File size includes data, symbols, metadata, embedded bitcode and archive overhead where present. Instruction text is measured separately from `__text` sections; the tool also recognizes ELF `.text` and `.text.*` sections. Stubs, unwind information, constants and all other sections are retained separately in the evidence. The section reader rejects unreadable archive members rather than treating them as zero-sized code. Only the current artifact format has been exercised; this is not a cross-platform build claim.

The static archive contains **26,008,880 bytes of LLVM bitcode** in addition to machine instructions and other content. Its instruction total sums all archive members before the host linker selects and removes unused code. Neither the archive's file size nor its summed instruction size represents an application's final link increment. Rust `.rlib` metadata is not substituted for a linked release library.

[Release measurements, source fingerprints, artifact SHA-256 values, build configuration and every section total](research/code-size-baseline-2026-10-20/summary.json) identify the baseline. No hardware, host name or local checkout path is recorded in the published evidence.

## Benchmark drivers

Development drivers contain different harness and dependency code and are kept separate from shipping artifacts. These are the existing frozen baseline drivers, not rebuilds with the release-size path mapping:

| Driver | Entire file bytes | Instruction text bytes |
| --- | ---: | ---: |
| Extended benchmark | 5,068,880 | 3,758,756 |
| CPU profiler | 5,015,584 | 3,719,448 |
| Counted allocator | 5,516,912 | 4,201,216 |
| Cross-engine benchmark | 9,363,088 | 6,795,660 |

[Driver measurements and fingerprints](research/code-size-baseline-2026-10-20/drivers.json) match the binaries frozen before this round. A change in a driver is not reported as a change in the CLI or FFI.

## Reproduction

Keep all builds and section measurements outside latency timing, tests, CPU sampling, allocation probes and RSS measurements. From the repository root, create an isolated baseline source snapshot:

```sh
mkdir -p target/code-size/baseline-source
git archive 4a03a25c2ef74b393b283751cb04086edd3c2a81 \
  Cargo.toml Cargo.lock crates tools/profiler \
  | tar -x -C target/code-size/baseline-source
python3 tools/bench/build-size.py \
  --source target/code-size/baseline-source \
  --output target/code-size/before \
  --revision 4a03a25c2ef74b393b283751cb04086edd3c2a81
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --report target/code-size/baseline.json
```

For a candidate, build its source checkout with the same script and configuration, using a fresh output directory and the actual revision label. The script refuses to overwrite a frozen manifest, rejects environment profile/rustflag overrides, and fingerprints sources before and after compilation. Then compare:

```sh
python3 tools/bench/build-size.py \
  --source . --output target/code-size/after \
  --revision working-tree
python3 tools/bench/compare-size.py \
  --before target/code-size/before \
  --after target/code-size/after \
  --report target/code-size/comparison.json
```

The comparison rejects mismatched toolchains, declared release profiles, build commands, normalized build environments or artifact sets. It never strips or modifies the frozen binaries. A same-baseline comparison was exercised for all three artifacts: file and text deltas were zero and artifact fingerprints matched. The CLI text total was also checked using `size -m`.

Record stripping and debug/symbol changes separately from instruction reductions. Do not obtain smaller artifacts by disabling diagram types or features, weakening precision, simplifying output, caching results, reducing workloads or changing acceptance thresholds. Compiler/profile changes require a fresh paired speed and resource measurement as well as the size comparison.
