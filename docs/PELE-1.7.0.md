# Pele 1.7.0 对齐记录

按用户 2026-10-20 的升级要求，将当前参照从 `44da152bbb79b91f03e68016f42499bb25bf6e82` 升至正式发布提交 `8603bf3313add9143cd417015d4a6bd6c522dada`。运行时仍是纯 Rust，不增加 Node、浏览器或外部布局依赖。

来源：[1.7.0 发布说明](https://github.com/obsidianmd/pele/releases/tag/1.7.0)、[相对原固定提交的完整变更](https://github.com/obsidianmd/pele/compare/44da152bbb79b91f03e68016f42499bb25bf6e82...8603bf3313add9143cd417015d4a6bd6c522dada)。发布说明中的中日文换行优化已包含在原固定提交中；此次保留实现，并补齐该优化的原始测试映射。

## 实现

| 上游修改 | Rust 对应 |
| --- | --- |
| `MermaidType`、`Detected` 和检测器的原始类型名 | `MermaidType` 的 37 个枚举值、`detect::Detected`、`detect::detect_diagram`；保持检测顺序、C4 非首分支未锚定的行为与 Railroad 大小写规则 |
| `parse()` 返回带只读 `mermaidType` 的 `ParsedDiagramModel` | `ParsedDiagram::mermaid_type()` 与 `ParsedDiagramModel` 类型别名；字段保存在不可变解析封装中，`model()` 仍返回原 Rust 图模型 |
| `render(source/model)` 返回 `mermaidType` | `RenderResult::mermaid_type`；`render_parsed()` 和缓存保留最初检测到的方言 |
| `tagErrors` 与输出限制错误 | `Error::mermaid_type: Option<MermaidType>`；检测后为解析、渲染和输出限制错误补齐缺失类型，不覆盖已有类型；检测前的输入限制或未知类型错误仍为 `None` |
| `info` 版本文本 | 默认 `Pele v1.7.0`；CLI 自身版本仍是 `0.1.0` |
| 原始版本号和 API 文档 | Rust 文档、CLI JSON、版权说明、当前工具版本标识与本记录同步 |

`detect_type()`、图表 `type`、SVG 的 `pele-*` class 和 `aria-roledescription` 继续使用 Pele 的名称；新增字段给出 Mermaid 的名称：

| 源码头部 | Pele 类型 | Mermaid 类型 |
| --- | --- | --- |
| `graph TD` / `flowchart LR` | `flowchart` | `flowchart-v2` |
| `flowchart-elk TD` | `flowchart` | `flowchart-elk` |
| `classDiagram` / `classDiagram-v2` | `class` | `classDiagram` |
| `stateDiagram` / `stateDiagram-v2` | `state` | `stateDiagram` |
| `railroad-beta` | `railroad` | `railroad` |
| `railroad-ebnf-beta` | `railroad` | `railroadEbnf` |
| `railroad-abnf-beta` | `railroad` | `railroadAbnf` |
| `railroad-peg-beta` | `railroad` | `railroadPeg` |

```rust
use zebra_mermaid::{parse, render_parsed, MermaidType, ParseOptions, RenderOptions};

let model = parse("graph TD\n  A --> B", &ParseOptions::default()).unwrap();
assert_eq!(model.mermaid_type(), MermaidType::FlowchartV2);
let result = render_parsed(&model, &RenderOptions::default()).unwrap();
assert_eq!(result.diagram_type.as_str(), "flowchart");
assert_eq!(result.mermaid_type.as_str(), "flowchart-v2");
```

CLI 的 `render --format json` 和 `check --format json` 增加 `mermaidType`；普通库渲染路径的错误 JSON 也携带该字段。Zebra 原生主题、当前宿主 palette 传递和 C ABI v2 的结构保持原有契约。

精简构建验证还修复了一个既有问题：用于开启 serde 的自 dev-dependency 同时隐式开启了全部默认图表，使 `--no-default-features` 测试失去实际作用。现禁止该隐式开启，并用值匹配及借用字段处理无图表时的空 `DiagramModel`；不增加兜底分支，也不取消枚举穷尽性检查。

## 原始测试与证据

- 重新执行固定上游的全部测试，包含网站与宿主：**292 个文件、6,826 项；6,721 通过、103 skip、2 todo**。此数字是 TypeScript 参照结果，不能计入 Rust 通过数。
- 原有 6,810 项全部保留。ID 包含上游提交，所以同步更新 inventory、映射和已有 IR 的 ID；已移植的输入与预期值不变，生成器 `--check` 逐一核验。
- 原有 2,770 项映射全部保留。新增 `mermaid-type.test.ts` 的全部 **16 项**（13 个参数行和 3 个普通测试），并补齐 `cjk-wrap.test.ts` 的全部 **6 项**。`info` 模型相等断言增加上游要求的类型字段检查。
- CJK 测试保留原日文、韩文、英文输入，全部字符和词边界断言，以及 40…240、步长 8 的标点换行循环；没有放宽宽度阈值。
- 更新后的 `entries.test.ts` 与 `mount.test.ts` 原文已冻结。对应宿主行为仍属于未移植项，没有用类型检测或 SVG 测试冒充 DOM/加载器行为。
- 验收统计保留上游 skip/todo。两个原标记 skip 的 Class/ER 测试虽在 Rust 实际执行通过，也不计入正式验收通过数；报告分别保存 `executionState`、验收 `state` 和 `executedUpstreamSkips`。未添加 Rust ignore，也未删除这些测试的断言。

本轮独立超时门禁为 **2,785 验收通过、0 失败、7 保留跳过、4,034 未映射、0 待验证**，实际执行通过 2,787 项，其中 2 个源 skip 项不计入正式通过数。`complete: false`，命令退出 1。原测试进度与独立超时门禁结果见 [UPSTREAM-TEST-PARITY.md](UPSTREAM-TEST-PARITY.md)。存在未移植和跳过项，不能宣布完整验收通过。

## 输出对照与复现

最终工作区回归 **3,189 passed、0 failed、5 ignored**，真正无默认图表构建的两项回归测试通过，验收统计规则的两项 Node 测试通过。运行命令、分类结果、原始测试二进制/源码 SHA-256 和迁移核对记录见 [验证汇总 JSON](research/pele-1.7.0-validation.json)。

新建独立的 [`pele-1.7.0.jsonl.gz`](../tests/fixtures/pele-1.7.0.jsonl.gz)，全部期望值由固定上游源码生成，保留所有 corpus 输入和原顺序。**761/761 条完整响应一致，其中 702 条成功、59 条预期拒绝**；成功输入覆盖全部 33 类。比较完整 SVG、尺寸、链接、Pele/Mermaid 类型名和错误 code/message/line/column/snippet/type/mermaidType。JSON 数字按数值精确比较，不引入容差。

测量采用上游 Arial 字宽表、固定 `idPrefix: baseline`、固定时钟 `2026-10-19T00:00:00Z` 和 UTC。来源文件 SHA-256、提交及 Node 版本保存在 [元数据](../tests/fixtures/pele-1.7.0.metadata.json)。运行时版本为 Node v26.3.0、rustc 1.96.1。

旧版黄金输出和性能报告保留原版本标识。当前英文首页、渲染画廊和[四引擎性能表](BENCHMARK.md)已重新生成：167 个 Sample 三种引擎全部成功，Rust/Pele 完全一致；四引擎共同可比集合为 110 个 Sample、22 类，采用三轮独立进程测量。39 组扩展/资源性能矩阵与采集全集未在此轮重跑，不据此声称新增加速比例。开发期 oracle 改为从当前源代码重新构建，避免 checkout 升级后误用旧 bundle。历史 fixture 生成器要求原提交；新版本使用独立生成入口。

```sh
git -C .research/pele checkout --detach 8603bf3313add9143cd417015d4a6bd6c522dada
node tools/test-parity/inventory.mjs
node tools/compat/gen-release-fixtures.mjs
cargo test --workspace --all-features --locked -- --test-threads=1
cargo test -p zebra-mermaid --no-default-features --test mermaid_type
node --test tools/test-parity/result.test.mjs
node tools/test-parity/check.mjs --require-complete
```

最后一条命令对未移植/跳过保持非零退出；子集执行成功不代表全套完成。无 feature 测试只验证未启用图表的错误元数据，不计作图表实现覆盖。
