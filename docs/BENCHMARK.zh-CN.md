# 全图类型性能与一致性对比

测量时间：2026-10-23T10:29:49.322Z。Pele 固定提交 `e7f092a4de147cc32210913c20e36eaa5fcd0ef1`。Rust 实际构建启用 **33/33 类**。

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

33 类文档工作负载的 render 比值几何平均为 **0.539**（每类等权，不能代表任意应用的总体速度）。

## 成功渲染

| case | 成功输入/总输入 | 一致性 | pele ms | zebra-mermaid ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114/114 | 通过 | 2.12689 | 1.08429 | 0.510 | 0.482–0.540 |
| tree: 50 nodes | 1/1 | 通过 | 0.20417 | 0.15773 | 0.773 | 0.716–0.797 |
| mesh: 50 nodes, 20 extra labelled edges | 1/1 | 通过 | 0.58000 | 0.31951 | 0.551 | 0.551–0.560 |
| tree: 500 nodes | 1/1 | 通过 | 2.03707 | 1.64106 | 0.806 | 0.794–0.813 |
| mesh: 500 nodes, 200 extra labelled edges | 1/1 | 通过 | 8.58125 | 4.90008 | 0.571 | 0.548–0.582 |
| corpus: agentflow-docs | 12/12 | 通过 | 0.54671 | 0.40006 | 0.732 | 0.645–0.772 |
| corpus: agentflow-reference | 180/239 | 通过 | 3.88627 | 2.45802 | 0.632 | 0.575–0.649 |
| corpus: architecture-docs | 6/6 | 通过 | 0.20956 | 0.08843 | 0.422 | 0.402–0.449 |
| corpus: block-docs | 30/30 | 通过 | 0.25567 | 0.17083 | 0.668 | 0.616–0.730 |
| corpus: c4-docs | 6/6 | 通过 | 0.59830 | 0.31266 | 0.523 | 0.441–0.548 |
| corpus: class-docs | 38/38 | 通过 | 1.08735 | 0.49553 | 0.456 | 0.415–0.558 |
| corpus: cynefin-docs | 3/3 | 通过 | 0.10237 | 0.05245 | 0.512 | 0.487–0.522 |
| corpus: er-docs | 25/25 | 通过 | 0.63546 | 0.43279 | 0.681 | 0.661–0.691 |
| corpus: eventmodeling-docs | 12/12 | 通过 | 0.23402 | 0.10096 | 0.431 | 0.431–0.438 |
| corpus: flowchart-docs | 114/114 | 通过 | 2.00391 | 1.14684 | 0.572 | 0.552–0.575 |
| corpus: gantt-docs | 11/11 | 通过 | 0.37913 | 0.20011 | 0.528 | 0.516–0.547 |
| corpus: git-docs | 35/35 | 通过 | 1.29504 | 0.68511 | 0.529 | 0.477–0.556 |
| corpus: info-docs | 2/2 | 通过 | 0.00255 | 0.00109 | 0.428 | 0.326–0.432 |
| corpus: ishikawa-docs | 1/1 | 通过 | 0.01709 | 0.00953 | 0.558 | 0.553–0.574 |
| corpus: journey-docs | 1/1 | 通过 | 0.01359 | 0.00820 | 0.604 | 0.604–0.639 |
| corpus: kanban-docs | 3/3 | 通过 | 0.06409 | 0.03960 | 0.618 | 0.580–0.635 |
| corpus: mindmap-docs | 13/13 | 通过 | 0.08792 | 0.04656 | 0.530 | 0.524–0.530 |
| corpus: packet-docs | 4/4 | 通过 | 0.05279 | 0.02891 | 0.548 | 0.531–0.548 |
| corpus: pie-docs | 2/2 | 通过 | 0.02410 | 0.01364 | 0.566 | 0.560–0.580 |
| corpus: quadrant-docs | 3/3 | 通过 | 0.05301 | 0.03138 | 0.592 | 0.566–0.612 |
| corpus: radar-docs | 3/3 | 通过 | 0.07031 | 0.02985 | 0.425 | 0.416–0.433 |
| corpus: railroad-docs | 14/14 | 通过 | 0.31774 | 0.22271 | 0.701 | 0.668–0.709 |
| corpus: requirement-docs | 9/9 | 通过 | 0.29578 | 0.17985 | 0.608 | 0.600–0.622 |
| corpus: sankey-docs | 8/8 | 通过 | 0.40908 | 0.15957 | 0.390 | 0.388–0.393 |
| corpus: sequence-docs | 38/38 | 通过 | 0.56801 | 0.34326 | 0.604 | 0.580–0.619 |
| corpus: state-docs | 22/22 | 通过 | 0.86648 | 0.49470 | 0.571 | 0.535–0.594 |
| corpus: swimlane-docs | 13/13 | 通过 | 0.52846 | 0.36919 | 0.699 | 0.674–0.705 |
| corpus: timeline-docs | 14/14 | 通过 | 0.26545 | 0.16592 | 0.625 | 0.608–0.628 |
| corpus: treemap-docs | 8/8 | 通过 | 0.17832 | 0.07974 | 0.447 | 0.412–0.461 |
| corpus: treeview-docs | 13/13 | 通过 | 0.17027 | 0.07269 | 0.427 | 0.421–0.445 |
| corpus: usecase-docs | 24/24 | 通过 | 1.21512 | 0.63127 | 0.520 | 0.483–0.541 |
| corpus: venn-docs | 8/8 | 通过 | 0.49671 | 0.20455 | 0.412 | 0.410–0.414 |
| corpus: wardley-docs | 19/19 | 通过 | 0.31309 | 0.16908 | 0.540 | 0.503–0.572 |
| corpus: xychart-docs | 8/8 | 通过 | 0.14512 | 0.08583 | 0.591 | 0.561–0.613 |

## 非法输入拒绝

| case | 非法输入 | pele rejection ms | zebra-mermaid rejection ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| corpus: agentflow-reference | 59 | 0.80712 | 0.26515 | 0.329 |

## 仅解析

| case | 有效输入 | pele parse ms | zebra-mermaid parse ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114 | 0.53291 | 0.31285 | 0.587 |
| tree: 50 nodes | 1 | 0.05614 | 0.05240 | 0.933 |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.05766 | 0.05963 | 1.034 |
| tree: 500 nodes | 1 | 0.51810 | 0.53442 | 1.032 |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 0.64265 | 0.60946 | 0.948 |
| corpus: agentflow-docs | 12 | 0.23783 | 0.23310 | 0.980 |
| corpus: agentflow-reference | 180 | 1.93381 | 1.35655 | 0.701 |
| corpus: architecture-docs | 6 | 0.12062 | 0.02742 | 0.227 |
| corpus: block-docs | 30 | 0.11832 | 0.06459 | 0.546 |
| corpus: c4-docs | 6 | 0.10919 | 0.07037 | 0.644 |
| corpus: class-docs | 38 | 0.19301 | 0.14361 | 0.744 |
| corpus: cynefin-docs | 3 | 0.01911 | 0.00540 | 0.282 |
| corpus: er-docs | 25 | 0.11971 | 0.12140 | 1.014 |
| corpus: eventmodeling-docs | 12 | 0.10321 | 0.03581 | 0.347 |
| corpus: flowchart-docs | 114 | 0.48000 | 0.30560 | 0.637 |
| corpus: gantt-docs | 11 | 0.06714 | 0.03454 | 0.515 |
| corpus: git-docs | 35 | 0.58842 | 0.25794 | 0.438 |
| corpus: info-docs | 2 | 0.00160 | 0.00044 | 0.277 |
| corpus: ishikawa-docs | 1 | 0.00346 | 0.00223 | 0.646 |
| corpus: journey-docs | 1 | 0.00329 | 0.00148 | 0.449 |
| corpus: kanban-docs | 3 | 0.01645 | 0.00956 | 0.582 |
| corpus: mindmap-docs | 13 | 0.02392 | 0.01119 | 0.468 |
| corpus: packet-docs | 4 | 0.02911 | 0.01097 | 0.377 |
| corpus: pie-docs | 2 | 0.01031 | 0.00448 | 0.435 |
| corpus: quadrant-docs | 3 | 0.03102 | 0.01634 | 0.527 |
| corpus: radar-docs | 3 | 0.04398 | 0.01490 | 0.339 |
| corpus: railroad-docs | 14 | 0.11101 | 0.06026 | 0.543 |
| corpus: requirement-docs | 9 | 0.06638 | 0.04495 | 0.677 |
| corpus: sankey-docs | 8 | 0.06484 | 0.03867 | 0.596 |
| corpus: sequence-docs | 38 | 0.20595 | 0.11122 | 0.540 |
| corpus: state-docs | 22 | 0.21820 | 0.17140 | 0.785 |
| corpus: swimlane-docs | 13 | 0.13540 | 0.10860 | 0.802 |
| corpus: timeline-docs | 14 | 0.05991 | 0.03250 | 0.542 |
| corpus: treemap-docs | 8 | 0.09369 | 0.02488 | 0.266 |
| corpus: treeview-docs | 13 | 0.10067 | 0.03434 | 0.341 |
| corpus: usecase-docs | 24 | 0.31065 | 0.22342 | 0.719 |
| corpus: venn-docs | 8 | 0.02348 | 0.01401 | 0.597 |
| corpus: wardley-docs | 19 | 0.14407 | 0.06375 | 0.442 |
| corpus: xychart-docs | 8 | 0.04877 | 0.02866 | 0.588 |

## 进程启动与单图渲染

同一张四节点流程图，预运行后各测 30 次，交替启动顺序，验证完整 SVG 一致。

| 运行方式 | 中位耗时 ms | 最小–最大 ms |
| --- | ---: | ---: |
| Rust CLI | 3.938 | 3.061–5.572 |
| Node + Pele 参照脚本 | 84.626 | 75.476–132.903 |

该项包含进程创建、模块加载、输入输出及一次渲染，使用已热的文件系统缓存。Node 参照脚本不是 Pele 官方 CLI，不能把启动时间比值当成渲染算法加速倍数。

## 体积、限制及数据

- Rust 33 类 CLI：9,237,136 bytes（未单独 strip）；Pele JS bundle：701,816 bytes。两者分发形式不同，不据此声称部署体积优势。
- 本次版本对齐未重测 CPU、分配或 RSS；[Class 与 Agentflow 历史剖析](PROFILING-CLASS-AGENTFLOW.zh-CN.md) 不代表本版本。尚未覆盖其他平台；慢项保留，不泛称 Rust 全面更快。
- 图类型和基准覆盖完成，不等于 全量原始测试已经全部 1:1 移植；后者单独统计。
- [汇总 JSON](research/benchmark-2026-10-23-33-types-pele-1-9-0-ratex/comparison.json) 及同目录的输入和六份原始运行报告保留样本、重复次数、逐输入校验与运行时版本。
- 输入 SHA-256：`3394dca947019058a8c45335c6f09ce2a0e3f938b2e8ef3af2ef03ea89ddcf90`。

```sh
node tools/bench/cases.mjs
CARGO_TARGET_DIR=target node tools/bench/run.mjs
node tools/bench/compare.mjs
```
