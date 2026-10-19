# zebra-mermaid 实施状态

本文件记录 Rust 重写的实际进度，供中断后恢复执行。只记录已执行并有证据的结果；计划项明确标为“待做”。

**新增验收标准：Pele 全部测试 1:1 移植且通过。** 当前清单 6,810 项，逐项映射 718 项，716 通过、2 跳过、6,092 未映射。详情见 [UPSTREAM-TEST-PARITY.md](UPSTREAM-TEST-PARITY.md)。图类型覆盖为 33/33，基准一致性为 39/39；原始测试 1:1 通过率约 10.51%，这三种指标分别统计。

- 最后更新：2026-10-19（33/33 类已接通，默认全部启用；39/39 组基准一致）
- 执行主体：Codex；用户要求停止的 Claude 会话 `63eb197d` 已停止，不自动恢复。
- 方案：[RUST-REWRITE-PLAN.zh-CN.md](RUST-REWRITE-PLAN.zh-CN.md)
- Zebra 主题契约（必要验收项，自 P1 起）：[ZEBRA-THEME-COMPATIBILITY.md](ZEBRA-THEME-COMPATIBILITY.md)
- 测试计划：[TEST-PLAN.md](TEST-PLAN.md)
- 上游参照：Pele `44da152bbb79b91f03e68016f42499bb25bf6e82`（package 1.6.0，Mermaid 语法目标 12.1.0），快照在 `.research/pele/`（不入库）
- Zebra 主题参考快照：`.research/zebra-theme-contract/`（只读；Zebra 原工程不修改）

## 1. 目标

1. 纯 Rust 库 + CLI，运行时不调用 Node/JS/外部布局程序；Node 只作开发期 oracle。
2. 按 P0–P6 覆盖上游全部 33 类图表；移植 Pele 的手写解析器、YAML 子集、文本估算与 layered/compound/lanes 布局，保持语义与几何。
3. 两种颜色输出：`css`（Pele 原样 `--pele-*` 变量，供差分与网页嵌入）和 `resolved`（具体颜色，CLI 默认与 Zebra 原生接入模式）。
4. 支持 Zebra 现有主题接入方式：camelCase `MermaidPalette`（17 个颜色 + 6 系列色）、light/dark、fontSize、opaque/transparent 画布；语义角色贯穿各图表。
5. MVP（flowchart/sequence/pie/info）只是中间里程碑，不得报告为完整重写。

## 2. 阶段状态

图表数量覆盖率为 33/33 = 100%，不等于全量原始测试、跨平台或发布验收已经完成。

| 阶段 | 内容 | 当前状态 |
| --- | --- | --- |
| P0 | 研究、workspace、固定上游、参照工具与 fixture | 完成 |
| P1 | 预处理、文本、SVG、pie/info、主题契约、单文件 CLI | 完成 |
| P2 | layered/compound/lanes、flowchart、swimlane | 已实现并通过文档与采集全集对照 |
| P3 | sequence、批量 CLI、缓存与预算 | 主体完成；最终统一验收待做 |
| P4 | class/state/er | 解析、渲染、文档与采集对照通过；原始测试逐项迁移继续 |
| P5 | 其余图类型 | usecase、agentflow 已补齐，全部 33 类通过当前文档和采集对照 |
| P6 | 全量收敛、平台构建、性能、发布 | 性能对比和 C ABI 测试已完成；全量原测试迁移、跨平台与发布验收未完成 |

## 3. 实际支持的图表类型

**默认 `implemented` 已指向 `all-diagrams`，库、CLI、FFI 默认均启用全部 33 类。** `--no-default-features` 仍可配合逐类型 feature 构建精简子集。

以下为 33 类的采集全集结果，均使用固定 Pele 提交、固定测量与时间参数；比较 SVG、尺寸、链接和错误。数量包含与上游一致的成功渲染及预期错误，不等于全部输入都应成功渲染。主题输出另外验证。

| 类型 | 当前采集全集一致数量 |
| --- | --- |
| agentflow | 3,297/3,297 |
| architecture | 1,719/1,719 |
| block | 2,665/2,665 |
| c4 | 2,939/2,939 |
| class | 3,269/3,269 |
| cynefin | 955/955 |
| er | 1,788/1,788 |
| eventmodeling | 1,015/1,015 |
| flowchart | 3,225/3,225 |
| gantt | 2,907/2,907 |
| gitGraph | 1,337/1,337 |
| info | 505/505 |
| ishikawa | 4,002/4,002 |
| journey | 1,708/1,708 |
| kanban | 2,902/2,902 |
| mindmap | 1,805/1,805 |
| packet | 1,105/1,105 |
| pie | 554/554 |
| quadrantChart | 1,095/1,095 |
| radar | 1,264/1,264 |
| railroad | 2,347/2,347 |
| requirement | 1,480/1,480 |
| sankey | 1,710/1,710 |
| sequence | 4,200/4,200 |
| state | 4,443/4,443 |
| swimlane | 527/527 |
| timeline | 3,838/3,838 |
| treeView | 2,054/2,054 |
| treemap | 2,152/2,152 |
| usecase | 2,676/2,676 |
| venn | 1,958/1,958 |
| wardley | 1,359/1,359 |
| xychart | 1,383/1,383 |

基准缺口已补齐：Agentflow 文档 12 条、参考组 180 条成功与 59 条预期拒绝、state 文档 22 条、usecase 文档 24 条均通过对应行为验证。

## 4. Zebra 主题兼容要求（P1 起的必要验收项）

依据 [ZEBRA-THEME-COMPATIBILITY.md](ZEBRA-THEME-COMPATIBILITY.md) 与 `.research/zebra-theme-contract/` 中的 Swift/Rust/C ABI 参考源码。落实方式：

1. **数据契约**：库内 `MermaidPalette`（`background`、`surface`、`surfaceSecondary`、`text`、`secondaryText`、`accent`、`border`、`selection`、`warningBackground`、`warningText`、`dangerBackground`、`dangerText`、`successBackground`、`successText` 14 个必需；`edge`、`groupBackground`、`groupBorder`、`series` 可省略/null；`series` 恰好 6 项）。颜色只接受大写规范 `#RRGGBB`。库自带 JSON 读取（不依赖 serde），另在 `serde` feature 下提供 camelCase 序列化往返。
2. **两层默认**：Swift 构造器补齐（edge=secondaryText、groupBackground=background、groupBorder=border、series=[accent, successText, warningText, dangerText, text, secondaryText]）作为显式辅助函数；直接 JSON 省略可选字段时按 `renderer/src/theme.rs` 的原生回退（edge=text/background 0.5 分量混合；inner stroke=0.12 混合；series 缺省走 chart_palette/chart_series 派生）。两者不合并。palette 为 None 时按 light/dark 使用 `beautiful_light/beautiful_dark`。
3. **独立选项**：`theme: Light|Dark`、`palette: Option<MermaidPalette>`、`font_size`/`font_family`、`canvas_background: Opaque|Transparent`、`color_mode: Css|Resolved`，互不混用。
4. **具体颜色**：resolved 模式输出不含 `var()`、`color-mix()`、`light-dark()`、媒体查询；字重、圆角、等宽字体引用同样解析。
5. **语义角色贯穿图表**：渲染代码在每个着色点同时给出语义角色和 Pele 原样 CSS 表达式；css 模式写 Pele 原值（保持差分逐字节一致），resolved 模式写角色颜色。group 边框不并入节点边框，edge 不并入 accent。
6. **透明画布**：只省略根背景矩形（opaque 时为带 `data-mermaid-canvas-background="opaque"` 的整幅矩形），内部填充与对比度仍用 `palette.background`；尺寸不变。
7. **对比度**：饼图等图表标签按 `core/src/theme.rs::contrasting_text_on_fill` 算法移植（f32 计算）。Pele 切片不透明；旧 renderer 的 0.92 不透明度属于视觉策略，若不沿用须记录并验证最终对比结果。
8. **作者样式优先**：允许的 `style`/`classDef`/`linkStyle` 值保持优先，不做最终全局字符串替换。
9. **Zebra 安全 profile**：关闭 `<a>`、`<image>`，满足 Zebra `MermaidSVGSecurityPolicy`（无外部引用、无 `on*` 属性、无非 `#` href）。
10. **渲染身份**：提供包含完整规范化 palette、theme、字号、画布、颜色模式的身份键；颜色变化不复用旧 SVG；仅颜色变化时几何保持一致。
11. **target_width**：Zebra 兼容适配（含将来的 C ABI v2 适配）校验 `target_width`（1…100000）但不用于布局，保持 `renderingDependsOnTargetWidth = false` 的宿主缓存契约；通用 `max_width` 仅在非 Zebra 调用中生效。
12. **CLI**：`--theme light|dark`、`--color-mode resolved|css`、`--palette <json>`、`--canvas-background opaque|transparent`；原 `--theme resolved` 取消。

状态：1–12 的基础实现已完成。33 类的 702 条成功文档/参考输入已通过双调色板测试，确认 SVG 属性使用具体值、颜色变化不改变尺寸；新增三类另通过 4 项主题、画布及原生 SVG 策略测试。原生 App 内视觉与缓存验证尚未执行。C ABI v2 crate 与 C 头文件已建立，3 项 ABI 测试通过；库内适配覆盖参数校验、错误码与 target_width 不参与布局。完整 Zebra 集成（xcframework 替换、App 内主题切换与缓存测试）不在本项目单元测试范围内，不得冒充。

## 5. 已运行的验证（实际结果）

- `.research/all-33-verification.log`：全部 features 的工作区测试 1,096 通过、0 失败、2 跳过；包括 70,183 条采集全集对照和 13,194 条抽样对照。抽样是全集子集，不能相加。
- `.research/default-33-verification.log`：切换默认全类型后，工作区回归 1,100 通过、0 失败、2 跳过；包含后来增加的 4 项新图类型主题测试。
- `.research/state-full-verification.log`：state 4,443 条采集输入一致；22 条文档输入与主题矩阵通过。
- `.research/usecase-full-verification.log`：usecase 2,676 条采集输入一致；24 条文档输入与主题矩阵通过。
- 33 类主题矩阵共 702 条成功文档/参考输入，各用两套调色板验证具体属性和尺寸稳定性。
- 13 条源输入包含 Rust UTF-8 字符串无法表示的孤立 UTF-16 代理项，按精确记录清单排除，未算通过；抽样中有 6 条。该排除不是全量原始测试验收的豁免。
- 原始测试逐项移植：718 项有映射，716 通过、2 保留原跳过，6,092 未映射。ER 的 632 项有实际操作/断言的测试在原版和 Rust 两侧都通过，其中原跳过的 Unicode 测试已经启用。

性能验证已保存两种口径：

1. [Pele 原脚本口径](BENCHMARK-PELE-PROTOCOL.zh-CN.md)：原脚本直接执行，5 组输入和统计方式完全沿用；118 个输入完整 SVG 与尺寸一致。
2. [全图类型扩展基准](BENCHMARK.zh-CN.md)：33 类构建、39/39 组一致，共 820 个成功渲染和 59 个预期拒绝；成功渲染与拒绝处理分别计时，三轮独立进程交替顺序。

以上都不是“6,810 项原测试已全部移植”。上游原始基线为 6,705 通过、103 跳过、2 todo（包含网站测试）；另发现两个显示 passed 的 ER 测试体只有 TODO 注释，未生成空测试凑数。

## 6. 已知差异与决定

| 项 | 说明 |
| --- | --- |
| info 文本 | 新的原测试移植要求暴露默认行为差异，现已修正为固定参照的 `Pele v1.6.0`；`RenderOptions::info_text` 仍可由宿主显式指定。CLI 自身版本不变。 |
| `%%{init}%%` JSON 深度 | 上限 200 层，超过报 `limit`。JS 的 `JSON.parse` 无固定上限，Pele 在 merge 递归耗尽栈时才失败；Rust 使用线程栈递归，需显式限制。 |
| 孤立代理项 | JSON/YAML 转义产生的孤立 UTF-16 代理项无法存入 Rust 字符串，替换为 U+FFFD；成对代理项正常合并。 |
| `TextMeasurer` | 采用 `&self`（实现可用内部可变缓存），与方案草案的 `&mut self` 不同：测量在布局深处被多处借用调用。 |
| 默认 feature | `implemented` 已指向全部 33 类；`all-diagrams` 可编译并通过当前回归。精简构建必须显式禁用默认 features。 |
| 数学函数 | sin/cos/tan/asin/acos/atan/atan2/exp/log/log2/log10 移植 Node v26.3.0 所带 V8 14.6.202.34 的 fdlibm（`util/fdlibm.rs`）；Node 的 clang 构建会把 `a*b+c` 收缩为 FMA，移植在 clang IR 显示融合的 108 处使用 `mul_add`，因此这些函数在所有平台结果一致，并与 Node 逐位一致（每函数 54 万条随机/边界输入 + fixture 5,251 条）。`Math.hypot` 移植 V8 补偿求和。`Math.pow` 与 V8 一样调用平台 C 库 `pow`：已采集的差分结果与 Node 逐位一致，但**不保证跨平台复现**（Linux/wasm 的 libm 舍入不同）；Pele 仅在 sankey 收敛系数、xychart 刻度与 cynefin 平方处使用 pow，平方已按 V8 特例 `x*x` 处理。 |
| 饼图标签对比度 | Zebra 旧渲染器以 0.92 不透明度画切片；新引擎沿用 Pele 的不透明切片，对实际绘制色应用同一 `contrasting_text_on_fill` 算法。中明度样例 `#5E7E51`（底 `#FAE7C9`、字 `#322A20`）由旧的 `#000000` 变为 `#FFFFFF`（两者对比度都约 4.5:1）。 |
| Zebra 适配源长度上限 | 采用 Zebra 宿主的 262,144 UTF-16 单位（Pele 默认 50,000），避免拒绝 Zebra 现有可渲染的长图；输出上限按 Zebra 的 4 MiB UTF-8 字节。 |
| Zebra 字号 | Zebra 旧渲染器把标签字号缩放为 fontSize×13/16；新引擎按 Pele 直接使用 fontSize。属视觉策略变化，需在集成阶段确认。 |
| fixture 时区 | oracle 以 `TZ=UTC` 运行；缺省或非有限 `now` 归一化到 2026-10-19T00:00:00Z，保留归一化元信息；不把它计为非有限时钟原始行为覆盖。Gantt 当前仅提供固定偏移时区，完整 DST 策略尚未完成。 |

## 7. 方案修订记录

- 2026-10-19：按用户新增要求，主题输出拆为 `theme`（light/dark）与 `color_mode`（css/resolved），并加入 Zebra palette 契约（主方案已由用户同步更新）。

## 8. 下一步

1. 继续按 6,810 项全量测试清单逐项移植断言与修复实现；图类型本身已全部接通。
2. 收敛 Clippy 风格告警，完成跨平台构建、性能优化与发布验证。当前有部分类型慢于 Pele，不宣称全面更快。
3. 原生 Zebra App/xcframework 接入尚未执行；当前工作集中在本 Rust 库、CLI 与兼容适配层。

## 9. 主题范围澄清

用户补充：Pele CSS 主题若足以接入 Zebra，则无需增加 JSON 主题配置。不会要求用户另行维护 JSON 主题文件；现有 palette JSON 是 Zebra 已有的 Swift → Rust ABI 传输方式。

已沿当前 macOS 展示路径做最小实测（`NSImage(data:)`，见 `.research/native-theme-probe/`）：字面量 `#CC3311` 与 `currentColor` 正确；CSS `var()`、嵌套别名、变量 fallback 和 `color-mix()` 输出黑色。因此保留网页 Pele CSS 输出，以及原生具体颜色输出；原生颜色来自 Zebra 当前主题自动转换。该探针不代表完整 App 集成验收。
