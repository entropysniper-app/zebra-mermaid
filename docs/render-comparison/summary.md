# Pele Sample 三引擎渲染对照

[打开可视化对照表](index.html)。每行展示同一份源码的 Rust、Pele 和 Mermaid.js 输出；支持按类型、源码和状态筛选，点击图形放大。

样例：固定 Pele 提交 `44da152bbb79b91f03e68016f42499bb25bf6e82` 的 `sampleGroups()`，共 33 类、167 个，包含上游自有示例及原始选择顺序。版本：zebra-mermaid 0.1.0、Pele 1.6.0、Mermaid.js 12.1.0。

| 类型 | Sample 数 | Rust 成功 | Pele 成功 | Mermaid.js 成功 | Rust/Pele SVG 与尺寸一致 |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 6 | 6 | 6 | 6/6 |
| block | 6 | 6 | 6 | 6 | 6/6 |
| c4 | 6 | 6 | 6 | 6 | 6/6 |
| class | 6 | 6 | 6 | 6 | 6/6 |
| er | 6 | 6 | 6 | 6 | 6/6 |
| eventmodeling | 6 | 6 | 6 | 6 | 6/6 |
| gantt | 6 | 6 | 6 | 6 | 6/6 |
| git | 6 | 6 | 6 | 6 | 6/6 |
| info | 2 | 2 | 2 | 2 | 2/2 |
| journey | 1 | 1 | 1 | 1 | 1/1 |
| kanban | 3 | 3 | 3 | 3 | 3/3 |
| mindmap | 6 | 6 | 6 | 6 | 6/6 |
| packet | 4 | 4 | 4 | 4 | 4/4 |
| pie | 3 | 3 | 3 | 3 | 3/3 |
| quadrant | 3 | 3 | 3 | 3 | 3/3 |
| requirement | 6 | 6 | 6 | 6 | 6/6 |
| sankey | 6 | 6 | 6 | 6 | 6/6 |
| sequence | 6 | 6 | 6 | 6 | 6/6 |
| state | 6 | 6 | 6 | 6 | 6/6 |
| timeline | 6 | 6 | 6 | 6 | 6/6 |
| xychart | 6 | 6 | 6 | 6 | 6/6 |
| agentflow | 6 | 6 | 6 | 6 | 6/6 |
| architecture | 6 | 6 | 6 | 6 | 6/6 |
| cynefin | 3 | 3 | 3 | 3 | 3/3 |
| ishikawa | 1 | 1 | 1 | 1 | 1/1 |
| radar | 3 | 3 | 3 | 3 | 3/3 |
| railroad | 6 | 6 | 6 | 6 | 6/6 |
| swimlane | 6 | 6 | 6 | 6 | 6/6 |
| treemap | 6 | 6 | 6 | 6 | 6/6 |
| treeview | 6 | 6 | 6 | 6 | 6/6 |
| usecase | 6 | 6 | 6 | 6 | 6/6 |
| venn | 6 | 6 | 6 | 6 | 6/6 |
| wardley | 6 | 6 | 6 | 6 | 6/6 |
| **合计** | **167** | **167** | **167** | **167** | **167/167** |

样式对齐 Pele Playground 默认深色：直接编译 base.scss、pele-theme.scss、playground.scss，并从浏览器读取最终 CSS 变量。背景 #1c1b1a、节点表面 #282726、前景 #cecdc3、圆角 6px。Rust/Pele 使用 Playground 系统字体栈、siteIcon 和同一真实浏览器宿主字体测量；Rust 的解析、布局和 SVG 仍独立执行。Mermaid 按 Playground 初始化为 dark 主题，保留输入中的 front matter 设置。画布宽 760px，可布局宽 712px，时钟固定为 2026-10-19 00:00 UTC。实际变量和源码哈希见 playground-style.json。

Mermaid 的输出按其支持的 HTML DOM 插入方式呈现，再通过 XMLSerializer 导出独立 SVG；其中 17 份原始字符串含 HTML 标签，无法直接按 XML 解析。原始字符串、规范化后的 SVG、源输入、尺寸及各自 SHA-256 均保留在 [results.json](results.json) 和相邻目录。这些是导出格式处理，不算引擎渲染失败。

渲染成功只表示返回了可展示结果，不证明与 Mermaid 语义或像素完全一致，也不替代 6,810 项原始测试验收。外部资源请求和失败信息逐项保留；Pele/Rust 使用上游 siteIcon；Mermaid 未额外注册图标包，与 Playground 保持一致。

## 复现

需要固定的 Pele 研究 checkout、其 website 锁定依赖和已安装浏览器的 Playwright。

```sh
cargo build --release -p zebra-mermaid --example playground_sample
# 若 Playwright 安装在其他目录，可通过 PLAYWRIGHT_MODULE 指定其模块位置。
node tools/compare/render-samples.mjs
node tools/compare/build-report.mjs
node tools/compare/readme-previews.mjs
```

可用 CARGO_TARGET_DIR 和 PLAYWRIGHT_BROWSERS_PATH 覆盖构建与浏览器缓存目录。生成的 index.html 可直接打开，也可通过任意静态 HTTP 服务查看。
