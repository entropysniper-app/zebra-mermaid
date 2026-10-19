# zebra-mermaid

纯 Rust 的 Mermaid → SVG 渲染库与命令行工具，移植自 [obsidianmd/pele](https://github.com/obsidianmd/pele)（固定提交 `44da152b`，package 1.6.0，Mermaid 语法目标 12.1.0）。运行时不依赖 JavaScript、浏览器或外部布局程序。

> **当前状态：默认支持全部 33 类图表；全量原始测试 1:1 移植仍在进行。** 当前 2,765/6,810 项通过（40.60%），5 项跳过、4,040 项未映射。39/39 组基准输入行为一致；本轮已提交输出样本 13,194 条一致，另 6 条孤立代理项未执行。70,183 条采集全集结果属于此前快照，本轮未重跑。实际支持范围、已知限制与逐项测试进度见 [实施状态](docs/IMPLEMENTATION-STATUS.md)，不能以差分通过代替原始测试验收。

[完整性能对比 Markdown](docs/BENCHMARK-COMPLETE.zh-CN.md) 汇总当前实现的 33 类/39 组渲染与解析、Pele 原协议、四引擎共同样本、CPU/RSS、Rust 分配、进程启动与分发体积，并附原始数据、源码指纹及串行复现入口。标签 `v0.1.0-benchmark.20261020` 表示性能快照，完整原始测试验收尚未完成。

<!-- sample-comparison:start -->
## Sample 渲染对比

使用固定 Pele 版本的 Playground `sampleGroups()` 原函数，完整选择 **33 类、167 个 Sample**，同一份源码分别由 Rust、Pele 和 Mermaid.js 渲染。

| 引擎 | 版本 | 成功 / Sample 总数 |
| --- | --- | ---: |
| Rust / zebra-mermaid | 0.1.0 | **167 / 167** |
| Pele | 1.6.0 | **167 / 167** |
| Mermaid.js | 12.1.0，真实 Chromium 渲染 | **167 / 167** |

Rust 与 Pele 的 **167/167 份 SVG 字节及尺寸完全一致**。三者采用 Pele Playground 的默认深色显示环境：Rust/Pele 复用其 Flexoki CSS、系统字体栈、6px 圆角和 siteIcon；Mermaid 按 Playground 使用 `theme: dark`。Rust/Pele 共用真实浏览器宿主字体测量，Rust 仍独立执行解析、布局和 SVG 生成。Mermaid 保留自身布局，因此外观和尺寸可以不同。此结果只描述 Sample 渲染，不能代替全量原始测试 1:1 验收。

- [完整 167 个 Sample 的三列可视化对照表](docs/render-comparison/index.html)：支持类型/源码筛选、图形放大、查看源码及打开 SVG。下载后直接打开 `index.html`；GitHub 网页通常显示 HTML 源码。
- [33 类汇总表与复现说明](docs/render-comparison/summary.md) · [结果、尺寸与 SHA-256](docs/render-comparison/results.json) · [Playground 样式变量与来源](docs/render-comparison/playground-style.json)。

下表展示每类在上游 Sample 顺序中的**第一个样例**，共 33 行。PNG 预览直接从对应 SVG 截取，点击图片打开 SVG；全部样例的源码和原始输出都保留。缩略图各自适配单元格，图形的实际尺寸见完整对照页。

| 类型 / Sample 源码 | Rust | Pele | Mermaid.js |
| --- | --- | --- | --- |
| **flowchart**<br>[flowchart-01](docs/render-comparison/sources/flowchart-01.mmd) | [<img src="docs/render-comparison/previews/flowchart-01.rust.png" width="240" alt="flowchart-01 rust">](docs/render-comparison/svg/flowchart-01.rust.svg) | [<img src="docs/render-comparison/previews/flowchart-01.pele.png" width="240" alt="flowchart-01 pele">](docs/render-comparison/svg/flowchart-01.pele.svg) | [<img src="docs/render-comparison/previews/flowchart-01.mermaid.png" width="240" alt="flowchart-01 mermaid">](docs/render-comparison/svg/flowchart-01.mermaid.svg) |
| **block**<br>[block-01](docs/render-comparison/sources/block-01.mmd) | [<img src="docs/render-comparison/previews/block-01.rust.png" width="240" alt="block-01 rust">](docs/render-comparison/svg/block-01.rust.svg) | [<img src="docs/render-comparison/previews/block-01.pele.png" width="240" alt="block-01 pele">](docs/render-comparison/svg/block-01.pele.svg) | [<img src="docs/render-comparison/previews/block-01.mermaid.png" width="240" alt="block-01 mermaid">](docs/render-comparison/svg/block-01.mermaid.svg) |
| **c4**<br>[c4-01](docs/render-comparison/sources/c4-01.mmd) | [<img src="docs/render-comparison/previews/c4-01.rust.png" width="240" alt="c4-01 rust">](docs/render-comparison/svg/c4-01.rust.svg) | [<img src="docs/render-comparison/previews/c4-01.pele.png" width="240" alt="c4-01 pele">](docs/render-comparison/svg/c4-01.pele.svg) | [<img src="docs/render-comparison/previews/c4-01.mermaid.png" width="240" alt="c4-01 mermaid">](docs/render-comparison/svg/c4-01.mermaid.svg) |
| **class**<br>[class-01](docs/render-comparison/sources/class-01.mmd) | [<img src="docs/render-comparison/previews/class-01.rust.png" width="240" alt="class-01 rust">](docs/render-comparison/svg/class-01.rust.svg) | [<img src="docs/render-comparison/previews/class-01.pele.png" width="240" alt="class-01 pele">](docs/render-comparison/svg/class-01.pele.svg) | [<img src="docs/render-comparison/previews/class-01.mermaid.png" width="240" alt="class-01 mermaid">](docs/render-comparison/svg/class-01.mermaid.svg) |
| **er**<br>[er-01](docs/render-comparison/sources/er-01.mmd) | [<img src="docs/render-comparison/previews/er-01.rust.png" width="240" alt="er-01 rust">](docs/render-comparison/svg/er-01.rust.svg) | [<img src="docs/render-comparison/previews/er-01.pele.png" width="240" alt="er-01 pele">](docs/render-comparison/svg/er-01.pele.svg) | [<img src="docs/render-comparison/previews/er-01.mermaid.png" width="240" alt="er-01 mermaid">](docs/render-comparison/svg/er-01.mermaid.svg) |
| **eventmodeling**<br>[eventmodeling-01](docs/render-comparison/sources/eventmodeling-01.mmd) | [<img src="docs/render-comparison/previews/eventmodeling-01.rust.png" width="240" alt="eventmodeling-01 rust">](docs/render-comparison/svg/eventmodeling-01.rust.svg) | [<img src="docs/render-comparison/previews/eventmodeling-01.pele.png" width="240" alt="eventmodeling-01 pele">](docs/render-comparison/svg/eventmodeling-01.pele.svg) | [<img src="docs/render-comparison/previews/eventmodeling-01.mermaid.png" width="240" alt="eventmodeling-01 mermaid">](docs/render-comparison/svg/eventmodeling-01.mermaid.svg) |
| **gantt**<br>[gantt-01](docs/render-comparison/sources/gantt-01.mmd) | [<img src="docs/render-comparison/previews/gantt-01.rust.png" width="240" alt="gantt-01 rust">](docs/render-comparison/svg/gantt-01.rust.svg) | [<img src="docs/render-comparison/previews/gantt-01.pele.png" width="240" alt="gantt-01 pele">](docs/render-comparison/svg/gantt-01.pele.svg) | [<img src="docs/render-comparison/previews/gantt-01.mermaid.png" width="240" alt="gantt-01 mermaid">](docs/render-comparison/svg/gantt-01.mermaid.svg) |
| **git**<br>[git-01](docs/render-comparison/sources/git-01.mmd) | [<img src="docs/render-comparison/previews/git-01.rust.png" width="240" alt="git-01 rust">](docs/render-comparison/svg/git-01.rust.svg) | [<img src="docs/render-comparison/previews/git-01.pele.png" width="240" alt="git-01 pele">](docs/render-comparison/svg/git-01.pele.svg) | [<img src="docs/render-comparison/previews/git-01.mermaid.png" width="240" alt="git-01 mermaid">](docs/render-comparison/svg/git-01.mermaid.svg) |
| **info**<br>[info-01](docs/render-comparison/sources/info-01.mmd) | [<img src="docs/render-comparison/previews/info-01.rust.png" width="240" alt="info-01 rust">](docs/render-comparison/svg/info-01.rust.svg) | [<img src="docs/render-comparison/previews/info-01.pele.png" width="240" alt="info-01 pele">](docs/render-comparison/svg/info-01.pele.svg) | [<img src="docs/render-comparison/previews/info-01.mermaid.png" width="240" alt="info-01 mermaid">](docs/render-comparison/svg/info-01.mermaid.svg) |
| **journey**<br>[journey-01](docs/render-comparison/sources/journey-01.mmd) | [<img src="docs/render-comparison/previews/journey-01.rust.png" width="240" alt="journey-01 rust">](docs/render-comparison/svg/journey-01.rust.svg) | [<img src="docs/render-comparison/previews/journey-01.pele.png" width="240" alt="journey-01 pele">](docs/render-comparison/svg/journey-01.pele.svg) | [<img src="docs/render-comparison/previews/journey-01.mermaid.png" width="240" alt="journey-01 mermaid">](docs/render-comparison/svg/journey-01.mermaid.svg) |
| **kanban**<br>[kanban-01](docs/render-comparison/sources/kanban-01.mmd) | [<img src="docs/render-comparison/previews/kanban-01.rust.png" width="240" alt="kanban-01 rust">](docs/render-comparison/svg/kanban-01.rust.svg) | [<img src="docs/render-comparison/previews/kanban-01.pele.png" width="240" alt="kanban-01 pele">](docs/render-comparison/svg/kanban-01.pele.svg) | [<img src="docs/render-comparison/previews/kanban-01.mermaid.png" width="240" alt="kanban-01 mermaid">](docs/render-comparison/svg/kanban-01.mermaid.svg) |
| **mindmap**<br>[mindmap-01](docs/render-comparison/sources/mindmap-01.mmd) | [<img src="docs/render-comparison/previews/mindmap-01.rust.png" width="240" alt="mindmap-01 rust">](docs/render-comparison/svg/mindmap-01.rust.svg) | [<img src="docs/render-comparison/previews/mindmap-01.pele.png" width="240" alt="mindmap-01 pele">](docs/render-comparison/svg/mindmap-01.pele.svg) | [<img src="docs/render-comparison/previews/mindmap-01.mermaid.png" width="240" alt="mindmap-01 mermaid">](docs/render-comparison/svg/mindmap-01.mermaid.svg) |
| **packet**<br>[packet-01](docs/render-comparison/sources/packet-01.mmd) | [<img src="docs/render-comparison/previews/packet-01.rust.png" width="240" alt="packet-01 rust">](docs/render-comparison/svg/packet-01.rust.svg) | [<img src="docs/render-comparison/previews/packet-01.pele.png" width="240" alt="packet-01 pele">](docs/render-comparison/svg/packet-01.pele.svg) | [<img src="docs/render-comparison/previews/packet-01.mermaid.png" width="240" alt="packet-01 mermaid">](docs/render-comparison/svg/packet-01.mermaid.svg) |
| **pie**<br>[pie-01](docs/render-comparison/sources/pie-01.mmd) | [<img src="docs/render-comparison/previews/pie-01.rust.png" width="240" alt="pie-01 rust">](docs/render-comparison/svg/pie-01.rust.svg) | [<img src="docs/render-comparison/previews/pie-01.pele.png" width="240" alt="pie-01 pele">](docs/render-comparison/svg/pie-01.pele.svg) | [<img src="docs/render-comparison/previews/pie-01.mermaid.png" width="240" alt="pie-01 mermaid">](docs/render-comparison/svg/pie-01.mermaid.svg) |
| **quadrant**<br>[quadrant-01](docs/render-comparison/sources/quadrant-01.mmd) | [<img src="docs/render-comparison/previews/quadrant-01.rust.png" width="240" alt="quadrant-01 rust">](docs/render-comparison/svg/quadrant-01.rust.svg) | [<img src="docs/render-comparison/previews/quadrant-01.pele.png" width="240" alt="quadrant-01 pele">](docs/render-comparison/svg/quadrant-01.pele.svg) | [<img src="docs/render-comparison/previews/quadrant-01.mermaid.png" width="240" alt="quadrant-01 mermaid">](docs/render-comparison/svg/quadrant-01.mermaid.svg) |
| **requirement**<br>[requirement-01](docs/render-comparison/sources/requirement-01.mmd) | [<img src="docs/render-comparison/previews/requirement-01.rust.png" width="240" alt="requirement-01 rust">](docs/render-comparison/svg/requirement-01.rust.svg) | [<img src="docs/render-comparison/previews/requirement-01.pele.png" width="240" alt="requirement-01 pele">](docs/render-comparison/svg/requirement-01.pele.svg) | [<img src="docs/render-comparison/previews/requirement-01.mermaid.png" width="240" alt="requirement-01 mermaid">](docs/render-comparison/svg/requirement-01.mermaid.svg) |
| **sankey**<br>[sankey-01](docs/render-comparison/sources/sankey-01.mmd) | [<img src="docs/render-comparison/previews/sankey-01.rust.png" width="240" alt="sankey-01 rust">](docs/render-comparison/svg/sankey-01.rust.svg) | [<img src="docs/render-comparison/previews/sankey-01.pele.png" width="240" alt="sankey-01 pele">](docs/render-comparison/svg/sankey-01.pele.svg) | [<img src="docs/render-comparison/previews/sankey-01.mermaid.png" width="240" alt="sankey-01 mermaid">](docs/render-comparison/svg/sankey-01.mermaid.svg) |
| **sequence**<br>[sequence-01](docs/render-comparison/sources/sequence-01.mmd) | [<img src="docs/render-comparison/previews/sequence-01.rust.png" width="240" alt="sequence-01 rust">](docs/render-comparison/svg/sequence-01.rust.svg) | [<img src="docs/render-comparison/previews/sequence-01.pele.png" width="240" alt="sequence-01 pele">](docs/render-comparison/svg/sequence-01.pele.svg) | [<img src="docs/render-comparison/previews/sequence-01.mermaid.png" width="240" alt="sequence-01 mermaid">](docs/render-comparison/svg/sequence-01.mermaid.svg) |
| **state**<br>[state-01](docs/render-comparison/sources/state-01.mmd) | [<img src="docs/render-comparison/previews/state-01.rust.png" width="240" alt="state-01 rust">](docs/render-comparison/svg/state-01.rust.svg) | [<img src="docs/render-comparison/previews/state-01.pele.png" width="240" alt="state-01 pele">](docs/render-comparison/svg/state-01.pele.svg) | [<img src="docs/render-comparison/previews/state-01.mermaid.png" width="240" alt="state-01 mermaid">](docs/render-comparison/svg/state-01.mermaid.svg) |
| **timeline**<br>[timeline-01](docs/render-comparison/sources/timeline-01.mmd) | [<img src="docs/render-comparison/previews/timeline-01.rust.png" width="240" alt="timeline-01 rust">](docs/render-comparison/svg/timeline-01.rust.svg) | [<img src="docs/render-comparison/previews/timeline-01.pele.png" width="240" alt="timeline-01 pele">](docs/render-comparison/svg/timeline-01.pele.svg) | [<img src="docs/render-comparison/previews/timeline-01.mermaid.png" width="240" alt="timeline-01 mermaid">](docs/render-comparison/svg/timeline-01.mermaid.svg) |
| **xychart**<br>[xychart-01](docs/render-comparison/sources/xychart-01.mmd) | [<img src="docs/render-comparison/previews/xychart-01.rust.png" width="240" alt="xychart-01 rust">](docs/render-comparison/svg/xychart-01.rust.svg) | [<img src="docs/render-comparison/previews/xychart-01.pele.png" width="240" alt="xychart-01 pele">](docs/render-comparison/svg/xychart-01.pele.svg) | [<img src="docs/render-comparison/previews/xychart-01.mermaid.png" width="240" alt="xychart-01 mermaid">](docs/render-comparison/svg/xychart-01.mermaid.svg) |
| **agentflow**<br>[agentflow-01](docs/render-comparison/sources/agentflow-01.mmd) | [<img src="docs/render-comparison/previews/agentflow-01.rust.png" width="240" alt="agentflow-01 rust">](docs/render-comparison/svg/agentflow-01.rust.svg) | [<img src="docs/render-comparison/previews/agentflow-01.pele.png" width="240" alt="agentflow-01 pele">](docs/render-comparison/svg/agentflow-01.pele.svg) | [<img src="docs/render-comparison/previews/agentflow-01.mermaid.png" width="240" alt="agentflow-01 mermaid">](docs/render-comparison/svg/agentflow-01.mermaid.svg) |
| **architecture**<br>[architecture-01](docs/render-comparison/sources/architecture-01.mmd) | [<img src="docs/render-comparison/previews/architecture-01.rust.png" width="240" alt="architecture-01 rust">](docs/render-comparison/svg/architecture-01.rust.svg) | [<img src="docs/render-comparison/previews/architecture-01.pele.png" width="240" alt="architecture-01 pele">](docs/render-comparison/svg/architecture-01.pele.svg) | [<img src="docs/render-comparison/previews/architecture-01.mermaid.png" width="240" alt="architecture-01 mermaid">](docs/render-comparison/svg/architecture-01.mermaid.svg) |
| **cynefin**<br>[cynefin-01](docs/render-comparison/sources/cynefin-01.mmd) | [<img src="docs/render-comparison/previews/cynefin-01.rust.png" width="240" alt="cynefin-01 rust">](docs/render-comparison/svg/cynefin-01.rust.svg) | [<img src="docs/render-comparison/previews/cynefin-01.pele.png" width="240" alt="cynefin-01 pele">](docs/render-comparison/svg/cynefin-01.pele.svg) | [<img src="docs/render-comparison/previews/cynefin-01.mermaid.png" width="240" alt="cynefin-01 mermaid">](docs/render-comparison/svg/cynefin-01.mermaid.svg) |
| **ishikawa**<br>[ishikawa-01](docs/render-comparison/sources/ishikawa-01.mmd) | [<img src="docs/render-comparison/previews/ishikawa-01.rust.png" width="240" alt="ishikawa-01 rust">](docs/render-comparison/svg/ishikawa-01.rust.svg) | [<img src="docs/render-comparison/previews/ishikawa-01.pele.png" width="240" alt="ishikawa-01 pele">](docs/render-comparison/svg/ishikawa-01.pele.svg) | [<img src="docs/render-comparison/previews/ishikawa-01.mermaid.png" width="240" alt="ishikawa-01 mermaid">](docs/render-comparison/svg/ishikawa-01.mermaid.svg) |
| **radar**<br>[radar-01](docs/render-comparison/sources/radar-01.mmd) | [<img src="docs/render-comparison/previews/radar-01.rust.png" width="240" alt="radar-01 rust">](docs/render-comparison/svg/radar-01.rust.svg) | [<img src="docs/render-comparison/previews/radar-01.pele.png" width="240" alt="radar-01 pele">](docs/render-comparison/svg/radar-01.pele.svg) | [<img src="docs/render-comparison/previews/radar-01.mermaid.png" width="240" alt="radar-01 mermaid">](docs/render-comparison/svg/radar-01.mermaid.svg) |
| **railroad**<br>[railroad-01](docs/render-comparison/sources/railroad-01.mmd) | [<img src="docs/render-comparison/previews/railroad-01.rust.png" width="240" alt="railroad-01 rust">](docs/render-comparison/svg/railroad-01.rust.svg) | [<img src="docs/render-comparison/previews/railroad-01.pele.png" width="240" alt="railroad-01 pele">](docs/render-comparison/svg/railroad-01.pele.svg) | [<img src="docs/render-comparison/previews/railroad-01.mermaid.png" width="240" alt="railroad-01 mermaid">](docs/render-comparison/svg/railroad-01.mermaid.svg) |
| **swimlane**<br>[swimlane-01](docs/render-comparison/sources/swimlane-01.mmd) | [<img src="docs/render-comparison/previews/swimlane-01.rust.png" width="240" alt="swimlane-01 rust">](docs/render-comparison/svg/swimlane-01.rust.svg) | [<img src="docs/render-comparison/previews/swimlane-01.pele.png" width="240" alt="swimlane-01 pele">](docs/render-comparison/svg/swimlane-01.pele.svg) | [<img src="docs/render-comparison/previews/swimlane-01.mermaid.png" width="240" alt="swimlane-01 mermaid">](docs/render-comparison/svg/swimlane-01.mermaid.svg) |
| **treemap**<br>[treemap-01](docs/render-comparison/sources/treemap-01.mmd) | [<img src="docs/render-comparison/previews/treemap-01.rust.png" width="240" alt="treemap-01 rust">](docs/render-comparison/svg/treemap-01.rust.svg) | [<img src="docs/render-comparison/previews/treemap-01.pele.png" width="240" alt="treemap-01 pele">](docs/render-comparison/svg/treemap-01.pele.svg) | [<img src="docs/render-comparison/previews/treemap-01.mermaid.png" width="240" alt="treemap-01 mermaid">](docs/render-comparison/svg/treemap-01.mermaid.svg) |
| **treeview**<br>[treeview-01](docs/render-comparison/sources/treeview-01.mmd) | [<img src="docs/render-comparison/previews/treeview-01.rust.png" width="240" alt="treeview-01 rust">](docs/render-comparison/svg/treeview-01.rust.svg) | [<img src="docs/render-comparison/previews/treeview-01.pele.png" width="240" alt="treeview-01 pele">](docs/render-comparison/svg/treeview-01.pele.svg) | [<img src="docs/render-comparison/previews/treeview-01.mermaid.png" width="240" alt="treeview-01 mermaid">](docs/render-comparison/svg/treeview-01.mermaid.svg) |
| **usecase**<br>[usecase-01](docs/render-comparison/sources/usecase-01.mmd) | [<img src="docs/render-comparison/previews/usecase-01.rust.png" width="240" alt="usecase-01 rust">](docs/render-comparison/svg/usecase-01.rust.svg) | [<img src="docs/render-comparison/previews/usecase-01.pele.png" width="240" alt="usecase-01 pele">](docs/render-comparison/svg/usecase-01.pele.svg) | [<img src="docs/render-comparison/previews/usecase-01.mermaid.png" width="240" alt="usecase-01 mermaid">](docs/render-comparison/svg/usecase-01.mermaid.svg) |
| **venn**<br>[venn-01](docs/render-comparison/sources/venn-01.mmd) | [<img src="docs/render-comparison/previews/venn-01.rust.png" width="240" alt="venn-01 rust">](docs/render-comparison/svg/venn-01.rust.svg) | [<img src="docs/render-comparison/previews/venn-01.pele.png" width="240" alt="venn-01 pele">](docs/render-comparison/svg/venn-01.pele.svg) | [<img src="docs/render-comparison/previews/venn-01.mermaid.png" width="240" alt="venn-01 mermaid">](docs/render-comparison/svg/venn-01.mermaid.svg) |
| **wardley**<br>[wardley-01](docs/render-comparison/sources/wardley-01.mmd) | [<img src="docs/render-comparison/previews/wardley-01.rust.png" width="240" alt="wardley-01 rust">](docs/render-comparison/svg/wardley-01.rust.svg) | [<img src="docs/render-comparison/previews/wardley-01.pele.png" width="240" alt="wardley-01 pele">](docs/render-comparison/svg/wardley-01.pele.svg) | [<img src="docs/render-comparison/previews/wardley-01.mermaid.png" width="240" alt="wardley-01 mermaid">](docs/render-comparison/svg/wardley-01.mermaid.svg) |

样式直接来自上游 `base.scss`、`pele-theme.scss` 与 `playground.scss`；背景为 `#1c1b1a`，节点表面为 `#282726`，正文为 `#cecdc3`。固定画布宽 760px、扣除内边距后的布局宽 712px。时钟固定为 2026-10-19 00:00 UTC；保留样例自身的 front matter。Mermaid 有 17 份返回字符串含 HTML 标签，先按其支持的 DOM 插入方式呈现，再序列化为独立 SVG，原始字符串另存为 `.mermaid.raw.svg`。没有修改源码来强求三者视觉一致。
<!-- sample-comparison:end -->

## 历史 CPU 与内存优化

此前对 Agentflow 的 global 节点处理增加所属 flow 索引，保留成员顺序与 Pele 输出。以下保留该轮优化实验，当前实现的全部资源复测见 [完整性能报告](docs/BENCHMARK-COMPLETE.zh-CN.md)。与当时上一轮 Rust 实现相比：

| 测量范围 | 结果 |
| --- | --- |
| 3,000 节点后接 3,000 个 global 块，release 完整渲染 | 52.85 → 9.57 ms，耗时下降 81.9%，约 5.52 倍速度 |
| 33 类文档 render，三轮交替配对 | 等权几何平均耗时低 1.44%；常规 Agentflow 文档组 +0.34%，主要收益在压力路径 |
| 输出与行为一致性 | 39/39 组一致；目标压力 SVG 的 985,192 字节与 Pele 完全一致 |
| 持续内存检查 | 1,408,680 个输入处理轮次，每轮结束后净保留 0 字节、0 分配块 |
| 独立系统泄漏检查 | render、模型复用、缓存、C ABI 四条路径均为 0 泄漏；受控泄漏成功检出 |

内存轮次包含相同输入的重复处理，不代表新增测试项。计数器、缓存所有权检查、系统泄漏工具与 RSS 分开验证；结论为已测场景内未发现泄漏，不代替所有输入及原生 App 的验收。owner 索引在目标压力场景增加约 28 KiB 峰值。详细方法、常规慢项、RSS 波动、原始数据与复现命令见 [CPU 优化与内存验证](docs/PROFILING-CPU-MEMORY.zh-CN.md)，按 Pele 原格式的五组表见 [原脚本基准](docs/BENCHMARK-PELE-PROTOCOL.zh-CN.md)。

<!-- four-engine-benchmark:start -->
## 四引擎性能对比

同一批 Pele Playground Sample，比较 Mermaid.js / Pele / zebra-mermaid / mermaid-rs-renderer。下面每行单位为共同输入整批的 ms，三轮独立进程中位数，启动与预热不计时。

| 图表场景 | 共同样本 / 原样本 | Mermaid.js ms | Pele ms | zebra-mermaid ms | mermaid-rs-renderer ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 / 6 | 94.80000 | 0.33297 | 0.18595 | 79.07788 |
| block | 6 / 6 | 84.00000 | 0.09768 | 0.08522 | 0.12396 |
| c4 | 6 / 6 | 105.80000 | 0.57638 | 0.38153 | 0.97150 |
| class | 5 / 6 | 127.90000 | 0.22591 | 0.13896 | 0.31429 |
| er | 6 / 6 | 89.00000 | 0.27581 | 0.16699 | 5.93300 |
| eventmodeling | 0 / 6 | — | — | — | — |
| gantt | 6 / 6 | 17.90000 | 0.28588 | 0.16441 | 0.34364 |
| git | 6 / 6 | 109.20000 | 0.44657 | 0.24813 | 0.53246 |
| info | 0 / 2 | — | — | — | — |
| journey | 1 / 1 | 3.10000 | 0.01631 | 0.01551 | 0.03355 |
| kanban | 3 / 3 | 28.30000 | 0.05409 | 0.04538 | 0.10682 |
| mindmap | 6 / 6 | 98.40000 | 0.09709 | 0.06520 | 0.31331 |
| packet | 4 / 4 | 6.85000 | 0.06419 | 0.04185 | 2.14840 |
| pie | 3 / 3 | 5.10000 | 0.03441 | 0.02198 | 0.06293 |
| quadrant | 3 / 3 | 10.10000 | 0.05526 | 0.04110 | 0.07402 |
| requirement | 6 / 6 | 89.60000 | 0.24337 | 0.17294 | 1.62577 |
| sankey | 6 / 6 | 31.90000 | 0.31204 | 0.13431 | 0.36918 |
| sequence | 4 / 6 | 26.60000 | 0.09400 | 0.08092 | 0.64719 |
| state | 6 / 6 | 132.20000 | 0.34674 | 0.19541 | 4.81023 |
| timeline | 6 / 6 | 35.70000 | 0.20598 | 0.14031 | 0.32911 |
| xychart | 6 / 6 | 27.70000 | 0.11773 | 0.08771 | 0.18329 |
| agentflow | 0 / 6 | — | — | — | — |
| architecture | 6 / 6 | 80.40000 | 0.21959 | 0.10696 | 0.22420 |
| cynefin | 0 / 3 | — | — | — | — |
| ishikawa | 0 / 1 | — | — | — | — |
| radar | 3 / 3 | 7.25000 | 0.07342 | 0.03497 | 0.07750 |
| railroad | 0 / 6 | — | — | — | — |
| swimlane | 0 / 6 | — | — | — | — |
| treemap | 6 / 6 | 18.30000 | 0.14599 | 0.08642 | 0.21741 |
| treeview | 0 / 6 | — | — | — | — |
| usecase | 0 / 6 | — | — | — | — |
| venn | 0 / 6 | — | — | — | — |
| wardley | 0 / 6 | — | — | — | — |

四者共同通过类型与 SVG 检查的样本为 **110/167**；mmdr 对其他图表的 Flowchart 误识别单独排除，完整覆盖率、失败原因、测量限制与源码版本见 [四引擎报告](docs/BENCHMARK-FOUR-ENGINES.zh-CN.md)。SVG 返回成功不等于四者视觉或语义完全一致。
<!-- four-engine-benchmark:end -->

## 文档

- [实施状态（进度、实际测试结果、已知差异）](docs/IMPLEMENTATION-STATUS.md)
- [Rust 重写技术方案与实施路径](docs/RUST-REWRITE-PLAN.zh-CN.md)
- [Zebra 现有主题接入兼容契约](docs/ZEBRA-THEME-COMPATIBILITY.md)
- [测试计划](docs/TEST-PLAN.md)
- [Pele 全量测试 1:1 移植与验收](docs/UPSTREAM-TEST-PARITY.md)
- [完整性能对比（当前性能快照、全部表格与原始证据）](docs/BENCHMARK-COMPLETE.zh-CN.md)
- [按 Pele 原脚本格式与口径的性能对比](docs/BENCHMARK-PELE-PROTOCOL.zh-CN.md)
- [33 类、39 组扩展性能与一致性报告](docs/BENCHMARK.zh-CN.md)
- [CPU/内存 profiling 与优化前后对比](docs/PROFILING-AND-OPTIMIZATION.zh-CN.md)
- [测试扩展后的性能与资源复测](docs/PROFILING-TEST-EXPANSION.zh-CN.md)
- [Class 样式与 Agentflow 的 CPU/内存优化](docs/PROFILING-CLASS-AGENTFLOW.zh-CN.md)
- [Agentflow CPU 再优化与持续内存/泄漏验证](docs/PROFILING-CPU-MEMORY.zh-CN.md)
- [移植规范（Porting guide）](docs/PORTING-GUIDE.md)
- [第三方许可](THIRD_PARTY_NOTICES.md)

## 仓库结构

| 路径 | 内容 |
| --- | --- |
| `crates/zebra-mermaid` | 库：预处理、YAML、文本测量、布局、SVG 输出、各图表、Zebra 主题层 |
| `crates/zebra-mermaid-cli` | 命令行工具 `zebra-mermaid` |
| `crates/zebra-mermaid-ffi` | 与 Zebra 现有原生桥同形状的 C ABI v2（`include/zebra_mermaid.h`）；尚未接入 Zebra |
| `tools/compat` | 开发期 Node oracle（打包固定 Pele 源码）、fixture 生成、上游测试输入采集 |
| `tools/bench` | 与 Pele 的性能对照、CPU/RSS profiling 和优化前后比较脚本 |
| `tools/profiler` | 开发专用分配计数器；不链接到发布的渲染库 |
| `tests/fixtures` | 差分 fixture（Pele 语料、单元对照、上游测试采集抽样） |

研究快照 `.research/`（Pele 源码、Zebra 主题参考源码）已加入 `.gitignore`，不入库。

## 库用法

```rust
use zebra_mermaid::{render, ColorMode, MermaidPalette, MermaidThemeMode, RenderOptions};

// Pele 兼容输出：--pele-* CSS 变量，可由页面主题化。
let svg = render("pie\n  \"Dogs\" : 386\n  \"Cats\" : 85\n", &RenderOptions::default())?.svg;

// 原生具体颜色输出：可直接选择默认深色；自定义颜色由宿主现有主题提供。
let options = RenderOptions { color_mode: ColorMode::Resolved, theme: MermaidThemeMode::Dark, ..Default::default() };
let result = render(source, &options)?;
```

Zebra 现有调用形状（深色标志、调色板 JSON、字号、target_width、画布策略）见 `zebra_mermaid::zebra::{render_for_zebra, render_abi_v2}`；该适配不让几何随 target_width 变化。

无需另建 JSON 主题文件：Zebra 已会把当前主题自动转换并传给 Rust。网页可继续设置 Pele 的 `--pele-*` CSS 变量；当前 macOS `NSImage` 路径不能正确解析这些变量，因此原生输出先解析成具体颜色。`--palette` 文件参数仅作为复用现有协议的可选 CLI 输入。

## 命令行

```sh
cargo build --release -p zebra-mermaid-cli     # 生成 target/release/zebra-mermaid

zebra-mermaid render diagram.mmd -o diagram.svg
cat diagram.mmd | zebra-mermaid render - > diagram.svg
zebra-mermaid render diagram.mmd --theme dark \
  --color-mode resolved --canvas-background transparent -o diagram.svg
zebra-mermaid check diagram.mmd --format json
zebra-mermaid batch ./diagrams --out-dir ./svg --jobs 4
zebra-mermaid --help
```

退出码：0 成功、2 参数错误、3 不支持/语法/语义错误、4 资源限制、5 输入输出错误、1 内部错误。默认不覆盖已有输出文件（`--force` 允许），输出经同目录临时文件原子替换。

## 开发与验证

```sh
cargo test --workspace                      # 当前默认构建的单元、差分、主题、CLI、健壮性测试
node tools/test-parity/check.mjs --run       # 已映射原始测试子集；不代表完整验收
node tools/test-parity/check.mjs --require-complete # 完整门禁，当前仍应失败
ZM_FULL_HARVEST=1 cargo test -p zebra-mermaid --test compat_harvest   # 上游测试采集全集（需先生成，见下）

# 重新生成 fixture（需要 .research/pele 与其 node_modules）
node tools/compat/gen-fixtures.mjs
node tools/compat/harvest.mjs --fixtures
```

上述命令均从仓库根目录执行。
