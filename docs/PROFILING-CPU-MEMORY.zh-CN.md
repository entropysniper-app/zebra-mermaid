# Agentflow CPU 优化与持续内存验证

本轮继续基于上一阶段的 CPU 采样优化 Agentflow，并独立检查 Rust 库、模型复用、缓存和 Zebra C ABI 的内存释放。固定参照仍是 Pele `44da152bbb79b91f03e68016f42499bb25bf6e82`。优化前基线已经包含 Class 声明缓存、共享样式字符串和 Agentflow 的 `swap_remove`；本轮收益不能再与此前各阶段百分比相加。

## CPU 改动及测量

Pele 的 Agentflow 数据库保证一个节点最多属于一个 flow，重复声明同一个 flow 会合并成员，global 节点永久排除在 flow 之外。Rust 利用这些约束，在私有成员索引中记录所属 flow：单节点 global 块直接在所属 flow 中保序删除；批量 global 块只过滤受影响的 flow，且每个 flow 只过滤一次。成员插入序号继续负责模型 JSON 的顺序。

这延续了 Pele 避免重复工作的思路，但所属 flow 索引是 Rust 本轮新增的实现优化。原版 Pele 的 `addGlobal` 仍扫描所有子图。单节点处理不创建临时数组、不逐个对剩余节点做 global 集合查询；`Vec::remove` 仍会移动后续元素，因此不能将整个处理过程声称为线性复杂度。生产库没有新增 unsafe。

| 指标：3,000 个节点后接 3,000 个 global 块 | 优化前 | 优化后 |
| --- | ---: | ---: |
| release 完整渲染耗时 | 52.845 ms | 9.572 ms |
| 分配次数 | 195,265 | 195,265 |
| realloc 次数 | 21,210 | 21,210 |
| 累计申请字节 | 37,062,922 | 37,091,594 |
| 额外存活字节峰值 | 11,270,131 | 11,298,803 |
| 单批次结束后净保留字节 | 0 | 0 |

该压力场景耗时下降 **81.89%**，加速 **5.52 倍**。私有 owner 索引增加 **28 KiB** 额外存活峰值，约 **0.25%**；本轮没有宣称该组内存也下降。

耗时来自三对独立进程，交替运行前后版本，各预热五批后运行至少三秒完整渲染循环，取三次循环平均耗时的中位数。这是压力场景专项指标，独立于 Pele 原脚本表格。计时进程没有装载分配计数器或 CPU 采样器。分配统计在另一个进程中进行，累计申请量含 realloc 的完整新请求，不是 RSS。

外部 CPU 采样为五秒、间隔 1 ms：优化前 4,064 个主线程样本中，3,190 个调用栈包含 `add_global`；优化后为 3,691 个样本中的 485 个。它们是包含子调用的栈计数，不是互斥 CPU 百分比。完整渲染热点现已更多转到正常解析和绘制。

## 常规 benchmark 与 Pele 原格式

39 组、879 个输入在六次前后版本运行中的行为验证均一致：820 个成功渲染、59 个预期拒绝。33 类文档 render 耗时比的等权几何平均为 **0.98561**，即本次测量低约 **1.44%**。常规 Agentflow 文档组为 0.50354 → 0.50523 ms（+0.34%），参考组为 3.31555 → 3.32547 ms（+0.30%）。这些小幅变化不能支持常规 Agentflow 显著加速的结论；明确的大幅收益来自大量 global 块的压力路径。

常规基准保持原有扩展协议：五次预热、至少 10 ms 校准、每轮至少九个样本且累计至少 300 ms，三轮独立进程、交替顺序。成功 render、parse 和 rejection 分开计时；所有原始样本保留。

另直接执行未修改的 Pele `scripts/bench.mjs`，Rust 使用同样输入与计时循环，结果按原格式如下：

| case | diagrams | pele | zebra-mermaid |
| --- | ---: | ---: | ---: |
| docs: all examples | 114 | 2.6 ms | 1.6 ms |
| tree: 50 nodes | 1 | 0.250 ms | 0.226 ms |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.716 ms | 0.448 ms |
| tree: 500 nodes | 1 | 3.0 ms | 2.3 ms |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 9.6 ms | 6.0 ms |

这是整批输入耗时，第一行包含 114 张图。预热一次、至少 300 ms 且至少五个样本、最多 2,000 次、排序取上中位数；118 个 SVG 完整字符串及尺寸均与 Pele 一致。完整方法和原始记录见 [原脚本报告](BENCHMARK-PELE-PROTOCOL.zh-CN.md)。

## 内存检查方法

开发专用程序 `zebra-mermaid-memory-check` 包装 System 分配器，分别记录存活请求字节和分配块数。分配、零初始化分配、realloc 成功/失败、释放分别按所有权计数；不修改生产分配器。输入与预热不计入净增量。每一轮都在输出、模型释放及缓存 clear 后核对基线；并非只检查最后一轮。

覆盖路径包括：完整 render；解析后克隆模型、释放原模型再重复绘制；缓存命中、淘汰、clear、drop；错误解析；C ABI 成功结果和错误缓冲区的独立释放、整体释放与重复 destroy。缓存还通过 Weak 引用确认淘汰确实释放对象，且调用方仍持有的输出在 clear 后有效、最后一个调用方释放后消失。缓存容器 clear 后可保留容量，drop 后另行核对。

计数器的受控验证故意 `forget` 一个 4,096 字节的分配，确认观测到 4,096 字节、1 块，再重建原 Vec 释放。独立系统 `leaks` 负面对照进程故意遗留一块内存，被检测为 **1 块、5,120 字节**，其中系统分配尺寸与请求尺寸不同。正常 render、parsed、cache、ffi 四个进程各运行十轮后，系统检测均报告 **0 块、0 字节泄漏**。

上述仪器化进程的运行时间不用于 CPU 性能比较。系统原始日志只保留在忽略目录，公开报告保存结果与哈希，不包含本机路径或系统信息。

## 持续检查结果与 RSS

| 场景 | 输入数 | 轮数 | 每轮结束净保留（字节 / 块） | 首/末检查点 RSS MiB | 观测 RSS 最大 MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 常规完整渲染 | 879 | 1000 | 0 / 0 | 14.72 / 17.59 | 17.59 |
| 常规模型克隆/复用 | 879 | 200 | 0 / 0 | 15.39 / 18.45 | 18.45 |
| 常规缓存命中/淘汰/清空 | 879 | 200 | 0 / 0 | 16.23 / 20.61 | 20.61 |
| 常规 C ABI + 错误缓冲区 | 879 | 200 | 0 / 0 | 16.52 / 17.81 | 17.81 |
| 三组大型输入完整渲染 | 3 | 100 | 0 / 0 | 127.30 / 127.47 | 127.47 |
| 三组大型输入模型复用 | 3 | 30 | 0 / 0 | 173.11 / 201.33 | 201.33 |
| 三组大型输入缓存 | 3 | 30 | 0 / 0 | 121.84 / 125.53 | 125.53 |
| 三组大型输入模型复用 | 3 | 300 | 0 / 0 | 163.84 / 191.39 | 202.16 |
| 三组大型输入缓存 | 3 | 300 | 0 / 0 | 136.55 / 144.95 | 149.30 |

每个表格行是独立进程。合计 **1,408,680 个输入处理轮次**，包含同一输入的重复运行，不代表不同测试数量；不包含五批预热、额外 ABI 无效参数检查或独立系统 leaks 运行。常规语料每轮为 820 个成功与 59 个预期错误；C ABI 每轮额外覆盖五项无效源/UTF-8/调色板/字号/语法输入，因此为 820 个成功与 64 个错误。

RSS 在子进程释放对象后暂停时采集，每次进程记录二十个检查点。大型模型复用会保留克隆模型，再绘制两次，压力显著高于单次 render。其加长测试曾在约 202.16 MiB 稳定多个检查点，末期回落到 191.39 MiB；缓存压力进程也有上涨与回落。分配器可能保留已释放对象对应的页，clear 也允许缓存容器保留容量，所以 **RSS 不回到进程启动值并不等于对象泄漏**。本报告保留这些波动，没有将其写成“RSS 恒定”或“内存全部归还系统”。

结论是：**在本次输入、轮次和四条系统检测路径内，未发现内存泄漏**。每轮的存活请求字节/块没有新增，所有权控制与系统检测均通过。该结论不覆盖所有可能输入、其他平台或原生 Zebra App 的集成生命周期，也不代表任意调用方忘记释放 C ABI 缓冲区时仍不会泄漏。

## 回归与验收范围

- 全 features 工作区与采集全集回归：**3,016 通过、0 失败、5 原有跳过**，包括 70,183 条可表示采集输入、主题与 ABI 回归。
- 全部 172 项已迁移原始压力测试通过，输入、迭代数、3 秒门槛与原始超时未改变。Agentflow 对应原始调试测试本轮约 74.04 ms；它与上面的 release 完整渲染计时范围不同，不能混作同一指标。
- 新增项目回归用例覆盖嵌套 flow、重复声明、一个 global 块影响多个 flow、重复 global ID，以及后续单节点释放。测试先在旧 Rust 通过，再独立对照固定 Pele 成员/全局/子图顺序，最后验证新实现。它不计入原始测试映射数量。
- 逐项独立进程审计：**2,629/6,810 通过，0 失败，5 跳过，4,176 未映射，0 待验证**。原始测试完整目标仍未完成；本轮通过率仍为 **38.60%**。
- Rustfmt（本轮 Rust 文件）与 diff 空白检查通过。常规 Clippy 执行完成但仍有警告，包括可选 serde 才读取的成员序号及开发探针的取模风格建议，未宣称无警告验收。

## 数据与出处

- [总摘要、测试结果及测量范围](research/agentflow-memory-2026-10-19/summary.json)
- [压力场景三对原始计时](research/agentflow-memory-2026-10-19/timing.json)
- [39 组配对 benchmark](research/agentflow-memory-2026-10-19/benchmark-comparison.json)；同目录保留六次完整压缩结果。
- [持续分配、RSS、系统 leaks 与控制实验](research/agentflow-memory-2026-10-19/memory.json)
- [大型模型/缓存加长到 300 轮](research/agentflow-memory-2026-10-19/memory-soak.json)
- [压力 SVG 与 Pele 逐字节对照](research/agentflow-memory-2026-10-19/reference.json) · [嵌套/批量模型顺序对照](research/agentflow-memory-2026-10-19/model-reference.json)
- [源码与输入指纹](research/agentflow-memory-2026-10-19/metadata.json)；同目录保留前后 CPU 栈聚合、分配数据及冻结二进制指纹。



## 复现

```sh
# 所有命令从仓库根目录执行；研究参照与基准输入需先生成。
cargo build --release -p zebra-mermaid --all-features \
  --example profile --example bench --example bench_upstream
cargo build --release -p zebra-mermaid-cli -p zebra-mermaid-profiler --bins

# 优化前先冻结 profile、bench、CLI、allocator，再实现改动并重新构建。
# profile-agentflow-owners.py 对应本轮固定的 before 目录，不覆盖已测结果。
python3 tools/bench/profile-agentflow-owners.py
ZM_BENCH_BEFORE=target/profiling/agentflow-owners/before/bench \
ZM_BENCH_OUTPUT=target/profiling/agentflow-owners/paired \
  python3 tools/bench/compare-optimization.py

# 内存脚本拒绝覆盖已有输出；重测时指定一个新的目录。
ZM_MEMORY_OUTPUT=target/profiling/memory-repeat python3 tools/bench/check-memory.py
ZM_MEMORY_OUTPUT=target/profiling/memory-soak-repeat \
  python3 tools/bench/check-memory.py --stress-soak

node tools/bench/run-upstream.mjs
ZM_FULL_HARVEST=1 cargo test --workspace --all-features
node tools/test-parity/run-stress.mjs
node tools/test-parity/check.mjs --run --isolate
```

可设置 `CARGO_TARGET_DIR` 指向独立构建目录。计数器与 RSS 检查支持独立运行；当前系统泄漏交叉检查使用 `leaks`，CPU 栈采样使用 `sample`。在其他平台未运行的系统检测不能被当成通过。原始测试全量验收仍使用 `--require-complete`，未覆盖项存在时不会通过。
