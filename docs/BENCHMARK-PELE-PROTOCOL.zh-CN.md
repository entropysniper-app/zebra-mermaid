# 按 Pele 仓库原口径的性能对比

测量时间：2026-10-19T09:06:43.245Z。运行时：Node v26.3.0；Rust 1.96.1 release，thin LTO，codegen-units=1，启用当前全部 33 类。

参照是固定提交的 [scripts/bench.mjs](https://github.com/obsidianmd/pele/blob/44da152bbb79b91f03e68016f42499bb25bf6e82/scripts/bench.mjs)。Pele 一侧直接执行该原文件，未修改计时函数；仅捕获最终 console.table。Rust 逐行移植同一计时循环，使用该脚本直接求值获得的同一组输入。

## 与 npm run bench 相同的格式

| case | diagrams | pele | zebra-mermaid |
| --- | ---: | ---: | ---: |
| docs: all examples | 114 | 2.2 ms | 1.5 ms |
| tree: 50 nodes | 1 | 0.209 ms | 0.221 ms |
| mesh: 50 nodes, 20 extra labelled edges | 1 | 0.632 ms | 0.436 ms |
| tree: 500 nodes | 1 | 2.5 ms | 2.3 ms |
| mesh: 500 nodes, 200 extra labelled edges | 1 | 8.3 ms | 6.2 ms |

这里的时间是**整批输入的中位渲染耗时**：第一行是 114 张图的总耗时，其余每行一张。格式与原脚本一致：小于 1 ms 保留三位小数，否则保留一位。没有把 CLI 启动时间、预解析渲染、缓存命中或多轮中位数混入这张表。

## 完全对应的计时口径

- 测完整 `render(source)`，使用默认选项与默认文本测量；5 组均为流程图，不需要人为固定时钟。
- 每组预热一次；随后每个样本执行一次完整输入批次。
- 采样持续到累计耗时达到 300 ms 且至少有 5 个样本；最多 2,000 个样本。
- 排序后取 `samples[samples.length >> 1]`，与原脚本相同（偶数样本取上中位数）。
- 原脚本跳过渲染错误并在成功数量不足时标注。两边本次 118 个输入全部成功，计时后完整 SVG 字符串与尺寸逐项相等。
- 本表是一轮按原脚本执行的测量结果，不是跨平台结论。另有三轮、33 类扩展基准见 [扩展报告](BENCHMARK.zh-CN.md)，两种口径分开保存。

## 仓库首页声明的指标格式

仓库 [首页表格](https://github.com/obsidianmd/pele/blob/44da152bbb79b91f03e68016f42499bb25bf6e82/website/src/content/pages/index.md) 的指标如下。其渲染区间没有标出完整输入、样本统计和机器环境，因此不能把上面单图/批量数字冒充那个区间的严格复测。

| 指标 | Pele 仓库声明 | zebra-mermaid 当前对应情况 |
| --- | --- | --- |
| Bundle size gzip | 230 kB | 原生 Rust 库/CLI，未发布 JS/WASM bundle，无法按浏览器 bundle 同口径比较 |
| Render time | 0.05–0.3 ms | 同口径可复现结果见上方原脚本的五组表；不套用未注明输入的区间 |
| Direct dependencies | 0 | Rust 核心有 2 个必需直接依赖：indexmap、url；serde 可选 |
| Small screens | Adapts the layout | 已实现的类型保留 max_width/方向适配；Zebra 原生宽度策略单独兼容 |
| Theming | CSS variables | 保留 Pele CSS 变量；Zebra 原生使用当前宿主主题生成具体颜色 |

## 构建体积实测

原脚本输出：

`bundle: 662.1 kB minified, 245.8 kB gzipped`

该固定版本的实测 gzip 为 245.8 kB，与首页声明的 230 kB 不相同。Rust 当前 33 类 CLI 为 4804.3 kB 原生可执行文件、gzip 后 2223.5 kB。它们不是同类分发产物，不据此给出“几倍更小”的结论。

## 复现与证据

```sh
# 先 release 构建 examples/bench_upstream，features 与本次 33 类构建一致
CARGO_TARGET_DIR=target node tools/bench/run-upstream.mjs
node tools/bench/report-upstream.mjs
```

[原始结果 JSON](research/upstream-benchmark-2026-10-19-33-types/comparison.json)、[Pele 原控制台输出](research/upstream-benchmark-2026-10-19-33-types/pele-console.txt) 及同目录的输入、Rust 原始样本与完整 SVG 验证输出已保存。脚本 SHA-256：`5afa837231ac555041eccae2d8b82e29414675caff0aa3ec4c48ddfaaed35ec6`。
