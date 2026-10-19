# 全图类型性能与一致性对比

测量时间：2026-10-19T09:09:27.588Z。Pele 固定提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`。Rust 实际构建启用 **33/33 类**。

本报告为扩展基准；按 Pele 仓库原始格式与口径的主表见 [原脚本基准](BENCHMARK-PELE-PROTOCOL.zh-CN.md)。

## 一致性覆盖

**39/39 组的全部输入行为一致**，共 879 个输入，其中 820 个成功渲染、59 个预期拒绝。每次成功比较 SVG 的 UTF-8 FNV-1a-64 与尺寸；每次拒绝比较错误码、消息、行、列。完整逐字节比较另由语料/采集回归承担。

Agentflow 参考组同时包含合法与非法输入，因此分成两种计时：有效的 180 条参与 render 与 parse；非法的 59 条只参与独立的 rejection 计时。没有把错误处理当成成功渲染，也没有删除这些验证项。

## 测量条件

- 硬件与操作系统信息未纳入公开报告；结果仅描述本次测量，不作为跨平台结论。
- rustc 1.96.1 (31fca3adb 2026-06-26)；Node v26.3.0。Rust release、thin LTO、codegen-units=1；Pele 生产 bundle。
- 相同输入、CSS 输出、默认 Arial 估算测量器、UTC、固定 now=1792368000000；parse 与完整 render 分开计时，不用预解析或缓存命中替代完整渲染。
- 三轮独立进程，交替引擎先后顺序；预热 5 个批次，校准重复次数使计时片段至少约 10 ms。每轮至少 9 个样本且累计至少 300 ms，取每轮中位数，再取三轮中位数。
- 计时期间没有并行编译或测试。验证和报告写出在计时之外。
- 下表单位是 **ms/该行成功输入的整批**。Rust/Pele 小于 1 为 Rust 更快。三轮比值范围不等同于置信区间。

33 类文档工作负载的 render 比值几何平均为 **0.953**（每类等权，不能代表任意应用的总体速度）。

## 成功渲染

| case | 成功输入/总输入 | 一致性 | pele ms | zebra-mermaid ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114/114 | 通过 | 1.93812 | 1.50740 | 0.778 | 0.766–0.788 |
| tree: 50 nodes | 1/1 | 通过 | 0.22958 | 0.23173 | 1.009 | 0.951–1.034 |
| mesh: 50 nodes, 20 extra labelled edges | 1/1 | 通过 | 0.64965 | 0.46158 | 0.711 | 0.707–0.742 |
| tree: 500 nodes | 1/1 | 通过 | 2.69864 | 2.35286 | 0.872 | 0.849–0.902 |
| mesh: 500 nodes, 200 extra labelled edges | 1/1 | 通过 | 8.77767 | 5.92131 | 0.675 | 0.668–0.700 |
| corpus: agentflow-docs | 12/12 | 通过 | 0.46236 | 0.48230 | 1.043 | 1.026–1.044 |
| corpus: agentflow-reference | 180/239 | 通过 | 3.35674 | 3.04784 | 0.908 | 0.858–0.917 |
| corpus: architecture-docs | 6/6 | 通过 | 0.20562 | 0.12709 | 0.618 | 0.618–0.648 |
| corpus: block-docs | 30/30 | 通过 | 0.23975 | 0.28215 | 1.177 | 1.173–1.198 |
| corpus: c4-docs | 6/6 | 通过 | 0.55165 | 0.51104 | 0.926 | 0.914–0.926 |
| corpus: class-docs | 38/38 | 通过 | 0.80394 | 0.73484 | 0.914 | 0.890–0.936 |
| corpus: cynefin-docs | 3/3 | 通过 | 0.09394 | 0.11077 | 1.179 | 1.179–1.195 |
| corpus: er-docs | 25/25 | 通过 | 0.61896 | 0.70508 | 1.139 | 1.130–1.142 |
| corpus: eventmodeling-docs | 12/12 | 通过 | 0.23509 | 0.23487 | 0.999 | 0.999–1.011 |
| corpus: flowchart-docs | 114/114 | 通过 | 1.86427 | 1.52586 | 0.818 | 0.797–0.830 |
| corpus: gantt-docs | 11/11 | 通过 | 0.35693 | 0.29449 | 0.825 | 0.799–0.845 |
| corpus: git-docs | 35/35 | 通过 | 1.27962 | 1.11904 | 0.875 | 0.865–0.880 |
| corpus: info-docs | 2/2 | 通过 | 0.00219 | 0.00375 | 1.715 | 1.688–1.729 |
| corpus: ishikawa-docs | 1/1 | 通过 | 0.01761 | 0.01907 | 1.083 | 1.076–1.108 |
| corpus: journey-docs | 1/1 | 通过 | 0.01529 | 0.01974 | 1.291 | 1.260–1.394 |
| corpus: kanban-docs | 3/3 | 通过 | 0.06108 | 0.06124 | 1.003 | 0.978–1.010 |
| corpus: mindmap-docs | 13/13 | 通过 | 0.08121 | 0.08830 | 1.087 | 1.079–1.119 |
| corpus: packet-docs | 4/4 | 通过 | 0.05950 | 0.06239 | 1.049 | 1.033–1.067 |
| corpus: pie-docs | 2/2 | 通过 | 0.02391 | 0.02286 | 0.956 | 0.939–0.956 |
| corpus: quadrant-docs | 3/3 | 通过 | 0.05393 | 0.05237 | 0.971 | 0.971–0.986 |
| corpus: radar-docs | 3/3 | 通过 | 0.07299 | 0.05200 | 0.712 | 0.710–0.717 |
| corpus: railroad-docs | 14/14 | 通过 | 0.30687 | 0.35602 | 1.160 | 1.158–1.217 |
| corpus: requirement-docs | 9/9 | 通过 | 0.26968 | 0.29603 | 1.098 | 1.083–1.113 |
| corpus: sankey-docs | 8/8 | 通过 | 0.30568 | 0.24106 | 0.789 | 0.769–0.789 |
| corpus: sequence-docs | 38/38 | 通过 | 0.59665 | 0.65781 | 1.103 | 1.095–1.136 |
| corpus: state-docs | 22/22 | 通过 | 0.84641 | 0.64589 | 0.763 | 0.732–0.799 |
| corpus: swimlane-docs | 13/13 | 通过 | 0.52919 | 0.51796 | 0.979 | 0.958–0.994 |
| corpus: timeline-docs | 14/14 | 通过 | 0.29014 | 0.32541 | 1.122 | 1.122–1.122 |
| corpus: treemap-docs | 8/8 | 通过 | 0.17669 | 0.13823 | 0.782 | 0.773–0.784 |
| corpus: treeview-docs | 13/13 | 通过 | 0.15813 | 0.11000 | 0.696 | 0.694–0.732 |
| corpus: usecase-docs | 24/24 | 通过 | 0.90777 | 0.77387 | 0.852 | 0.841–0.894 |
| corpus: venn-docs | 8/8 | 通过 | 0.45077 | 0.21364 | 0.474 | 0.469–0.480 |
| corpus: wardley-docs | 19/19 | 通过 | 0.30419 | 0.27746 | 0.912 | 0.903–0.913 |
| corpus: xychart-docs | 8/8 | 通过 | 0.13985 | 0.16840 | 1.204 | 1.150–1.209 |

## 非法输入拒绝

| case | 非法输入 | pele rejection ms | zebra-mermaid rejection ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| corpus: agentflow-reference | 59 | 0.73839 | 0.29288 | 0.397 |

## 仅解析

| case | 有效输入 | pele parse ms | zebra-mermaid parse ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114 | 0.44736 | 0.37396 | 0.836 |
| tree: 50 nodes | 1 | 0.05179 | 0.05748 | 1.110 |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.06408 | 0.06348 | 0.991 |
| tree: 500 nodes | 1 | 0.55074 | 0.58062 | 1.054 |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 0.71258 | 0.64751 | 0.909 |
| corpus: agentflow-docs | 12 | 0.20553 | 0.24523 | 1.193 |
| corpus: agentflow-reference | 180 | 1.62355 | 1.44181 | 0.888 |
| corpus: architecture-docs | 6 | 0.11067 | 0.03096 | 0.280 |
| corpus: block-docs | 30 | 0.08576 | 0.07735 | 0.902 |
| corpus: c4-docs | 6 | 0.09709 | 0.06848 | 0.705 |
| corpus: class-docs | 38 | 0.16844 | 0.16718 | 0.993 |
| corpus: cynefin-docs | 3 | 0.01712 | 0.00719 | 0.420 |
| corpus: er-docs | 25 | 0.10632 | 0.12645 | 1.189 |
| corpus: eventmodeling-docs | 12 | 0.09141 | 0.04260 | 0.466 |
| corpus: flowchart-docs | 114 | 0.45057 | 0.37077 | 0.823 |
| corpus: gantt-docs | 11 | 0.06215 | 0.03811 | 0.613 |
| corpus: git-docs | 35 | 0.55324 | 0.29250 | 0.529 |
| corpus: info-docs | 2 | 0.00119 | 0.00142 | 1.190 |
| corpus: ishikawa-docs | 1 | 0.00313 | 0.00259 | 0.827 |
| corpus: journey-docs | 1 | 0.00290 | 0.00218 | 0.752 |
| corpus: kanban-docs | 3 | 0.01623 | 0.01130 | 0.696 |
| corpus: mindmap-docs | 13 | 0.01980 | 0.01562 | 0.789 |
| corpus: packet-docs | 4 | 0.02760 | 0.01530 | 0.554 |
| corpus: pie-docs | 2 | 0.01006 | 0.00666 | 0.662 |
| corpus: quadrant-docs | 3 | 0.02794 | 0.01791 | 0.641 |
| corpus: radar-docs | 3 | 0.04270 | 0.01825 | 0.427 |
| corpus: railroad-docs | 14 | 0.09753 | 0.06605 | 0.677 |
| corpus: requirement-docs | 9 | 0.06042 | 0.04707 | 0.779 |
| corpus: sankey-docs | 8 | 0.05390 | 0.03490 | 0.648 |
| corpus: sequence-docs | 38 | 0.18678 | 0.16272 | 0.871 |
| corpus: state-docs | 22 | 0.20052 | 0.18705 | 0.933 |
| corpus: swimlane-docs | 13 | 0.12941 | 0.12185 | 0.942 |
| corpus: timeline-docs | 14 | 0.05713 | 0.04597 | 0.805 |
| corpus: treemap-docs | 8 | 0.08630 | 0.03241 | 0.376 |
| corpus: treeview-docs | 13 | 0.09281 | 0.04540 | 0.489 |
| corpus: usecase-docs | 24 | 0.26840 | 0.25929 | 0.966 |
| corpus: venn-docs | 8 | 0.02475 | 0.01640 | 0.663 |
| corpus: wardley-docs | 19 | 0.13288 | 0.07695 | 0.579 |
| corpus: xychart-docs | 8 | 0.04586 | 0.03316 | 0.723 |

## 进程启动与单图渲染

同一张四节点流程图，预运行后各测 30 次，交替启动顺序，验证完整 SVG 一致。

| 运行方式 | 中位耗时 ms | 最小–最大 ms |
| --- | ---: | ---: |
| Rust CLI | 2.057 | 1.594–2.356 |
| Node + Pele 参照脚本 | 68.978 | 67.689–83.755 |

该项包含进程创建、模块加载、输入输出及一次渲染，使用已热的文件系统缓存。Node 参照脚本不是 Pele 官方 CLI，不能把启动时间比值当成渲染算法加速倍数。

## 体积、限制及数据

- Rust 33 类 CLI：4,919,584 bytes（未单独 strip）；Pele JS bundle：678,032 bytes。两者分发形式不同，不据此声称部署体积优势。
- 尚未测量峰值内存、分配次数及其他平台。测试结果中的慢项应保留，不能泛称 Rust 全面更快。
- 图类型和基准覆盖完成，不等于 6,810 项原始测试已经全部 1:1 移植；后者单独统计。
- [汇总 JSON](research/benchmark-2026-10-19-33-types/comparison.json) 及同目录的输入和六份原始运行报告保留样本、重复次数、逐输入校验与运行时版本。
- 输入 SHA-256：`3394dca947019058a8c45335c6f09ce2a0e3f938b2e8ef3af2ef03ea89ddcf90`。

```sh
node tools/bench/cases.mjs
CARGO_TARGET_DIR=target node tools/bench/run.mjs
node tools/bench/compare.mjs
```
