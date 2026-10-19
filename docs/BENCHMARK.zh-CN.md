# 全图类型性能与一致性对比

测量时间：2026-10-19T23:19:38.603Z。Pele 固定提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`。Rust 实际构建启用 **33/33 类**。

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

33 类文档工作负载的 render 比值几何平均为 **0.628**（每类等权，不能代表任意应用的总体速度）。

## 成功渲染

| case | 成功输入/总输入 | 一致性 | pele ms | zebra-mermaid ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114/114 | 通过 | 2.77588 | 1.14345 | 0.412 | 0.375–0.491 |
| tree: 50 nodes | 1/1 | 通过 | 0.25215 | 0.16344 | 0.648 | 0.579–0.668 |
| mesh: 50 nodes, 20 extra labelled edges | 1/1 | 通过 | 0.71310 | 0.30476 | 0.427 | 0.356–0.431 |
| tree: 500 nodes | 1/1 | 通过 | 3.03223 | 1.69246 | 0.558 | 0.521–0.568 |
| mesh: 500 nodes, 200 extra labelled edges | 1/1 | 通过 | 9.73700 | 4.97833 | 0.511 | 0.507–0.519 |
| corpus: agentflow-docs | 12/12 | 通过 | 0.53020 | 0.39420 | 0.743 | 0.597–0.747 |
| corpus: agentflow-reference | 180/239 | 通过 | 4.03307 | 2.49248 | 0.618 | 0.536–0.639 |
| corpus: architecture-docs | 6/6 | 通过 | 0.23249 | 0.10532 | 0.453 | 0.441–0.482 |
| corpus: block-docs | 30/30 | 通过 | 0.26412 | 0.21555 | 0.816 | 0.751–0.894 |
| corpus: c4-docs | 6/6 | 通过 | 0.62570 | 0.38056 | 0.608 | 0.602–0.635 |
| corpus: class-docs | 38/38 | 通过 | 0.90777 | 0.58151 | 0.641 | 0.583–0.734 |
| corpus: cynefin-docs | 3/3 | 通过 | 0.10472 | 0.07214 | 0.689 | 0.681–0.741 |
| corpus: er-docs | 25/25 | 通过 | 0.70871 | 0.49287 | 0.695 | 0.680–0.710 |
| corpus: eventmodeling-docs | 12/12 | 通过 | 0.26033 | 0.17275 | 0.664 | 0.628–0.678 |
| corpus: flowchart-docs | 114/114 | 通过 | 2.27494 | 1.10973 | 0.488 | 0.363–0.533 |
| corpus: gantt-docs | 11/11 | 通过 | 0.37968 | 0.23478 | 0.618 | 0.534–0.622 |
| corpus: git-docs | 35/35 | 通过 | 1.40193 | 0.82710 | 0.590 | 0.581–0.602 |
| corpus: info-docs | 2/2 | 通过 | 0.00243 | 0.00226 | 0.930 | 0.913–0.976 |
| corpus: ishikawa-docs | 1/1 | 通过 | 0.01970 | 0.01380 | 0.701 | 0.681–0.714 |
| corpus: journey-docs | 1/1 | 通过 | 0.01822 | 0.01529 | 0.839 | 0.766–0.927 |
| corpus: kanban-docs | 3/3 | 通过 | 0.06772 | 0.04448 | 0.657 | 0.647–0.660 |
| corpus: mindmap-docs | 13/13 | 通过 | 0.08871 | 0.06297 | 0.710 | 0.694–0.790 |
| corpus: packet-docs | 4/4 | 通过 | 0.06470 | 0.04141 | 0.640 | 0.634–0.686 |
| corpus: pie-docs | 2/2 | 通过 | 0.02617 | 0.01591 | 0.608 | 0.608–0.611 |
| corpus: quadrant-docs | 3/3 | 通过 | 0.06138 | 0.04059 | 0.661 | 0.511–0.668 |
| corpus: radar-docs | 3/3 | 通过 | 0.08497 | 0.03392 | 0.399 | 0.399–0.428 |
| corpus: railroad-docs | 14/14 | 通过 | 0.35381 | 0.28472 | 0.805 | 0.805–0.845 |
| corpus: requirement-docs | 9/9 | 通过 | 0.30004 | 0.22475 | 0.749 | 0.503–0.756 |
| corpus: sankey-docs | 8/8 | 通过 | 0.33880 | 0.14304 | 0.422 | 0.410–0.464 |
| corpus: sequence-docs | 38/38 | 通过 | 0.69990 | 0.53201 | 0.760 | 0.740–0.832 |
| corpus: state-docs | 22/22 | 通过 | 0.98771 | 0.47991 | 0.486 | 0.474–0.525 |
| corpus: swimlane-docs | 13/13 | 通过 | 0.59445 | 0.38768 | 0.652 | 0.642–0.715 |
| corpus: timeline-docs | 14/14 | 通过 | 0.32457 | 0.25976 | 0.800 | 0.798–0.852 |
| corpus: treemap-docs | 8/8 | 通过 | 0.19306 | 0.10797 | 0.559 | 0.454–0.564 |
| corpus: treeview-docs | 13/13 | 通过 | 0.17430 | 0.08336 | 0.478 | 0.476–0.500 |
| corpus: usecase-docs | 24/24 | 通过 | 1.09961 | 0.61150 | 0.556 | 0.556–0.595 |
| corpus: venn-docs | 8/8 | 通过 | 0.48226 | 0.20350 | 0.422 | 0.351–0.450 |
| corpus: wardley-docs | 19/19 | 通过 | 0.33705 | 0.22193 | 0.658 | 0.637–0.658 |
| corpus: xychart-docs | 8/8 | 通过 | 0.15298 | 0.10228 | 0.669 | 0.659–0.670 |

## 非法输入拒绝

| case | 非法输入 | pele rejection ms | zebra-mermaid rejection ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| corpus: agentflow-reference | 59 | 0.77963 | 0.25374 | 0.325 |

## 仅解析

| case | 有效输入 | pele parse ms | zebra-mermaid parse ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |
| docs: all flowchart examples | 114 | 0.58644 | 0.31289 | 0.534 |
| tree: 50 nodes | 1 | 0.06099 | 0.05003 | 0.820 |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.06731 | 0.05425 | 0.806 |
| tree: 500 nodes | 1 | 0.62586 | 0.52408 | 0.837 |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 0.77368 | 0.57660 | 0.745 |
| corpus: agentflow-docs | 12 | 0.23271 | 0.21460 | 0.922 |
| corpus: agentflow-reference | 180 | 1.97879 | 1.21341 | 0.613 |
| corpus: architecture-docs | 6 | 0.12299 | 0.03023 | 0.246 |
| corpus: block-docs | 30 | 0.10635 | 0.06328 | 0.595 |
| corpus: c4-docs | 6 | 0.10426 | 0.06733 | 0.646 |
| corpus: class-docs | 38 | 0.19438 | 0.13396 | 0.689 |
| corpus: cynefin-docs | 3 | 0.01825 | 0.00682 | 0.374 |
| corpus: er-docs | 25 | 0.12784 | 0.10980 | 0.859 |
| corpus: eventmodeling-docs | 12 | 0.10157 | 0.03849 | 0.379 |
| corpus: flowchart-docs | 114 | 0.50629 | 0.30654 | 0.605 |
| corpus: gantt-docs | 11 | 0.06829 | 0.03310 | 0.485 |
| corpus: git-docs | 35 | 0.59614 | 0.24361 | 0.409 |
| corpus: info-docs | 2 | 0.00137 | 0.00085 | 0.617 |
| corpus: ishikawa-docs | 1 | 0.00352 | 0.00240 | 0.683 |
| corpus: journey-docs | 1 | 0.00308 | 0.00182 | 0.591 |
| corpus: kanban-docs | 3 | 0.01799 | 0.00938 | 0.521 |
| corpus: mindmap-docs | 13 | 0.02284 | 0.01205 | 0.527 |
| corpus: packet-docs | 4 | 0.02971 | 0.01165 | 0.392 |
| corpus: pie-docs | 2 | 0.01058 | 0.00472 | 0.446 |
| corpus: quadrant-docs | 3 | 0.03057 | 0.01592 | 0.521 |
| corpus: radar-docs | 3 | 0.04642 | 0.01519 | 0.327 |
| corpus: railroad-docs | 14 | 0.11872 | 0.05510 | 0.464 |
| corpus: requirement-docs | 9 | 0.06690 | 0.04309 | 0.644 |
| corpus: sankey-docs | 8 | 0.06117 | 0.03037 | 0.497 |
| corpus: sequence-docs | 38 | 0.20151 | 0.14306 | 0.710 |
| corpus: state-docs | 22 | 0.23260 | 0.15374 | 0.661 |
| corpus: swimlane-docs | 13 | 0.14093 | 0.10450 | 0.741 |
| corpus: timeline-docs | 14 | 0.06265 | 0.03507 | 0.560 |
| corpus: treemap-docs | 8 | 0.09900 | 0.02736 | 0.276 |
| corpus: treeview-docs | 13 | 0.10120 | 0.03554 | 0.351 |
| corpus: usecase-docs | 24 | 0.30725 | 0.22117 | 0.720 |
| corpus: venn-docs | 8 | 0.02783 | 0.01294 | 0.465 |
| corpus: wardley-docs | 19 | 0.14744 | 0.06973 | 0.473 |
| corpus: xychart-docs | 8 | 0.05103 | 0.02705 | 0.530 |

## 进程启动与单图渲染

同一张四节点流程图，预运行后各测 30 次，交替启动顺序，验证完整 SVG 一致。

| 运行方式 | 中位耗时 ms | 最小–最大 ms |
| --- | ---: | ---: |
| Rust CLI | 2.667 | 1.952–6.702 |
| Node + Pele 参照脚本 | 80.409 | 75.583–129.442 |

该项包含进程创建、模块加载、输入输出及一次渲染，使用已热的文件系统缓存。Node 参照脚本不是 Pele 官方 CLI，不能把启动时间比值当成渲染算法加速倍数。

## 体积、限制及数据

- Rust 33 类 CLI：4,955,152 bytes（未单独 strip）；Pele JS bundle：678,032 bytes。两者分发形式不同，不据此声称部署体积优势。
- CPU/分配与峰值存活内存的分阶段实测见 [Class 与 Agentflow 优化报告](PROFILING-CLASS-AGENTFLOW.zh-CN.md)。尚未覆盖其他平台；慢项保留，不泛称 Rust 全面更快。
- 图类型和基准覆盖完成，不等于 6,810 项原始测试已经全部 1:1 移植；后者单独统计。
- [汇总 JSON](research/benchmark-2026-10-19-33-types-codex-takeover/comparison.json) 及同目录的输入和六份原始运行报告保留样本、重复次数、逐输入校验与运行时版本。
- 输入 SHA-256：`3394dca947019058a8c45335c6f09ce2a0e3f938b2e8ef3af2ef03ea89ddcf90`。

```sh
node tools/bench/cases.mjs
CARGO_TARGET_DIR=target node tools/bench/run.mjs
node tools/bench/compare.mjs
```
