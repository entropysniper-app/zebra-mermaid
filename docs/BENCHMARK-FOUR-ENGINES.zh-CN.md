# 四引擎性能横向对比

测量时间：2026-10-19T23:26:15.591561+00:00。

本次实际比较 Mermaid.js、Pele、zebra-mermaid 和 [mermaid-rs-renderer](https://github.com/1jehuang/mermaid-rs-renderer/tree/3726ccbffe0e8032361eb9668694b24f77858060)。输入为固定 Pele Playground 的 **33 类、167 个 Sample**，四个引擎逐项尝试完全相同的源码，不为某个引擎改写语法。

## 版本与口径

- Mermaid.js 12.1.0，Chromium 151.0.7922.34，真实浏览器中调用 `mermaid.render`。
- Pele 1.6.0，固定提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`；zebra-mermaid 0.1.0，本轮优化后的 Rust 库。
- mermaid-rs-renderer 0.3.1，固定提交 `3726ccbffe0e8032361eb9668694b24f77858060`；直接调用原版 `render_with_options`，保留默认原生字体测量，`fast_text_metrics=false`。
- 两个 Rust 库使用相同 release / thin LTO / codegen-units=1 编译配置。第三方引擎仅进入独立开发驱动，不进入 zebra-mermaid 生产依赖。rustc 1.96.1 (31fca3adb 2026-06-26)；Node v26.3.0。
- 请求 Arial 16；Pele/Rust 保留 Pele CSS 变量，Mermaid.js/mmdr 使用 dark 主题；输入自身指令保留。各引擎的字体测量、布局和样式实现不同，输出尺寸和视觉效果不保证一致。

计时为**完整 parse + layout + SVG 生成的稳态墙钟耗时**，单位为同一行全部共同样本的 ms。每轮预热五批，校准到至少 10 ms，每次采样至少九个样本且累计至少 300 ms，三个独立进程轮次交错引擎顺序。每轮取样本中位数，再取三轮中位数。

浏览器启动、模块加载、字体首次预热、验证、跨进程通信及写盘不计时；Mermaid 的测量在浏览器内部完成。DOM 清理计入 Mermaid 调用循环；mmdr 的 API 消费 options，因此配置克隆计入其调用成本。未测 CLI 冷启动，本表不能用于宣称 CLI 启动速度。

## 支持范围与比较集合

| 引擎 | 返回 SVG / 167 | 通过类型与 SVG 检查 / 167 |
| --- | ---: | ---: |
| Mermaid.js | 167 | 167 |
| Pele | 167 | 167 |
| zebra-mermaid | 167 | 167 |
| mermaid-rs-renderer | 156 | 110 |

最终共同集合为 **110 个 Sample、22 个图表组**。每行四个引擎都只测该行同一组共同输入；没有共同输入的行保留 `—`。原始 167 项的成功、失败和排除原因全部保存。

SVG 检查包含 XML、有限正尺寸，以及 mmdr 的 `DiagramKind` 是否符合输入图表类型。验证中发现 mmdr 会将部分带 `-beta` 的其他图表头识别为 Flowchart；这些结果记录为“返回 SVG、类型不匹配”，不计入合格成功或耗时排名。Mermaid 的 HTML 形式 SVG 在计时外通过 DOM 序列化；Info 没有 viewBox，使用浏览器实际 viewport 尺寸。

Pele 与 zebra-mermaid 的 **167/167 SVG 字节与尺寸完全一致**。另外两个引擎只进行了可渲染性与类型校验，不能由此断言所有节点、边、文字、布局或像素语义完全等价。可在原始 SVG 中进一步核对。

## 同输入耗时

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

只应在同一行内比较。单个样例数量、图复杂度、实现的布局和字体行为会影响比值，不把某一行的倍率推广为所有 Mermaid 输入的性能结论。

## 覆盖明细

| 图表场景 | 原样本数 | Mermaid.js 合格 | Pele 合格 | zebra-mermaid 合格 | mmdr 合格 |
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

## 复现与证据

```sh
git clone https://github.com/1jehuang/mermaid-rs-renderer.git .research/mermaid-rs-renderer
git -C .research/mermaid-rs-renderer checkout 3726ccbffe0e8032361eb9668694b24f77858060
CARGO_TARGET_DIR=target/cross-bench-build cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml
# 需要固定 Mermaid.js 12.1.0 和已安装 Chromium 的 Playwright。
# PLAYWRIGHT_MODULE / PLAYWRIGHT_BROWSERS_PATH 可指向自己的安装目录。
python3 tools/bench/run-cross.py
python3 tools/bench/publish-cross.py
```

结果见 [汇总及版本](research/four-engine-benchmark-2026-10-19-codex-takeover/comparison.json)、[逐输入错误与 SVG 路径](research/four-engine-benchmark-2026-10-19-codex-takeover/verification.json)、[共同输入](research/four-engine-benchmark-2026-10-19-codex-takeover/common.json)、[全部原始输入](research/four-engine-benchmark-2026-10-19-codex-takeover/cases.json)、[源码/依赖/二进制指纹](research/four-engine-benchmark-2026-10-19-codex-takeover/metadata.json)。同目录有十二份压缩原始计时、SVG 文件与 Pele/Rust 一致性记录。

这份报告与 [Pele 原脚本五组协议](BENCHMARK-PELE-PROTOCOL.zh-CN.md)、[39 组 Rust 优化前后测量](PROFILING-MULTI-DIAGRAM.zh-CN.md) 使用不同的输入集合，分别保存，不拼接比值。图表渲染成功也不增加 6,810 项原始测试的迁移通过数。
