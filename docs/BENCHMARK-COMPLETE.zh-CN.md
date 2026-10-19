# 完整性能对比：v0.1.0-benchmark.20261020

本报告针对当前提交的 Rust 实现重新测量，覆盖全部 33 类图表、39 组 Rust/Pele 工作负载、Pele 原脚本五组基准、四引擎共同样本，以及 39 组 CPU、RSS 和 Rust 分配统计。所有构建先完成，测量进程串行运行，没有并行编译或测试。

性能快照标签：`v0.1.0-benchmark.20261020`。扩展基准时间：2026-10-19T23:19:38.603Z；四引擎基准时间：2026-10-19T23:26:15.591561+00:00。日期和时钟记录使用 UTC；源码和二进制以 [SHA-256 清单](research/complete-benchmark-2026-10-19-codex-takeover/metadata.json) 标识，标签指向包含实现、测试与报告的同一提交。

## 1. 主要结果与验收边界

| 指标 | 本次实际结果 |
| --- | --- |
| Rust 图类型覆盖 | 33/33 |
| Rust/Pele 扩展输入行为一致性 | 39/39 组；按工作负载累计 820 次成功渲染、59 次预期拒绝 |
| 33 类文档完整渲染 Rust/Pele 比值几何平均 | 0.6280；小于 1 表示 Rust 耗时较低 |
| 33 类文档仅解析 Rust/Pele 比值几何平均 | 0.5198 |
| 完整渲染更快的文档图类型 | 33/33；其余慢项原样保留 |
| 四引擎共同成功样本 | 110/167 个 Sample，22 个图表组 |
| Pele/Rust Sample SVG 字节与尺寸完全一致 | 167/167 |
| 原始测试 1:1 验收 | 2765/6810 通过，5 跳过，4040 未映射，完整验收未通过 |
| 常规工作区回归 | 3163 通过、0 失败、5 跳过 |
| 已提交输出样本差分 | 13,194/13,194 一致；另 6 条孤立代理项未执行、不计通过 |

三个覆盖指标分别统计：图类型接通、SVG/错误对照通过、原始测试逐项通过互不替代。该标签是性能快照，不能解读为全部 6,810 项测试已移植完成。本轮没有重新生成或运行本地缺失的 70,183 条采集全集，历史全集结果不算本次验证。

几何平均对每种图表等权，仅概括这 33 类文档工作负载，不是任意应用的总加速倍数。39 组共有 879 次输入出现、753 份不同源码；流程图文档在原脚本组与 corpus 组中重复，不能相加当作独立测试覆盖。

## 2. 版本、测量方法与单位

- Pele 1.6.0，固定提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`；重新执行其原始构建脚本生成生产 bundle。
- zebra-mermaid 0.1.0；rustc 1.96.1 (31fca3adb 2026-06-26)；Node v26.3.0。Rust release、thin LTO、codegen-units=1，启用全部图类型。
- Mermaid.js 12.1.0、Chromium 151.0.7922.34、Playwright 1.62.1；mermaid-rs-renderer 0.3.1，固定提交 `3726ccbffe0e8032361eb9668694b24f77858060`。
- Rust/Pele 采用相同输入、CSS 输出、默认 Arial 字宽表，固定 now=1792368000000、UTC。完整 render 包含解析、布局和 SVG 生成，没有预解析或输出缓存命中。
- 扩展与四引擎基准每轮先预热 5 批，校准到计时片段至少约 10 ms，再采集至少 9 个样本且累计至少 300 ms。每轮取排序样本的下中位数，再取 3 个独立进程轮次的中位数；引擎顺序交替。三轮比值范围是观测范围，不是置信区间。
- 除资源表的特别说明外，单位为 **ms/整批输入**。同一行两侧输入完全相同；不同图类型的批次大小不同，不宜按绝对耗时排名。
- 比值一律为 **Rust/Pele**，小于 1 表示 Rust 更快；表中保留所有大于 1 的慢项。硬件、系统版本、主机名和个人路径未进入报告，结果不作跨平台性能保证。

## 3. Pele 原脚本五组基准

直接执行固定版本 `scripts/bench.mjs`；Rust 使用同一脚本求值获得的输入，并按原计时循环实现。一批预热，至少 300 ms 且至少 5 个样本，最多 2,000 个样本，偶数样本取上中位数。此表为原协议单轮结果，与后面的三轮扩展表分开。

| 工作负载 | 输入数 | Pele | zebra-mermaid |
| --- | ---: | ---: | ---: |
| docs: all examples | 114 | 2.5 ms | 1.1 ms |
| tree: 50 nodes | 1 | 0.245 ms | 0.159 ms |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.687 ms | 0.302 ms |
| tree: 500 nodes | 1 | 2.9 ms | 1.7 ms |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 9.9 ms | 5.0 ms |

118 个输入全部成功，计时外逐项比较完整 SVG 字符串与尺寸相等。第一行是全部流程图文档的整批耗时，其余每行一张；格式沿用上游显示精度。

## 4. 33 类、39 组完整渲染与解析

验证成功输入的 SVG UTF-8 FNV-1a-64 和尺寸；拒绝输入逐项比较错误码、消息、行、列。成功渲染与拒绝计时分离。完整字节对照另由 118 项原协议、167 项 Sample 和已提交样本回归提供。

### 完整渲染

| 工作负载 | 成功/总输入 | Pele ms | Rust ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114/114 | 2.77588 | 1.14345 | 0.412 | 0.375–0.491 |
| tree: 50 nodes | 1/1 | 0.25215 | 0.16344 | 0.648 | 0.579–0.668 |
| mesh: 50 nodes, 20 extra labelled edges | 1/1 | 0.71310 | 0.30476 | 0.427 | 0.356–0.431 |
| tree: 500 nodes | 1/1 | 3.03223 | 1.69246 | 0.558 | 0.521–0.568 |
| mesh: 500 nodes, 200 extra labelled edges | 1/1 | 9.73700 | 4.97833 | 0.511 | 0.507–0.519 |
| corpus: agentflow-docs | 12/12 | 0.53020 | 0.39420 | 0.743 | 0.597–0.747 |
| corpus: agentflow-reference | 180/239 | 4.03307 | 2.49248 | 0.618 | 0.536–0.639 |
| corpus: architecture-docs | 6/6 | 0.23249 | 0.10532 | 0.453 | 0.441–0.482 |
| corpus: block-docs | 30/30 | 0.26412 | 0.21555 | 0.816 | 0.751–0.894 |
| corpus: c4-docs | 6/6 | 0.62570 | 0.38056 | 0.608 | 0.602–0.635 |
| corpus: class-docs | 38/38 | 0.90777 | 0.58151 | 0.641 | 0.583–0.734 |
| corpus: cynefin-docs | 3/3 | 0.10472 | 0.07214 | 0.689 | 0.681–0.741 |
| corpus: er-docs | 25/25 | 0.70871 | 0.49287 | 0.695 | 0.680–0.710 |
| corpus: eventmodeling-docs | 12/12 | 0.26033 | 0.17275 | 0.664 | 0.628–0.678 |
| corpus: flowchart-docs | 114/114 | 2.27494 | 1.10973 | 0.488 | 0.363–0.533 |
| corpus: gantt-docs | 11/11 | 0.37968 | 0.23478 | 0.618 | 0.534–0.622 |
| corpus: git-docs | 35/35 | 1.40193 | 0.82710 | 0.590 | 0.581–0.602 |
| corpus: info-docs | 2/2 | 0.00243 | 0.00226 | 0.930 | 0.913–0.976 |
| corpus: ishikawa-docs | 1/1 | 0.01970 | 0.01380 | 0.701 | 0.681–0.714 |
| corpus: journey-docs | 1/1 | 0.01822 | 0.01529 | 0.839 | 0.766–0.927 |
| corpus: kanban-docs | 3/3 | 0.06772 | 0.04448 | 0.657 | 0.647–0.660 |
| corpus: mindmap-docs | 13/13 | 0.08871 | 0.06297 | 0.710 | 0.694–0.790 |
| corpus: packet-docs | 4/4 | 0.06470 | 0.04141 | 0.640 | 0.634–0.686 |
| corpus: pie-docs | 2/2 | 0.02617 | 0.01591 | 0.608 | 0.608–0.611 |
| corpus: quadrant-docs | 3/3 | 0.06138 | 0.04059 | 0.661 | 0.511–0.668 |
| corpus: radar-docs | 3/3 | 0.08497 | 0.03392 | 0.399 | 0.399–0.428 |
| corpus: railroad-docs | 14/14 | 0.35381 | 0.28472 | 0.805 | 0.805–0.845 |
| corpus: requirement-docs | 9/9 | 0.30004 | 0.22475 | 0.749 | 0.503–0.756 |
| corpus: sankey-docs | 8/8 | 0.33880 | 0.14304 | 0.422 | 0.410–0.464 |
| corpus: sequence-docs | 38/38 | 0.69990 | 0.53201 | 0.760 | 0.740–0.832 |
| corpus: state-docs | 22/22 | 0.98771 | 0.47991 | 0.486 | 0.474–0.525 |
| corpus: swimlane-docs | 13/13 | 0.59445 | 0.38768 | 0.652 | 0.642–0.715 |
| corpus: timeline-docs | 14/14 | 0.32457 | 0.25976 | 0.800 | 0.798–0.852 |
| corpus: treemap-docs | 8/8 | 0.19306 | 0.10797 | 0.559 | 0.454–0.564 |
| corpus: treeview-docs | 13/13 | 0.17430 | 0.08336 | 0.478 | 0.476–0.500 |
| corpus: usecase-docs | 24/24 | 1.09961 | 0.61150 | 0.556 | 0.556–0.595 |
| corpus: venn-docs | 8/8 | 0.48226 | 0.20350 | 0.422 | 0.351–0.450 |
| corpus: wardley-docs | 19/19 | 0.33705 | 0.22193 | 0.658 | 0.637–0.658 |
| corpus: xychart-docs | 8/8 | 0.15298 | 0.10228 | 0.669 | 0.659–0.670 |

### 仅解析

| 工作负载 | 有效输入 | Pele ms | Rust ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114 | 0.58644 | 0.31289 | 0.534 | 0.464–0.612 |
| tree: 50 nodes | 1 | 0.06099 | 0.05003 | 0.820 | 0.791–0.892 |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.06731 | 0.05425 | 0.806 | 0.770–0.844 |
| tree: 500 nodes | 1 | 0.62586 | 0.52408 | 0.837 | 0.788–0.908 |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 0.77368 | 0.57660 | 0.745 | 0.745–0.759 |
| corpus: agentflow-docs | 12 | 0.23271 | 0.21460 | 0.922 | 0.922–0.935 |
| corpus: agentflow-reference | 180 | 1.97879 | 1.21341 | 0.613 | 0.599–0.707 |
| corpus: architecture-docs | 6 | 0.12299 | 0.03023 | 0.246 | 0.243–0.268 |
| corpus: block-docs | 30 | 0.10635 | 0.06328 | 0.595 | 0.464–0.637 |
| corpus: c4-docs | 6 | 0.10426 | 0.06733 | 0.646 | 0.628–0.764 |
| corpus: class-docs | 38 | 0.19438 | 0.13396 | 0.689 | 0.665–0.714 |
| corpus: cynefin-docs | 3 | 0.01825 | 0.00682 | 0.374 | 0.364–0.381 |
| corpus: er-docs | 25 | 0.12784 | 0.10980 | 0.859 | 0.771–0.954 |
| corpus: eventmodeling-docs | 12 | 0.10157 | 0.03849 | 0.379 | 0.368–0.402 |
| corpus: flowchart-docs | 114 | 0.50629 | 0.30654 | 0.605 | 0.584–0.623 |
| corpus: gantt-docs | 11 | 0.06829 | 0.03310 | 0.485 | 0.469–0.505 |
| corpus: git-docs | 35 | 0.59614 | 0.24361 | 0.409 | 0.407–0.411 |
| corpus: info-docs | 2 | 0.00137 | 0.00085 | 0.617 | 0.609–0.659 |
| corpus: ishikawa-docs | 1 | 0.00352 | 0.00240 | 0.683 | 0.677–0.697 |
| corpus: journey-docs | 1 | 0.00308 | 0.00182 | 0.591 | 0.548–0.591 |
| corpus: kanban-docs | 3 | 0.01799 | 0.00938 | 0.521 | 0.463–0.538 |
| corpus: mindmap-docs | 13 | 0.02284 | 0.01205 | 0.527 | 0.527–0.546 |
| corpus: packet-docs | 4 | 0.02971 | 0.01165 | 0.392 | 0.391–0.418 |
| corpus: pie-docs | 2 | 0.01058 | 0.00472 | 0.446 | 0.412–0.453 |
| corpus: quadrant-docs | 3 | 0.03057 | 0.01592 | 0.521 | 0.323–0.531 |
| corpus: radar-docs | 3 | 0.04642 | 0.01519 | 0.327 | 0.282–0.329 |
| corpus: railroad-docs | 14 | 0.11872 | 0.05510 | 0.464 | 0.460–0.513 |
| corpus: requirement-docs | 9 | 0.06690 | 0.04309 | 0.644 | 0.520–0.685 |
| corpus: sankey-docs | 8 | 0.06117 | 0.03037 | 0.497 | 0.430–0.540 |
| corpus: sequence-docs | 38 | 0.20151 | 0.14306 | 0.710 | 0.644–0.867 |
| corpus: state-docs | 22 | 0.23260 | 0.15374 | 0.661 | 0.647–0.791 |
| corpus: swimlane-docs | 13 | 0.14093 | 0.10450 | 0.741 | 0.684–0.759 |
| corpus: timeline-docs | 14 | 0.06265 | 0.03507 | 0.560 | 0.546–0.566 |
| corpus: treemap-docs | 8 | 0.09900 | 0.02736 | 0.276 | 0.256–0.301 |
| corpus: treeview-docs | 13 | 0.10120 | 0.03554 | 0.351 | 0.345–0.364 |
| corpus: usecase-docs | 24 | 0.30725 | 0.22117 | 0.720 | 0.671–0.833 |
| corpus: venn-docs | 8 | 0.02783 | 0.01294 | 0.465 | 0.449–0.515 |
| corpus: wardley-docs | 19 | 0.14744 | 0.06973 | 0.473 | 0.458–0.506 |
| corpus: xychart-docs | 8 | 0.05103 | 0.02705 | 0.530 | 0.528–0.548 |

### 预期拒绝

| 工作负载 | 非法输入 | Pele ms | Rust ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| corpus: agentflow-reference | 59 | 0.77963 | 0.25374 | 0.325 |

## 5. 四引擎同输入横向对比

四个引擎各自尝试同一批 33 类、167 个 Pele Playground Sample，不改写源码迁就某个引擎。计时只使用四者共同成功的输入；无共同输入的类型仍列出。Mermaid 在真实浏览器中测量，启动、模块加载、验证、IPC 和写盘位于计时之外，DOM 清理计入调用；mmdr 的 options 克隆计入原生 API 调用。

请求 Arial 16；Pele/Rust 使用字宽表和 CSS 变量，Mermaid 使用浏览器字体度量，mmdr 使用默认原生字体度量、fast_text_metrics=false；后两者采用 dark 主题。四者布局、样式和语义实现不同，表格不表示输出视觉等价。第三方引擎仅属于独立开发驱动，不进入 Rust 生产依赖。

| 引擎 | 返回 SVG / 167 | 通过类型、XML 与尺寸检查 / 167 |
| --- | ---: | ---: |
| Mermaid.js | 167 | 167 |
| Pele | 167 | 167 |
| zebra-mermaid | 167 | 167 |
| mermaid-rs-renderer | 156 | 110 |

四者共同集合为 110 个输入、22 组。mmdr 将其他图表误识别为 Flowchart 的结果不计合格成功。Mermaid SVG 的 DOM 序列化只用于计时外验证；Info 无 viewBox 时使用实际 viewport。Pele/Rust 的 167 项 SVG 字节和尺寸相等，其他两者仅验证可渲染性与类型，完整语义一致性未获证明。

| 图类型 | 共同/原样本 | Mermaid.js ms | Pele ms | Rust ms | mmdr ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6/6 | 94.80000 | 0.33297 | 0.18595 | 79.07788 |
| block | 6/6 | 84.00000 | 0.09768 | 0.08522 | 0.12396 |
| c4 | 6/6 | 105.80000 | 0.57638 | 0.38153 | 0.97150 |
| class | 5/6 | 127.90000 | 0.22591 | 0.13896 | 0.31429 |
| er | 6/6 | 89.00000 | 0.27581 | 0.16699 | 5.93300 |
| eventmodeling | 0/6 | — | — | — | — |
| gantt | 6/6 | 17.90000 | 0.28588 | 0.16441 | 0.34364 |
| git | 6/6 | 109.20000 | 0.44657 | 0.24813 | 0.53246 |
| info | 0/2 | — | — | — | — |
| journey | 1/1 | 3.10000 | 0.01631 | 0.01551 | 0.03355 |
| kanban | 3/3 | 28.30000 | 0.05409 | 0.04538 | 0.10682 |
| mindmap | 6/6 | 98.40000 | 0.09709 | 0.06520 | 0.31331 |
| packet | 4/4 | 6.85000 | 0.06419 | 0.04185 | 2.14840 |
| pie | 3/3 | 5.10000 | 0.03441 | 0.02198 | 0.06293 |
| quadrant | 3/3 | 10.10000 | 0.05526 | 0.04110 | 0.07402 |
| requirement | 6/6 | 89.60000 | 0.24337 | 0.17294 | 1.62577 |
| sankey | 6/6 | 31.90000 | 0.31204 | 0.13431 | 0.36918 |
| sequence | 4/6 | 26.60000 | 0.09400 | 0.08092 | 0.64719 |
| state | 6/6 | 132.20000 | 0.34674 | 0.19541 | 4.81023 |
| timeline | 6/6 | 35.70000 | 0.20598 | 0.14031 | 0.32911 |
| xychart | 6/6 | 27.70000 | 0.11773 | 0.08771 | 0.18329 |
| agentflow | 0/6 | — | — | — | — |
| architecture | 6/6 | 80.40000 | 0.21959 | 0.10696 | 0.22420 |
| cynefin | 0/3 | — | — | — | — |
| ishikawa | 0/1 | — | — | — | — |
| radar | 3/3 | 7.25000 | 0.07342 | 0.03497 | 0.07750 |
| railroad | 0/6 | — | — | — | — |
| swimlane | 0/6 | — | — | — | — |
| treemap | 6/6 | 18.30000 | 0.14599 | 0.08642 | 0.21741 |
| treeview | 0/6 | — | — | — | — |
| usecase | 0/6 | — | — | — | — |
| venn | 0/6 | — | — | — | — |
| wardley | 0/6 | — | — | — | — |

### 全部输入覆盖

| 图类型 | 总数 | Mermaid.js 合格 | Pele 合格 | Rust 合格 | mmdr 合格 |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 6 | 6 | 6 | 6 |
| block | 6 | 6 | 6 | 6 | 6 |
| c4 | 6 | 6 | 6 | 6 | 6 |
| class | 6 | 6 | 6 | 6 | 5 |
| er | 6 | 6 | 6 | 6 | 6 |
| eventmodeling | 6 | 6 | 6 | 6 | 0 |
| gantt | 6 | 6 | 6 | 6 | 6 |
| git | 6 | 6 | 6 | 6 | 6 |
| info | 2 | 2 | 2 | 2 | 0 |
| journey | 1 | 1 | 1 | 1 | 1 |
| kanban | 3 | 3 | 3 | 3 | 3 |
| mindmap | 6 | 6 | 6 | 6 | 6 |
| packet | 4 | 4 | 4 | 4 | 4 |
| pie | 3 | 3 | 3 | 3 | 3 |
| quadrant | 3 | 3 | 3 | 3 | 3 |
| requirement | 6 | 6 | 6 | 6 | 6 |
| sankey | 6 | 6 | 6 | 6 | 6 |
| sequence | 6 | 6 | 6 | 6 | 4 |
| state | 6 | 6 | 6 | 6 | 6 |
| timeline | 6 | 6 | 6 | 6 | 6 |
| xychart | 6 | 6 | 6 | 6 | 6 |
| agentflow | 6 | 6 | 6 | 6 | 0 |
| architecture | 6 | 6 | 6 | 6 | 6 |
| cynefin | 3 | 3 | 3 | 3 | 0 |
| ishikawa | 1 | 1 | 1 | 1 | 0 |
| radar | 3 | 3 | 3 | 3 | 3 |
| railroad | 6 | 6 | 6 | 6 | 0 |
| swimlane | 6 | 6 | 6 | 6 | 0 |
| treemap | 6 | 6 | 6 | 6 | 6 |
| treeview | 6 | 6 | 6 | 6 | 0 |
| usecase | 6 | 6 | 6 | 6 | 0 |
| venn | 6 | 6 | 6 | 6 | 0 |
| wardley | 6 | 6 | 6 | 6 | 0 |

## 6. 39 组 CPU 与进程峰值 RSS

这是独立资源实验，不与纯渲染计时混用。每组每引擎 3 个新进程，交替顺序，预热 5 批后运行至少 1 秒；每批包括完整 render、成功/拒绝计数及 SVG 字节统计。Agentflow reference 在此处理全部合法及非法输入，渲染与拒绝的纯耗时仍见第 4 节。

CPU 取子进程 user+system 时间，包含启动、模块/文件读取与预热；表中 CPU ms/批以该时间除以计时循环批数，含上述摊销，不是纯 render CPU 时间。RSS 是整个进程的系统记录峰值，包括运行时、代码和已加载数据，不能解读为单图申请量。每列分别取 3 轮中位数；MiB=1,048,576 bytes。

| 工作负载 | Pele CPU ms/批 | Rust CPU ms/批 | Pele 峰值 RSS MiB | Rust 峰值 RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 5.61814 | 1.16854 | 141.59 | 5.70 |
| tree: 50 nodes | 0.39137 | 0.16156 | 151.50 | 5.38 |
| mesh: 50 nodes, 20 extra labelled edges | 1.22074 | 0.32137 | 151.22 | 5.80 |
| tree: 500 nodes | 8.33550 | 1.74358 | 230.20 | 8.84 |
| mesh: 500 nodes, 200 extra labelled edges | 21.90680 | 5.27650 | 223.42 | 12.38 |
| corpus: agentflow-docs | 0.93091 | 0.39771 | 212.86 | 5.66 |
| corpus: agentflow-reference | 9.43726 | 2.87635 | 232.59 | 5.83 |
| corpus: architecture-docs | 0.27975 | 0.10778 | 92.86 | 5.16 |
| corpus: block-docs | 0.33929 | 0.21967 | 95.36 | 5.12 |
| corpus: c4-docs | 0.66606 | 0.38416 | 118.86 | 5.50 |
| corpus: class-docs | 1.56308 | 0.58260 | 126.44 | 5.77 |
| corpus: cynefin-docs | 0.13370 | 0.07313 | 107.98 | 5.08 |
| corpus: er-docs | 1.12167 | 0.50871 | 124.17 | 5.47 |
| corpus: eventmodeling-docs | 0.36569 | 0.18498 | 85.86 | 5.09 |
| corpus: flowchart-docs | 4.24315 | 1.13415 | 147.44 | 5.72 |
| corpus: gantt-docs | 0.55833 | 0.23736 | 96.39 | 5.22 |
| corpus: git-docs | 1.85780 | 0.86029 | 101.77 | 5.42 |
| corpus: info-docs | 0.00282 | 0.00239 | 69.41 | 4.08 |
| corpus: ishikawa-docs | 0.02395 | 0.01480 | 79.59 | 4.42 |
| corpus: journey-docs | 0.02061 | 0.01664 | 84.75 | 4.69 |
| corpus: kanban-docs | 0.07893 | 0.04584 | 83.27 | 4.78 |
| corpus: mindmap-docs | 0.11014 | 0.06752 | 92.38 | 5.09 |
| corpus: packet-docs | 0.08811 | 0.04321 | 81.11 | 4.70 |
| corpus: pie-docs | 0.02981 | 0.01655 | 82.72 | 4.62 |
| corpus: quadrant-docs | 0.07781 | 0.04489 | 92.73 | 4.94 |
| corpus: radar-docs | 0.11404 | 0.03750 | 85.09 | 4.94 |
| corpus: railroad-docs | 0.45803 | 0.28923 | 82.64 | 4.89 |
| corpus: requirement-docs | 0.43196 | 0.24500 | 109.03 | 5.45 |
| corpus: sankey-docs | 0.56056 | 0.14465 | 106.39 | 5.03 |
| corpus: sequence-docs | 0.98015 | 0.60875 | 125.25 | 5.64 |
| corpus: state-docs | 1.88638 | 0.54349 | 147.41 | 5.78 |
| corpus: swimlane-docs | 1.25868 | 0.44896 | 113.78 | 5.52 |
| corpus: timeline-docs | 0.46318 | 0.26679 | 89.56 | 5.17 |
| corpus: treemap-docs | 0.19006 | 0.11441 | 82.11 | 4.83 |
| corpus: treeview-docs | 0.22353 | 0.08660 | 78.61 | 5.12 |
| corpus: usecase-docs | 1.73180 | 0.62929 | 143.84 | 5.91 |
| corpus: venn-docs | 0.66230 | 0.23005 | 89.83 | 4.83 |
| corpus: wardley-docs | 0.45751 | 0.24054 | 112.00 | 5.30 |
| corpus: xychart-docs | 0.21179 | 0.10743 | 93.64 | 4.78 |

## 7. Rust 分配与峰值存活量

使用单独的带分配计数器可执行文件，避免影响正常耗时。5 批预热后观测一整批；输入/options 不计入，输出逐次释放。累计申请量含 realloc 的完整新请求，不是实际 RSS；峰值为相对批次开始的新增存活请求字节。没有同口径的 JS 分配计数，因此不计算 Rust/JS 分配倍率。

| 工作负载 | alloc 次数 | realloc 次数 | 累计申请 KiB | 新增存活峰值 KiB | 批后净保留 bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 34949 | 3309 | 2637.06 | 45.73 | 0 |
| tree: 50 nodes | 4356 | 419 | 561.17 | 232.84 | 0 |
| mesh: 50 nodes, 20 extra labelled edges | 5311 | 872 | 863.53 | 326.66 | 0 |
| tree: 500 nodes | 41762 | 3188 | 5846.72 | 2448.94 | 0 |
| mesh: 500 nodes, 200 extra labelled edges | 55303 | 9815 | 12952.90 | 3985.68 | 0 |
| corpus: agentflow-docs | 11832 | 1577 | 1161.13 | 65.45 | 0 |
| corpus: agentflow-reference | 84866 | 11976 | 7754.84 | 79.25 | 0 |
| corpus: architecture-docs | 1573 | 441 | 413.28 | 65.06 | 0 |
| corpus: block-docs | 4030 | 1012 | 850.38 | 41.66 | 0 |
| corpus: c4-docs | 5573 | 2029 | 1222.47 | 110.46 | 0 |
| corpus: class-docs | 12131 | 1969 | 1742.41 | 69.88 | 0 |
| corpus: cynefin-docs | 668 | 508 | 272.33 | 42.37 | 0 |
| corpus: er-docs | 11704 | 1735 | 1379.66 | 52.50 | 0 |
| corpus: eventmodeling-docs | 1967 | 827 | 513.88 | 28.70 | 0 |
| corpus: flowchart-docs | 34949 | 3309 | 2637.06 | 45.73 | 0 |
| corpus: gantt-docs | 3604 | 929 | 419.62 | 45.46 | 0 |
| corpus: git-docs | 12869 | 4066 | 2547.71 | 63.59 | 0 |
| corpus: info-docs | 64 | 11 | 6.61 | 1.55 | 0 |
| corpus: ishikawa-docs | 161 | 76 | 58.80 | 21.66 | 0 |
| corpus: journey-docs | 188 | 68 | 54.35 | 23.92 | 0 |
| corpus: kanban-docs | 618 | 274 | 135.17 | 42.32 | 0 |
| corpus: mindmap-docs | 1106 | 462 | 233.20 | 36.05 | 0 |
| corpus: packet-docs | 501 | 126 | 99.91 | 24.44 | 0 |
| corpus: pie-docs | 254 | 97 | 41.01 | 11.94 | 0 |
| corpus: quadrant-docs | 570 | 261 | 106.65 | 16.95 | 0 |
| corpus: radar-docs | 460 | 197 | 92.44 | 13.31 | 0 |
| corpus: railroad-docs | 3708 | 2084 | 938.22 | 38.67 | 0 |
| corpus: requirement-docs | 4786 | 1069 | 795.58 | 111.18 | 0 |
| corpus: sankey-docs | 1589 | 276 | 387.54 | 117.34 | 0 |
| corpus: sequence-docs | 7230 | 2967 | 1724.54 | 36.80 | 0 |
| corpus: state-docs | 13194 | 1979 | 1651.08 | 58.61 | 0 |
| corpus: swimlane-docs | 9631 | 1462 | 977.28 | 45.79 | 0 |
| corpus: timeline-docs | 3574 | 1563 | 771.39 | 28.80 | 0 |
| corpus: treemap-docs | 1350 | 578 | 242.63 | 15.34 | 0 |
| corpus: treeview-docs | 1093 | 605 | 269.58 | 16.71 | 0 |
| corpus: usecase-docs | 17255 | 2543 | 1906.08 | 78.20 | 0 |
| corpus: venn-docs | 1881 | 245 | 172.95 | 17.57 | 0 |
| corpus: wardley-docs | 3395 | 1634 | 674.05 | 36.81 | 0 |
| corpus: xychart-docs | 2030 | 654 | 308.51 | 29.28 | 0 |

39 组批次结束后净保留均为 0 字节。此结果仅覆盖这些批次，不是长期运行或所有输入的无泄漏证明；此前持续内存/系统泄漏实验保留在独立历史报告，本次没有将其计为重新执行。

## 8. 进程启动与分发体积

同一张四节点流程图，预运行后分别采样 30 次，交替进程启动顺序，完整 SVG 相等。墙钟含进程创建、模块加载、标准输入输出和一次渲染，文件系统缓存已热；Node 参照脚本不是 Pele 官方 CLI，不能将倍率视作渲染算法加速。

| 路径 | 中位 ms | 最小–最大 ms |
| --- | ---: | ---: |
| Rust CLI | 2.667 | 1.952–6.702 |
| Node + Pele 参照脚本 | 80.409 | 75.583–129.442 |

| 分发产物 | 原始 bytes | gzip bytes |
| --- | ---: | ---: |
| Rust 全图类型 CLI（未单独 strip） | 4955152 | 2288901 |
| Pele minified JS bundle | 678032 | 251692 |

原生可执行文件与 JS bundle 的运行时依赖和分发形式不同，不据此推断部署体积优势。Rust 生产运行时不依赖 JS、Node、浏览器或外部布局程序；这些仅用于开发验证。

## 9. 可复现入口与原始证据

准备固定 Pele checkout 及其锁定依赖、固定 mmdr checkout、Rust 工具链、Node，以及与本报告版本一致的 Playwright/Chromium。浏览器模块可通过 `PLAYWRIGHT_MODULE` 指定，浏览器安装目录可通过 `PLAYWRIGHT_BROWSERS_PATH` 指定；机器路径不写入仓库。

```sh
git clone https://github.com/obsidianmd/pele.git .research/pele
git -C .research/pele checkout 44da152bbb79b91f03e68016f42499bb25bf6e82
npm ci --prefix .research/pele
pnpm --dir .research/pele/website install --frozen-lockfile
git clone https://github.com/1jehuang/mermaid-rs-renderer.git .research/mermaid-rs-renderer
git -C .research/mermaid-rs-renderer checkout 3726ccbffe0e8032361eb9668694b24f77858060
npm install --prefix target/bench-browser playwright@1.62.1
target/bench-browser/node_modules/.bin/playwright install chromium
export PLAYWRIGHT_MODULE="$PWD/target/bench-browser/node_modules/playwright"
# 入口在计时全部结束后自动执行原始测试门禁、工作区与输出样本回归。
BENCHMARK_TAG=v0.1.0-benchmark.20261020 BENCH_REPORT_SUFFIX=-reproduction python3 tools/bench/run-complete.py
```

入口依次构建、运行原脚本基准、运行三轮扩展基准、采集全部资源组、运行四引擎验证与计时，再执行测试验收和回归，最后生成本文及分项 Markdown。`BENCH_REPORT_SUFFIX` 选择新的证据目录，发布器拒绝覆盖已有四引擎/完整快照。逐项测试验收使用 `node tools/test-parity/check.mjs --require-complete`，当前因未全量移植而退出 1；入口仅在失败和待验证均为 0、确有未映射或跳过项时接受这个预期的非完整状态，仍在报告中明确未通过完整验收。常规回归使用 `cargo test --workspace --all-features -- --test-threads=1`。

- [完整汇总](research/complete-benchmark-2026-10-19-codex-takeover/summary.json)、[版本/源码/驱动/二进制指纹](research/complete-benchmark-2026-10-19-codex-takeover/metadata.json)、[测试与输出对照统计](research/complete-benchmark-2026-10-19-codex-takeover/validation.json)。
- [39 组资源原始三轮数据及分配计数](research/complete-benchmark-2026-10-19-codex-takeover/resources.json)。
- [33 类扩展计时与逐输入验证](research/benchmark-2026-10-19-33-types-codex-takeover/comparison.json)，同目录保存输入及六份压缩原始采样报告。
- [Pele 原协议数据](research/upstream-benchmark-2026-10-19-33-types-codex-takeover/comparison.json)，同目录保存原控制台表、输入及完整 SVG 验证结果。
- [四引擎汇总](research/four-engine-benchmark-2026-10-19-codex-takeover/comparison.json)、[全部成功/失败及排除原因](research/four-engine-benchmark-2026-10-19-codex-takeover/verification.json)、[实际共同输入](research/four-engine-benchmark-2026-10-19-codex-takeover/common.json)，同目录保存十二份压缩计时记录及 SVG。

分项阅读：[Pele 原协议](BENCHMARK-PELE-PROTOCOL.zh-CN.md)、[33 类扩展报告](BENCHMARK.zh-CN.md)、[四引擎报告](BENCHMARK-FOUR-ENGINES.zh-CN.md)、[实施状态](IMPLEMENTATION-STATUS.md)、[原始测试逐项验收](UPSTREAM-TEST-PARITY.md)。
