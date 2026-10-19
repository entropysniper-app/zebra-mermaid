# CPU、内存 profiling 与第一轮优化

后续 Class/Agentflow 热点、共享存储和保序删除优化见 [最新优化报告](PROFILING-CLASS-AGENTFLOW.zh-CN.md)。

测试扩展阶段的复测见 [测试扩展后的性能与资源复测](PROFILING-TEST-EXPANSION.zh-CN.md)。本页保留第一轮优化快照。

日期：2026-10-19。参照固定 Pele 提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`；优化前 Rust 基线为提交 `4761e84` 的生产实现。测试输入沿用 33 类、39 组扩展 benchmark，共 879 条输入。

本报告记录 SVG 优化快照：坐标格式化使用栈缓冲区、直接追加 SVG、减少根节点包装的临时字符串，以及 flowchart/swimlane 借用图节点而不再复制整个节点。后续 class 底层解析器的前导换行兼容修复单独通过测试验收，不属于这里的性能采样快照。输入、优化文件和二进制 SHA-256 见 [测量元信息](research/profiling-2026-10-19/metadata.json)。

## 口径

- **渲染耗时**：使用原有 `examples/bench.rs`，优化前后分别三轮独立进程，交替先后顺序。预热 5 批、校准至少约 10 ms、每轮至少 9 样本且累计至少 300 ms；取每轮中位数，再取三轮中位数。成功渲染与预期拒绝分别计时。
- **CPU 热点**：外部采样器对 50 节点树、500 节点 mesh、agentflow 文档各采样 5 秒，间隔 1 ms。保存去除机器信息的函数聚合。调用栈计数包含子调用与递归，不能相加当成互斥 CPU 百分比。
- **进程 CPU / RSS**：39 组各为每引擎运行三个新进程，预热 5 批后循环至少 1 秒。OS CPU 时间与峰值 RSS 包含启动、模块、输入加载、预热和测量；表中的 CPU/批次以循环批次数归一化，不能替代上面的纯渲染 benchmark。
- **Rust 分配**：独立 `zebra-mermaid-profiler` 包包装系统分配器；核心库继续禁止 unsafe。只记录预热后的一个完整批次，输入和选项在记录前创建，每次结果立即释放。累计申请量包含重新分配的新尺寸；额外存活峰值是分配器请求字节相对批次开始的增量，不是 RSS。
- agentflow 参考组包含 180 条成功输入与 59 条拒绝；资源 profiling 覆盖完整输入批次，原有耗时 benchmark 仍将二者分开。
- 所有计时采集均与编译和测试错开。原始系统采样输出只保存在忽略的 `target/` 下，公开数据不含个人路径、主机名或硬件配置。

## 验证与结果

优化前后的六份 benchmark 报告 **39/39 组验证记录相同**，包含 SVG 哈希、尺寸及错误字段。完整 SVG 差分和主题矩阵另外由工作区回归验证。39 组分配测量结束时的净存活字节均为 0；这表示该批次未新增保留分配，不能替代通用内存泄漏检测。

33 类文档工作负载等权 render 耗时比几何平均为 **0.944**，即本次测量平均降低 **5.6%**。该指标不代表任意实际应用的总体加速。

| 工作负载 | 优化前 render ms/批 | 优化后 render ms/批 | 耗时下降 | 累计申请量下降 | 额外存活峰值下降 |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.53289 | 1.45189 | 5.28% | 16.29% | 16.11% |
| tree: 50 nodes | 0.23704 | 0.21691 | 8.49% | 14.68% | 10.01% |
| mesh: 500 nodes, 200 extra labelled edges | 5.91581 | 5.61108 | 5.15% | 7.74% | 5.57% |
| corpus: agentflow-docs | 0.48963 | 0.47619 | 2.74% | 1.54% | 1.05% |

以上四组有优化前的独立分配基线；其他组只报告优化后的资源实测，不推断其内存改善比例。RSS 受系统分配器保留与进程运行时影响，不保证与请求字节同步下降。

## 全部 39 组渲染对比

| case | render 前 ms/批 | render 后 ms/批 | 后/前 | parse 前 ms/批 | parse 后 ms/批 |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.53289 | 1.45189 | 0.947 | 0.37891 | 0.37907 |
| tree: 50 nodes | 0.23704 | 0.21691 | 0.915 | 0.05863 | 0.05836 |
| mesh: 50 nodes, 20 extra labelled edges | 0.46206 | 0.43139 | 0.934 | 0.06546 | 0.06471 |
| tree: 500 nodes | 2.39902 | 2.18749 | 0.912 | 0.58664 | 0.58683 |
| mesh: 500 nodes, 200 extra labelled edges | 5.91581 | 5.61108 | 0.948 | 0.64874 | 0.64542 |
| corpus: agentflow-docs | 0.48963 | 0.47619 | 0.973 | 0.24394 | 0.24647 |
| corpus: agentflow-reference | 3.09819 | 2.98963 | 0.965 | 1.44554 | 1.42706 |
| corpus: architecture-docs | 0.12951 | 0.12400 | 0.957 | 0.03175 | 0.03193 |
| corpus: block-docs | 0.28596 | 0.26997 | 0.944 | 0.07831 | 0.07866 |
| corpus: c4-docs | 0.50682 | 0.48730 | 0.961 | 0.06938 | 0.06906 |
| corpus: class-docs | 0.74048 | 0.70377 | 0.950 | 0.16738 | 0.16720 |
| corpus: cynefin-docs | 0.11264 | 0.10280 | 0.913 | 0.00731 | 0.00735 |
| corpus: er-docs | 0.70553 | 0.66765 | 0.946 | 0.12690 | 0.12708 |
| corpus: eventmodeling-docs | 0.23846 | 0.22204 | 0.931 | 0.04373 | 0.04422 |
| corpus: flowchart-docs | 1.54947 | 1.45867 | 0.941 | 0.38233 | 0.37710 |
| corpus: gantt-docs | 0.30162 | 0.29064 | 0.964 | 0.03841 | 0.03921 |
| corpus: git-docs | 1.15952 | 1.06903 | 0.922 | 0.29424 | 0.29226 |
| corpus: info-docs | 0.00380 | 0.00356 | 0.936 | 0.00143 | 0.00147 |
| corpus: ishikawa-docs | 0.01934 | 0.01716 | 0.887 | 0.00270 | 0.00261 |
| corpus: journey-docs | 0.01995 | 0.01907 | 0.956 | 0.00224 | 0.00221 |
| corpus: kanban-docs | 0.06279 | 0.05946 | 0.947 | 0.01161 | 0.01150 |
| corpus: mindmap-docs | 0.08814 | 0.08183 | 0.928 | 0.01576 | 0.01573 |
| corpus: packet-docs | 0.06346 | 0.05904 | 0.930 | 0.01513 | 0.01517 |
| corpus: pie-docs | 0.02325 | 0.02193 | 0.943 | 0.00661 | 0.00655 |
| corpus: quadrant-docs | 0.05261 | 0.05029 | 0.956 | 0.01771 | 0.01786 |
| corpus: radar-docs | 0.05283 | 0.05050 | 0.956 | 0.01830 | 0.01833 |
| corpus: railroad-docs | 0.35765 | 0.33757 | 0.944 | 0.06751 | 0.06778 |
| corpus: requirement-docs | 0.29499 | 0.28611 | 0.970 | 0.04698 | 0.04746 |
| corpus: sankey-docs | 0.24766 | 0.22353 | 0.903 | 0.03496 | 0.03504 |
| corpus: sequence-docs | 0.66153 | 0.61409 | 0.928 | 0.16631 | 0.16412 |
| corpus: state-docs | 0.64632 | 0.62440 | 0.966 | 0.18783 | 0.18883 |
| corpus: swimlane-docs | 0.52947 | 0.48286 | 0.912 | 0.12273 | 0.12322 |
| corpus: timeline-docs | 0.32770 | 0.30679 | 0.936 | 0.04559 | 0.04606 |
| corpus: treemap-docs | 0.13711 | 0.12977 | 0.946 | 0.03193 | 0.03143 |
| corpus: treeview-docs | 0.10955 | 0.10724 | 0.979 | 0.04450 | 0.04397 |
| corpus: usecase-docs | 0.78555 | 0.77238 | 0.983 | 0.26089 | 0.26519 |
| corpus: venn-docs | 0.21657 | 0.21414 | 0.989 | 0.01656 | 0.01666 |
| corpus: wardley-docs | 0.28247 | 0.26518 | 0.939 | 0.07763 | 0.07739 |
| corpus: xychart-docs | 0.17071 | 0.15755 | 0.923 | 0.03344 | 0.03370 |

## 进程资源与 Rust 渲染分配

RSS 为三个独立进程峰值的中位数。Node 一侧包含运行时、JIT 与 GC 的成本，Rust 一侧包含整个原生进程；不能据此声称渲染算法本身节省相同比例内存。

| case | Rust CPU ms/批 | Pele CPU ms/批 | Rust RSS MiB | Pele RSS MiB | Rust 额外存活峰值 KiB | Rust alloc / realloc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 1.49761 | 3.42820 | 5.22 | 150.95 | 49.62 | 40226 / 4964 |
| tree: 50 nodes | 0.21602 | 0.31758 | 5.12 | 152.52 | 254.59 | 4946 / 863 |
| mesh: 50 nodes, 20 extra labelled edges | 0.42774 | 1.00238 | 5.34 | 155.41 | 392.36 | 8938 / 1657 |
| tree: 500 nodes | 2.21937 | 5.19880 | 8.23 | 222.67 | 2690.44 | 47454 / 7412 |
| mesh: 500 nodes, 200 extra labelled edges | 5.83863 | 19.00084 | 10.36 | 227.34 | 3941.80 | 71365 / 18005 |
| corpus: agentflow-docs | 0.48794 | 0.80583 | 5.11 | 218.81 | 66.31 | 13097 / 1826 |
| corpus: agentflow-reference | 3.54867 | 8.03685 | 5.41 | 232.81 | 83.78 | 94648 / 12339 |
| corpus: architecture-docs | 0.12750 | 0.25768 | 4.75 | 92.81 | 56.06 | 2147 / 516 |
| corpus: block-docs | 0.28129 | 0.30520 | 4.59 | 95.78 | 42.32 | 5447 / 1267 |
| corpus: c4-docs | 0.50631 | 0.60246 | 4.77 | 118.94 | 123.51 | 8292 / 2341 |
| corpus: class-docs | 0.72548 | 1.38761 | 5.12 | 145.02 | 77.71 | 16016 / 2539 |
| corpus: cynefin-docs | 0.10732 | 0.12058 | 4.39 | 107.75 | 48.84 | 1190 / 755 |
| corpus: er-docs | 0.69942 | 0.97991 | 5.05 | 139.73 | 54.60 | 15047 / 3257 |
| corpus: eventmodeling-docs | 0.23321 | 0.29641 | 4.62 | 85.64 | 35.53 | 3565 / 984 |
| corpus: flowchart-docs | 1.49768 | 3.57591 | 5.23 | 148.61 | 49.62 | 40226 / 4964 |
| corpus: gantt-docs | 0.30011 | 0.44139 | 4.72 | 96.23 | 47.49 | 5517 / 1120 |
| corpus: git-docs | 1.17858 | 1.69971 | 4.94 | 116.58 | 77.00 | 18969 / 5064 |
| corpus: info-docs | 0.00375 | 0.00266 | 3.66 | 69.47 | 1.77 | 104 / 9 |
| corpus: ishikawa-docs | 0.01779 | 0.01999 | 4.12 | 79.50 | 21.95 | 267 / 99 |
| corpus: journey-docs | 0.01954 | 0.01851 | 4.28 | 84.33 | 29.34 | 359 / 70 |
| corpus: kanban-docs | 0.06209 | 0.06387 | 4.44 | 82.58 | 49.44 | 1193 / 274 |
| corpus: mindmap-docs | 0.08778 | 0.10165 | 4.50 | 91.84 | 36.83 | 1621 / 430 |
| corpus: packet-docs | 0.06176 | 0.06954 | 4.39 | 80.30 | 30.70 | 1159 / 141 |
| corpus: pie-docs | 0.02278 | 0.02833 | 4.17 | 82.34 | 11.96 | 449 / 96 |
| corpus: quadrant-docs | 0.05213 | 0.06647 | 4.38 | 92.50 | 19.82 | 806 / 286 |
| corpus: radar-docs | 0.05155 | 0.09026 | 4.36 | 83.95 | 13.07 | 891 / 302 |
| corpus: railroad-docs | 0.34813 | 0.40229 | 4.73 | 82.69 | 39.49 | 4922 / 2548 |
| corpus: requirement-docs | 0.29438 | 0.38556 | 4.92 | 110.78 | 98.52 | 6042 / 1180 |
| corpus: sankey-docs | 0.23573 | 0.42182 | 4.66 | 139.11 | 132.43 | 3093 / 466 |
| corpus: sequence-docs | 0.65827 | 0.85586 | 4.95 | 125.75 | 43.26 | 10352 / 2971 |
| corpus: state-docs | 0.65325 | 1.36037 | 5.03 | 172.00 | 58.71 | 15524 / 2922 |
| corpus: swimlane-docs | 0.49525 | 0.82600 | 4.86 | 130.81 | 49.82 | 10896 / 2066 |
| corpus: timeline-docs | 0.32712 | 0.36649 | 4.55 | 88.62 | 34.46 | 5446 / 1561 |
| corpus: treemap-docs | 0.13425 | 0.17954 | 4.50 | 82.28 | 16.64 | 2128 / 588 |
| corpus: treeview-docs | 0.11221 | 0.17956 | 4.42 | 78.72 | 16.92 | 1965 / 544 |
| corpus: usecase-docs | 0.79307 | 1.45962 | 5.12 | 150.44 | 82.78 | 19398 / 3191 |
| corpus: venn-docs | 0.22115 | 0.61711 | 4.42 | 89.75 | 16.77 | 3282 / 192 |
| corpus: wardley-docs | 0.28017 | 0.42673 | 4.75 | 112.20 | 41.71 | 5213 / 1366 |
| corpus: xychart-docs | 0.16128 | 0.18434 | 4.41 | 93.53 | 27.91 | 3515 / 1023 |

## 热点与下一步

首轮调用栈中，mesh 主要消耗在 layered/compound 布局，tree 和 agentflow 同时暴露字符串格式化、复制及分配开销。本轮先减少这些可证明不改变图形语义的工作；后续可继续审查布局临时数组、解析器 token 存储和 agentflow 模型构造。下一轮必须保留当前基线，不能用减少测试、输入或迭代次数换取更快数字。

[优化前 CPU 聚合](research/profiling-2026-10-19/cpu-before.json)、[优化后 CPU 聚合](research/profiling-2026-10-19/cpu-after.json)、[耗时对照及三轮样本](research/profiling-2026-10-19/optimization-comparison.json)、[优化前资源基线](research/profiling-2026-10-19/resources-before.json)、[优化后 39 组资源测量](research/profiling-2026-10-19/resources-after.json) 已保存。六份原始耗时报告以 gzip 保存在同目录。

## 复现

```sh
node tools/bench/cases.mjs
cargo build -p zebra-mermaid --example bench --all-features --release
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build -p zebra-mermaid --example profile --all-features --release
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build -p zebra-mermaid-profiler --release

# 在修改生产实现前保存基线可执行文件；需要先创建目标目录。
mkdir -p target/profiling/before
cp target/release/examples/bench target/profiling/before/bench
cp target/release/examples/profile target/profiling/before/profile
cp target/release/zebra-mermaid-profiler target/profiling/before/zebra-mermaid-profiler

python3 tools/bench/profile-resources.py
cp target/profiling/resources.json target/profiling/before/resources.json
python3 tools/bench/profile-cpu.py

# 完成修改后重新执行上面的构建命令，再进行前后对照。
python3 tools/bench/compare-optimization.py
python3 tools/bench/profile-resources.py
python3 tools/bench/profile-cpu.py
```

可通过 `CARGO_TARGET_DIR` 指定构建目录。CPU 调用栈脚本当前使用系统 `sample` 工具；资源脚本支持提供 `getrusage` 的 macOS/Linux，按平台约定统一 RSS 单位。资源与分配测量均不需要更改发布构建的分配器。
