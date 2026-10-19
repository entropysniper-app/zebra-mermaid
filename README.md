# zebra-mermaid

纯 Rust 的 Mermaid → SVG 渲染库与命令行工具，移植自 [obsidianmd/pele](https://github.com/obsidianmd/pele)（固定提交 `44da152b`，package 1.6.0，Mermaid 语法目标 12.1.0）。运行时不依赖 JavaScript、浏览器或外部布局程序。

> **当前状态：默认支持全部 33 类图表；全量原始测试 1:1 移植仍在进行。** 39/39 组基准输入行为一致，70,183 条可表示采集输入通过差分。实际支持范围、已知限制与逐项测试进度见 [实施状态](docs/IMPLEMENTATION-STATUS.md)，不能以差分通过代替原始测试验收。

## 文档

- [实施状态（进度、实际测试结果、已知差异）](docs/IMPLEMENTATION-STATUS.md)
- [Rust 重写技术方案与实施路径](docs/RUST-REWRITE-PLAN.zh-CN.md)
- [Zebra 现有主题接入兼容契约](docs/ZEBRA-THEME-COMPATIBILITY.md)
- [测试计划](docs/TEST-PLAN.md)
- [Pele 全量测试 1:1 移植与验收](docs/UPSTREAM-TEST-PARITY.md)
- [按 Pele 原脚本格式与口径的性能对比](docs/BENCHMARK-PELE-PROTOCOL.zh-CN.md)
- [33 类、39 组扩展性能与一致性报告](docs/BENCHMARK.zh-CN.md)
- [移植规范（Porting guide）](docs/PORTING-GUIDE.md)
- [第三方许可](THIRD_PARTY_NOTICES.md)

## 仓库结构

| 路径 | 内容 |
| --- | --- |
| `crates/zebra-mermaid` | 库：预处理、YAML、文本测量、布局、SVG 输出、各图表、Zebra 主题层 |
| `crates/zebra-mermaid-cli` | 命令行工具 `zebra-mermaid` |
| `crates/zebra-mermaid-ffi` | 与 Zebra 现有原生桥同形状的 C ABI v2（`include/zebra_mermaid.h`）；尚未接入 Zebra |
| `tools/compat` | 开发期 Node oracle（打包固定 Pele 源码）、fixture 生成、上游测试输入采集 |
| `tools/bench` | 与 Pele 的性能对照脚本 |
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
