# 多图表 CPU 优化与回归

本轮在上一阶段 Agentflow 所属 flow 索引的基础上，测量 33 类图表、39 组工作负载，并针对公共输出路径及多个绘制器优化。没有改动基准输入或原始测试阈值。

## 改动

1. `util::js::to_string` 对安全整数直接进行整数格式化。尺寸、字号和整数刻度不再先创建指数形式字符串再拆解重组；分数、超出安全范围、指数边界和非有限值继续走既有路径。新增由 Node 独立产生的 2,580 个参考值，覆盖边界、符号和零。
2. 默认 CSS 根样式改为借用静态字符串，权重查找与声明使用常量，减少每张图的短生命周期字符串。CSS 内容和输出顺序保持不变。
3. ER 关系标记只构造其基数类型使用的路径，直接写入一个输出缓冲区，避免为每个坐标、线段以及未使用的叉脚创建临时字符串。
4. 103 处 `push_str(&format!(...))` 改为直接 `write!` 到 String，覆盖公共 SVG、Sequence、Timeline、Journey、XYChart、Railroad、Block。保留相同格式串和参数，不调整几何或数值精度。

优化沿用 Pele 的可观察行为，同时利用 Rust 的借用与可复用缓冲区。未新增跨渲染缓存，也未新增生产 unsafe 或第三方运行时依赖。主题、CSS 配置与 Zebra C ABI 契约保持原样。

## 39 组配对结果

三轮独立进程，前后顺序交替；五批预热，至少 10 ms 校准，每轮至少九个样本且累计至少 300 ms。表中为整批成功输入的 render 时间：先取每轮样本中位数，再取三轮中位数。CPU 采样、分配计数、编译和测试均在计时结束后执行。

**39/39 组全部行为验证一致**，含 820 个成功渲染和 59 个预期拒绝。33 类文档 render 的等权耗时比几何平均为 **0.94463**，即本次测量低约 **5.54%**；其中 22/33 类中位耗时下降。所有变慢项也保留，不将等权平均推广为任意实际应用的总体收益。

| 图表场景 | 优化前 ms | 优化后 ms | 耗时变化 |
| --- | ---: | ---: | ---: |
| agentflow-docs | 0.52086 | 0.54010 | +3.69% |
| architecture-docs | 0.14201 | 0.12819 | -9.73% |
| block-docs | 0.29161 | 0.28674 | -1.67% |
| c4-docs | 0.53604 | 0.53814 | +0.39% |
| class-docs | 0.74039 | 0.74446 | +0.55% |
| cynefin-docs | 0.11021 | 0.10802 | -1.99% |
| er-docs | 0.73317 | 0.71711 | -2.19% |
| eventmodeling-docs | 0.24782 | 0.21845 | -11.85% |
| flowchart-docs | 1.59956 | 1.84769 | +15.51% |
| gantt-docs | 0.31959 | 0.29077 | -9.02% |
| git-docs | 1.16431 | 1.16853 | +0.36% |
| info-docs | 0.00388 | 0.00305 | -21.42% |
| ishikawa-docs | 0.01943 | 0.01799 | -7.43% |
| journey-docs | 0.02070 | 0.01735 | -16.17% |
| kanban-docs | 0.07159 | 0.05958 | -16.77% |
| mindmap-docs | 0.08965 | 0.08275 | -7.70% |
| packet-docs | 0.07039 | 0.05173 | -26.52% |
| pie-docs | 0.02616 | 0.02283 | -12.74% |
| quadrant-docs | 0.05919 | 0.05499 | -7.10% |
| radar-docs | 0.05822 | 0.05238 | -10.03% |
| railroad-docs | 0.36045 | 0.32601 | -9.55% |
| requirement-docs | 0.34605 | 0.29141 | -15.79% |
| sankey-docs | 0.27242 | 0.23800 | -12.63% |
| sequence-docs | 0.69361 | 0.61566 | -11.24% |
| state-docs | 0.74841 | 0.75211 | +0.49% |
| swimlane-docs | 0.53057 | 0.61711 | +16.31% |
| timeline-docs | 0.35008 | 0.33102 | -5.44% |
| treemap-docs | 0.14420 | 0.13359 | -7.36% |
| treeview-docs | 0.11581 | 0.11582 | +0.01% |
| usecase-docs | 0.87906 | 1.22280 | +39.10% |
| venn-docs | 0.24829 | 0.27529 | +10.88% |
| wardley-docs | 0.30028 | 0.31164 | +3.78% |
| xychart-docs | 0.17617 | 0.13361 | -24.16% |

完整 39 行及 parse/rejection 数据见 [配对 JSON](research/multi-diagram-optimization-2026-10-19/benchmark-comparison.json)。第一阶段仅做数字格式、根样式与 ER 标记时测得约 5.96% 的等权下降；该中间数据另存，不能与最终下降百分比相加。

## 波动与逐组复核

首次整轮配对存在较大波动。例如 `docs: all flowchart examples` 与 `corpus: flowchart-docs` 的输入完全相同，但前者约低 3.76%，后者约高 15.51%；Usecase 的优化后单轮中位数分别约为 1.651 / 1.223 / 0.829 ms。因此另外对所有初测回退至少 3% 的场景，加 Flowchart/ER/Sequence 对照，逐组交替运行前后版本各五个独立进程。

复核使用原来的冻结 benchmark 二进制、同一输入顺序、选项与单轮采样规则，仅缩短前后测量的时间间隔。所有初测和复核记录同时保留；不把复核选择性合并为一个新的总体几何平均，也不将波动直接归因于某项实现。

| 场景 | 复核前 ms | 复核后 ms | 耗时变化 |
| --- | ---: | ---: | ---: |
| docs: all flowchart examples | 1.60180 | 1.52015 | -5.10% |
| tree: 50 nodes | 0.22393 | 0.22250 | -0.64% |
| mesh: 50 nodes, 20 extra labelled edges | 0.44358 | 0.43147 | -2.73% |
| tree: 500 nodes | 2.27534 | 2.21272 | -2.75% |
| mesh: 500 nodes, 200 extra labelled edges | 6.38458 | 5.88821 | -7.77% |
| corpus: agentflow-docs | 0.49759 | 0.49651 | -0.22% |
| corpus: agentflow-reference | 3.13662 | 3.02001 | -3.72% |
| corpus: er-docs | 0.70245 | 0.62303 | -11.31% |
| corpus: flowchart-docs | 1.52576 | 1.44612 | -5.22% |
| corpus: sequence-docs | 0.73424 | 0.63043 | -14.14% |
| corpus: swimlane-docs | 0.52169 | 0.50074 | -4.02% |
| corpus: usecase-docs | 0.79070 | 0.76663 | -3.04% |
| corpus: venn-docs | 0.23127 | 0.23671 | +2.35% |
| corpus: wardley-docs | 0.28473 | 0.26485 | -6.98% |

[复核汇总与方法](research/multi-diagram-optimization-2026-10-19/focused-comparison.json)，同目录保存每次进程的完整样本。

## CPU 与分配证据

本轮对 ER、Sequence、Block、XYChart、Journey、Railroad、Timeline、Class、Info 九组分别采样优化前后 CPU 栈。采样为 5 秒、1 ms 间隔，计数包含子调用及可能的预热，不是互斥 CPU 百分比。原始系统日志保留在忽略目录，公开文件只包含符号汇总和哈希。

| 场景 | 分配次数变化 | 累计申请字节变化 | 额外存活峰值变化 |
| --- | ---: | ---: | ---: |
| er-docs | -10.60% | -1.97% | -0.40% |
| sequence-docs | -15.49% | -17.52% | -0.45% |
| block-docs | -11.60% | -8.19% | +7.64% |
| xychart-docs | -34.14% | -20.53% | +4.88% |
| journey-docs | -27.02% | -15.24% | -0.71% |
| railroad-docs | -17.86% | -15.06% | +19.29% |
| timeline-docs | -12.21% | -2.72% | -0.64% |
| info-docs | -25.00% | -7.86% | -12.42% |

分配数据为独立计数器预热五批后的单批测量。累计申请量包括 realloc 的新请求，不是 RSS；每组结束净新增存活字节均为 0。全部 39 组的前后原始计数见 [分配记录](research/multi-diagram-optimization-2026-10-19/allocations.json)。

## 内存与正确性回归

- 内存探针重新执行常规渲染 1,000 轮、模型复用/缓存/C ABI 各 200 轮，以及三组大型输入；合计 **1,406,880 个输入处理轮次**。每轮净保留均为 0 字节、0 分配块。
- 系统 leaks 对 render、parsed、cache、ffi 四条路径均报告 0 泄漏；独立故意遗留分配的控制进程成功检出。仅对本次场景作出未发现泄漏的结论，RSS 波动另存 [内存报告](research/multi-diagram-optimization-2026-10-19/memory.json)。
- 全 features 工作区和采集全集：**3,017 通过、0 失败、5 原有跳过**，含 70,183 条可表示采集输入、主题与 ABI 回归。
- 逐项独立超时审计：**2,629/6,810 通过，0 失败，5 跳过，4,176 未映射**；其中全部 172 项已迁移原始压力测试通过。新增数字格式项目回归不计入原始测试通过数。
- 四引擎比较另外验证 Pele/zebra 的 167 个 Sample SVG 字节与尺寸一致；与 Mermaid.js/mmdr 的共同输入测量见 [四引擎横向报告](BENCHMARK-FOUR-ENGINES.zh-CN.md)，不混用其样本数与本表 879 个输入。

## 复现和数据

```sh
node tools/bench/generate-number-reference.mjs --check
cargo test -p zebra-mermaid --all-features --test number_format
cargo build --release -p zebra-mermaid --all-features --example bench --example profile --example bench_upstream
cargo build --release -p zebra-mermaid-cli -p zebra-mermaid-profiler --bins
# 改动前先冻结旧版 bench；当前版使用新的输出目录。
ZM_BENCH_BEFORE=target/profiling/multi-diagram/before/bench \
ZM_BENCH_OUTPUT=target/profiling/new-comparison python3 tools/bench/compare-optimization.py
ZM_MEMORY_OUTPUT=target/profiling/new-memory python3 tools/bench/check-memory.py
ZM_FULL_HARVEST=1 cargo test --workspace --all-features
node tools/test-parity/check.mjs --run --isolate
```

支持 `CARGO_TARGET_DIR`。原始测量见 [摘要](research/multi-diagram-optimization-2026-10-19/summary.json)、[优化前 CPU](research/multi-diagram-optimization-2026-10-19/before-cpu.json)、[优化后 CPU](research/multi-diagram-optimization-2026-10-19/after-cpu.json)、[源码和二进制指纹](research/multi-diagram-optimization-2026-10-19/after-metadata.json)。同目录保存优化前元数据、六份完整计时、所有内存与原始测试审计摘要。
