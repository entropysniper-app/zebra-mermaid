# Class 与 Agentflow：按 profiling 推进的优化

本页保存 Class 共享存储阶段的测量快照。后续 Agentflow 所属 flow 索引优化、39 组配对复测及长循环/系统泄漏检查见 [CPU 再优化与内存验证](PROFILING-CPU-MEMORY.zh-CN.md)；下文剩余热点描述对应本页的历史阶段。

本轮先将 Sequence、Class、Agentflow 的 172 项原始压力测试完整迁移，再用失败用例和 CPU/分配采样定位开销，逐步补齐 Pele 已有的缓存并调整 Rust 存储。原始输入、实际重复次数和循环保持不变；169 项计时回调保留 3 秒性能门槛与 30 秒超时，另外 3 项默认输入限制测试保留原来的 60 秒超时。最终原版 Pele 与 Rust 的这 172 项测试均通过。

## 改动与依据

1. **复用 Pele 的声明缓存和 CSS 类组合缓存。** 上游 `src/diagrams/class/render.ts` 的 `effect` 与 `fromDefs` 缓存避免重复解析同一声明。Rust 原先缺少这部分，在每个类上反复调用 `resolve_style`。现在每次绘制内缓存声明效果和基础样式；使用属性索引保留首次出现的输出顺序，同时让最后有效声明覆盖旧值。声明文本借用模型，不跨渲染保留缓存。
2. **共享不可变样式字符串。** 上游 JavaScript 将同一批不可变字符串引用追加到多个类，Rust 原先复制每个 `String`。现在类之间共享 `Arc<String>`，每个类仍保留完整、有序、含重复值的样式列表。高扇出列表使用薄指针的 `Arc<String>`，保留模型的线程安全能力；没有添加可变共享状态或锁。Arc 克隆共享同一堆分配，这与此处的不可变声明用途匹配。[Rust Arc 文档](https://doc.rust-lang.org/std/sync/struct.Arc.html)
3. **将成员删除与序列化顺序分开处理。** Agentflow 原先用 `IndexSet::shift_remove` 删除每个全局节点，会移动后续成员。现在用带插入序号的 `IndexMap` 做 `swap_remove`，序列化时按序号输出原来的字符串数组；解析热路径不排序。直接遍历 `DocItem`，同时去掉了临时借用引用数组。子图节点的过滤与声明顺序继续保留。

Class 的 Rust 模型字段 `ClassNode.styles` 现在是 `Vec<Arc<String>>`；直接操作该模型的 Rust 调用方需要使用共享字符串类型，例如 `Arc::new(value)`，读取可继续用 `as_str()`。模型 JSON 仍为字符串数组，包含原来的重复值和顺序。Serde 的 `rc` 支持会序列化每个引用的值，不会把相同值自动去重。[Serde feature flags](https://serde.rs/feature-flags.html)

SVG、模型 JSON 与 Zebra C ABI 的兼容性分别验证。新增的两个项目回归测试先在旧实现通过，再约束新实现：样式声明的重复/覆盖顺序，以及全局块操作后的成员顺序和 JSON 字段集合。它们不计入原始测试的 1:1 通过数。

## Class 两个压力场景

下表的测试耗时来自调试构建的原始压力测试，包含输入构造和完整操作，是各阶段实际验收记录；不能与 release benchmark 或 Pele 的 Vitest 用例耗时直接作语言速度比较。分配数据来自独立 release 计数器，预热五批后记录一个完整渲染批次，输入在计数前创建。累计申请量包含 realloc 的新请求尺寸；存活峰值为相对批次开始的额外请求字节，均不是 RSS。

| 原始场景 | 实现阶段 | 测试耗时 ms | 分配次数 | 累计申请 MiB | 额外存活峰值 MiB |
| --- | --- | ---: | ---: | ---: | ---: |
| 1,600 classDef × 1,600 类 | 缺少绘制缓存 | 12668.40 | 69,228,985 | 1835.33 | 101.87 |
| 1,600 classDef × 1,600 类 | 加入绘制缓存 | 1410.27 | 2,665,803 | 189.51 | 101.87 |
| 1,600 classDef × 1,600 类 | 缓存 + 共享字符串 | 1179.90 | 107,403 | 67.60 | 29.97 |
| 5,000 声明 × 800 类 | 缺少绘制缓存 | 6879.83 | 36,065,591 | 186.52 | 119.79 |
| 5,000 声明 × 800 类 | 加入绘制缓存 | 1012.51 | 4,079,208 | 127.72 | 120.05 |
| 5,000 声明 × 800 类 | 缓存 + 共享字符串 | 860.69 | 84,209 | 40.73 | 33.39 |

最初两个测试分别耗时约 12.67 秒、6.88 秒，超过原来的 3 秒限制。缓存修复后通过；共享存储进一步降低了数据复制和峰值内存。相对已经加入缓存的版本，两个场景峰值分别下降 **70.58% / 72.19%**，累计申请字节下降 **64.33% / 68.11%**。两个场景在优化前、缓存版本、共享版本与固定 Pele 上的 SVG 字节完全一致，尺寸相同。

## Agentflow 全局节点

原始场景为“3,000 个节点的 flow 后跟 3,000 个 global 块”。

| 指标 | 改动前 | 改动后 |
| --- | ---: | ---: |
| 原始调试测试耗时 ms | 2545.10 | 867.13 |
| 分配次数 | 198,265 | 195,265 |
| 累计申请 bytes | 37,058,250 | 37,062,922 |
| 额外存活峰值 bytes | 11,241,459 | 11,270,131 |

删除路径与临时数组优化让该调试验证从约 2.55 秒降到 0.87 秒。保序索引增加约 **28 KiB** 峰值内存；分配次数减少 3,000 次。这里的收益主要是耗时，不能宣称这一组的所有内存指标都下降。优化前后与 Pele 的 SVG 完整字节相同，新增序列化测试也保持通过。

## CPU 栈证据与剩余热点

对完整渲染负载使用外部采样器，5 秒、间隔 1 ms。计数包含子调用和递归，不是互斥 CPU 百分比，也不能相加。

- 缓存前，Class 场景的 3,379 个主线程样本中有 2,815 个调用栈包含 `resolve_style`。
- 加入缓存后，热点转到模型构建与声明列表复制，`define_class` 出现在 3,899 个样本中的 1,807 个，推动了共享存储改动。
- 共享后，`ClassStyles::keep` 的重复声明遍历仍是后续优化候选；解析和渲染成本都保留在采样中。
- Agentflow 的 `add_global` 仍是热点，尤其是对子图节点列表的过滤。当前改动减少了集合删除成本，尚未消除这一场景的全部重复扫描。

每个已测渲染批次结束时的净新增存活字节均为 0。这只说明该批次未保留额外分配，不替代通用泄漏检测。

## 常规 benchmark 与验证

本轮专门对“缓存版本”与“共享存储/保序删除版本”进行三轮独立进程、交替顺序的 39 组配对测量。仍为 5 批预热、至少约 10 ms 校准、每轮至少 9 样本且累计至少 300 ms，成功渲染/解析/拒绝分开计时。39/39 组的全部验证记录一致。

这次 33 类文档 render 比值几何平均为 **1.0040**，即普通工作负载整体略慢约 **0.40%**。该小幅变化不能被包装为总体加速；这一阶段的明确收益是上述压力场景及其内存。原始绘制缓存阶段另外相对初始 Rust 基线测得约 5.77% 的总体下降，两次对照基线不同，不能把比例相加。

已另外重跑当前 Rust 与 Pele 的原脚本五组 benchmark 及 33 类、39 组扩展对比，见 [Pele 原格式/口径表](BENCHMARK-PELE-PROTOCOL.zh-CN.md) 和 [完整扩展报告](BENCHMARK.zh-CN.md)。当前扩展报告中 33 类文档 render 的 Rust/Pele 几何平均为 **0.895**；逐组慢项和独立 parse/rejection 数值一并保留。

当前工作区全 features、采集全集、主题与 ABI 回归为 **3,015 通过、0 失败、5 跳过**。原始测试逐项门禁为 **2,629/6,810 通过、5 跳过、4,176 未映射**，完整目标仍未验收通过。

## 原始数据与复现

- [绘制缓存阶段摘要与分配数据](research/class-style-optimization-2026-10-19/summary.json)
- [共享存储/保序删除阶段摘要](research/shared-style-optimization-2026-10-19/summary.json)
- [39 组配对原始结果](research/shared-style-optimization-2026-10-19/benchmark-comparison.json)
- [共享阶段 SVG 与 Pele 对照](research/shared-style-optimization-2026-10-19/reference.json)
- [当前原始压力测试逐项结果](research/shared-style-optimization-2026-10-19/stress-native.json)
- [源码及 benchmark 二进制哈希](research/shared-style-optimization-2026-10-19/metadata.json)

两个数据目录均保留 CPU 聚合、分配计数、冻结二进制指纹、各轮压缩报告。原始系统调用栈留在忽略的 target 目录，公开数据不含主目录、主机名或硬件信息。共享阶段的 Class 优化前分配/CPU 数据复用前一阶段已冻结的缓存版本；Agentflow 旧版本单独重新采样，未把复用数据写成一次新测量。

```sh
node tools/test-parity/compile-stress.mjs --check --emit-profile-cases
node tools/test-parity/run-stress.mjs
node tools/test-parity/check.mjs --require-complete

cargo build --release -p zebra-mermaid -p zebra-mermaid-profiler \
  -p zebra-mermaid-cli --example profile --example bench --example bench_upstream \
  --bin zebra-mermaid-profiler --bin zebra-mermaid

# 该脚本冻结二进制到阶段目录；已有阶段不覆盖。
ZM_STYLE_PROFILE_DIR=target/profiling/new-experiment \
  python3 tools/bench/profile-class-style.py before
# 在完成实现改动及重新构建后，以 after 阶段采样。

# 配对时明确指定自己的优化前冻结二进制，输出到独立目录。
ZM_BENCH_BEFORE=target/profiling/new-experiment/before/bench \
ZM_BENCH_OUTPUT=target/profiling/new-experiment/paired \
  python3 tools/bench/compare-optimization.py

node tools/bench/run-upstream.mjs
BENCH_MS=300 BENCH_SAMPLES=9 node tools/bench/run.mjs
```

完整门禁当前会因未映射项和原始跳过而非零退出；不得把该结果解释为已完成全部测试。对照前冻结 benchmark 二进制时须另行保存 `release/examples/bench`；CPU/分配脚本保存的是 profile、allocator 与 CLI。可通过 `CARGO_TARGET_DIR` 指定构建目录；CPU 栈采样要求系统提供 `sample` 命令。
