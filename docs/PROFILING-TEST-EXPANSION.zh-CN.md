# 测试扩展后的性能与资源复测

本次测量覆盖 Flowchart 模型字段、Class 回调参数和 Sequence 测试扩展后的当前代码。使用与首轮相同的冻结优化前 Rust 基线 `4761e84`，重新执行三轮独立进程、交替先后顺序的配对 benchmark；这不是在首轮收益上叠加的一次新优化。

逐项原始测试进度为 2,457/6,810，完整工作区回归为 2,841 通过、0 失败、5 跳过。全量原始测试验收仍未完成，性能测量不替代正确性验收。

39/39 组的六份验证记录完全一致，包含成功/拒绝数量、SVG 哈希、尺寸与错误字段。33 类文档工作负载等权 render 耗时比的几何平均为 **0.944320**，对应耗时降低 **5.57%**。每轮仍预热 5 批、校准至少约 10 ms、采样至少 9 个且累计至少 300 ms；成功渲染、解析和预期拒绝分别计时。

| 工作负载 | 优化前 render ms/批 | 当前 render ms/批 | 耗时变化 | 累计申请量下降 | 额外存活峰值下降 |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.65249 | 1.51357 | -8.41% | 15.68% | 15.06% |
| tree: 50 nodes | 0.26741 | 0.22337 | -16.47% | 13.87% | 8.77% |
| mesh: 500 nodes, 200 extra labelled edges | 6.12552 | 5.75479 | -6.05% | 7.21% | 4.51% |
| corpus: agentflow-docs | 0.51105 | 0.51294 | +0.37% | 1.54% | 1.05% |

时间变化为当前/优化前减一，负值表示更快。agentflow 文档组本次约慢 0.37%；不据此将所有工作负载概括为加速，也不把单次复测差异解释为确定回归。33 类几何平均与首轮约 5.6% 的测量接近。

内存表的百分比相对首轮冻结的优化前分配基线，使用同一输入和分配计数器。四组本次测量结束时净新增存活字节均为 0。新增 Flowchart 图模型字段后，累计申请量和存活峰值低于优化前，但下降幅度略小于第一轮；因此保留旧快照，同时单独报告本次数据。

## 进程 CPU 与内存

每引擎每组分别运行三个新进程，预热 5 批、循环至少 1 秒。CPU 时间和 RSS 包含启动、模块与输入加载、预热；CPU/批次不是纯渲染耗时，RSS 也不是渲染算法自身的内存。Rust/Pele 每批成功/拒绝数量和 SVG 字节数均相同。

| 工作负载 | Rust CPU ms/批 | Pele CPU ms/批 | Rust RSS MiB | Pele RSS MiB | Rust 额外存活峰值 KiB | alloc / realloc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.54616 | 3.63740 | 5.28 | 150.62 | 50.24 | 40268 / 4965 |
| tree: 50 nodes | 0.22170 | 0.36533 | 5.02 | 151.97 | 258.09 | 4946 / 863 |
| mesh: 500 nodes, 200 extra labelled edges | 6.35762 | 23.32912 | 11.61 | 227.75 | 3985.80 | 71365 / 18005 |
| corpus: agentflow-docs | 0.58153 | 1.01480 | 5.08 | 211.70 | 66.31 | 13097 / 1826 |

本次资源复测覆盖以上四组；首轮仍保留全部 39 组资源测量。其余 35 组没有被表述为本次重新完成资源采样。

## CPU 采样与后续优化方向

三个工作负载分别用外部采样器采样 5 秒、间隔 1 ms，渲染循环为 7 秒。原始系统报告留在忽略目录，公开文件仅有函数聚合与原始文件哈希。帧计数包含子调用与递归，不能相加成互斥 CPU 百分比。

- 500 节点 mesh 的主要热点仍在 layered/compound 布局及边路径生成；后续优化优先检查布局中的邻接遍历和中间分配，并以原始布局断言及输出哈希约束改动。
- agentflow 文档组同时呈现 parser statement、lexer 与 compound 布局热点，适合进一步检查词法/解析分配；本次没有根据少量耗时差异修改算法或削弱校验。
- 本轮新增测试调用记录只在 Rust 测试构建中启用；发布库不保留 spy 日志。独立内存计数器也不链接到发布库。

## 数据与复现

- [39 组完整配对结果](research/profiling-2026-10-19-test-expansion/comparison.json)
- [资源原始数值](research/profiling-2026-10-19-test-expansion/resources.json) · [CPU 函数聚合](research/profiling-2026-10-19-test-expansion/cpu.json)
- [源码、驱动和二进制 SHA-256](research/profiling-2026-10-19-test-expansion/metadata.json) · [摘要数值](research/profiling-2026-10-19-test-expansion/summary.json)
- 同目录保存优化前/后的三轮压缩原始 benchmark 报告。

需要原先固定输入 `target/bench/cases.json`、冻结基线 `target/profiling/before/bench` 及固定 Pele 开发依赖。复测期间不并行运行编译、测试或其他 benchmark。

```sh
cargo build --release -p zebra-mermaid --example bench --example profile
cargo build --release -p zebra-mermaid-profiler
python3 tools/bench/compare-optimization.py
python3 tools/bench/profile-resources.py \
  'docs: all flowchart examples' 'tree: 50 nodes' \
  'mesh: 500 nodes, 200 extra labelled edges' 'corpus: agentflow-docs'
python3 tools/bench/profile-cpu.py
# 使用新的目录名；已发布快照不覆盖。
python3 tools/bench/publish-checkpoint.py <new-snapshot-directory>
```

可通过 `CARGO_TARGET_DIR` 指定构建目录。CPU 栈采样脚本当前要求系统提供 `sample` 命令；进程资源测量与分配计数为独立步骤。报告不记录个人目录、主机名、硬件信息或系统版本。
