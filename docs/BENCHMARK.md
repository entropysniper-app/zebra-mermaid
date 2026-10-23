# Performance comparison

Current source: `2f2c3b097dcaa1e9d15a448f9cda0822049301a3`, with Pele **1.9.0** fixed at `e7f092a4de147cc32210913c20e36eaa5fcd0ef1`. Measurements were refreshed on 2026-10-23 after the default RaTeX integration. The library retains all 33 diagram types.

| Endpoint | Inputs | Equal-weight geometric mean speedup |
| --- | --- | ---: |
| API return | All 167 original samples, 33 types | **1.76×** |
| Contiguous UTF-8 output | The same 167 samples, 33 types | **1.89×** |

The **2× API-return target remains unmet**. Across the 33 types, API speedup ranges from agentflow 1.20× to sankey 2.77×; 23 types are below 2×. These averages describe this workload, not arbitrary application throughput. None of these original samples contains a math formula, so the table does not measure formula typesetting throughput. Continuous optimization remains stopped.

Both endpoints include full parsing, layout, and SVG generation with no parsed-model or output-cache reuse. Node's UTF-8 endpoint additionally encodes and allocates a Buffer; it is a different output contract. Three independent process runs alternate engine order, with five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Builds, tests, browser rendering and probes run outside timing.

The [complete 33-type table](OUTPUT-BENCHMARK.md) includes every input group and all raw process timings. Independent full SVG validation succeeds on 167/167 original samples. The refreshed [rendering gallery](render-comparison/index.html) also renders the same sources with Mermaid.js; it is a rendering comparison, not a current four-engine latency benchmark.

The [extended report](BENCHMARK.zh-CN.md) preserves all 39 workloads: 879 input occurrences, 820 successful renders and 59 expected rejections. All 39 groups agree, with complete response verification recorded separately. Its 33 documentation batches have Rust/Pele render-time geometric mean **0.53938** and parse-time mean **0.50958**. That input set differs from the primary 167-sample target and cannot replace it.

The [original Pele protocol](BENCHMARK-PELE-PROTOCOL.zh-CN.md) was also rerun directly: all five workloads, 118 inputs, complete SVG strings and dimensions equal. This single-run protocol has its own warm-up and median rules; it is kept separate from the three-process comparison.

[Current release footprint](CODE-SIZE-OPTIMIZATION.md) reports shipping CLI, FFI dynamic library and static archive file/text bytes separately. The upgrade and built-in math add code and font data; this is a feature addition with a measured size increase. Allocation, RSS and CPU profiling were not repeated in this alignment run. Historical resource and four-engine results remain in their original evidence directories and do not describe the current implementation.

[Current verification evidence](research/pele-1.9.0-alignment-2026-10-23/validation.json) records the remaining limits. Original-test acceptance is **2,808/6,971**, with 7 preserved skips and 4,156 unmapped tests; complete acceptance is unfinished.
