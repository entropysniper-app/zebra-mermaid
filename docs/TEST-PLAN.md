# zebra-mermaid 测试计划

**新增硬性验收：Pele 全部测试必须 1:1 移植并通过。** 完整范围、逐项映射与失败门禁见 [UPSTREAM-TEST-PARITY.md](UPSTREAM-TEST-PARITY.md)。包括网站与浏览器测试，skip/todo 不计为通过；不能用输入采集、SVG 差分或下列单元测试清单代替。

本计划另区分三种补充兼容口径，并把 Zebra 主题契约列为每个阶段的必要验收项。“待做”项在实际运行通过前不得计为已验证；运行结果记录在 [IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md)。

## 1. 三种参照

| 口径 | 参照 | 判定 |
| --- | --- | --- |
| 解析兼容 | Pele 固定提交的解析器（其 vendored Mermaid 12.1.0 grammar/spec 已在上游测试中对齐） | 接受/拒绝一致；错误码、消息、行、列（UTF-16）一致 |
| 语义兼容 | Pele 模型的显式可序列化视图 | 字段、顺序、Map 顺序一致 |
| 布局兼容 | Pele `render()` 输出（Node，内置 Arial 估算测量器，固定 `now`/`idPrefix`/时区） | css 模式 SVG 逐字节一致；不一致时须逐例分析，不放宽为截图阈值 |

Rust 自身不变量：XML 可解析、坐标有限、重复渲染确定、多线程独立、资源超限可控、恶意输入不 panic。

## 2. 工具

- `tools/compat/oracle.mjs`（已实现）：用 `.research/pele/node_modules/esbuild` 打包固定 Pele 源码中的公开与内部函数（render、parse、preprocess、parseYaml、layoutLabel、sanitizeUrl、num、Math 函数），按 JSONL 批量输出期望值。
- `tests/fixtures/`：导出的期望值（含来源与许可说明）。语料来自 Pele `tests/corpus`（MIT，含 Mermaid 文档示例，Mermaid MIT）。
- Rust 测试读取 fixture 逐例比较，并按类型汇总通过率；已实现类型的任何差异必须列入已知差异清单并说明原因，否则测试失败。

## 3. 核心模块单元测试

| 模块 | 内容 | 状态 |
| --- | --- | --- |
| `util::js` | `String(n)`、`Math.round` 负半值、`toFixed` 精确半值、`Number()`/`parseFloat`/`parseInt`、UTF-16 长度与切片、JS 空白（U+FEFF 是、U+0085 否） | 已写 |
| `util::math` | sin/cos/atan2/pow/hypot 与 Node 逐位对照表 | 待做 |
| `util::url` | 危险协议、编码绕过、`&#` 十进制实体、WHATWG 规范化、宿主协议白名单、网络路径 | 已写，待与 Node 对照 |
| `config::yaml` | 标量类型、flow/block、块标量、折行、转义、深度 64、错误消息；与 Pele `parseYaml` 对照 | 已写单测，对照待做 |
| `config::json` | `JSON.parse` 语义、重复键、深度上限 | 已写 |
| `preprocess` | CRLF、HTML 属性引号、frontmatter（含缩进/无结尾换行）、指令、注释、实体编码；source map 回到原文字节位置 | 已写，待与 Node 对照 |
| `text::label` | `<br>`、标签、Markdown 强调、图标留位、中文/日文逐字换行、片段切分 | 已写，待与 Node 对照 |
| `error` | UTF-16 列号、snippet、字节 span（emoji/组合字符/中文/非 BMP） | 部分已写 |

## 4. 图表差分

每个类型：上游文档语料 + 上游测试中提取的输入 + 定向样例（四方向、环、自环、多重边、嵌套子图、跨组边、标签、样式、中文、恶意输入）。P2 起 flowchart 的 114 条文档示例全部比对。

## 5. Zebra 主题验收（自 P1 起，每个图表都要补）

依据 [ZEBRA-THEME-COMPATIBILITY.md](ZEBRA-THEME-COMPATIBILITY.md) 第 6 节。

### 5.1 数据契约（P1）

- [ ] 直接解码 Swift 形状的完整 palette JSON（`docs/research/zebra-theme-palette.example.json`）。
- [ ] camelCase 序列化往返（`serde` feature）与库自带 JSON 读取结果一致。
- [ ] 14 字段旧式 JSON：可选字段走原生回退，与 Swift 补齐结果不同（各写断言）。
- [ ] palette 为 None：light/dark 分别得到 beautiful_light/beautiful_dark 的颜色。
- [ ] 拒绝：小写 hex、`#FFF`、`rgba()`/CSS 表达式、series 非 6 项、缺失必需字段、重复字段、非字符串值；未知字段忽略（与 serde 默认一致）。

### 5.2 角色消费（每个图表）

- [ ] 用 17 个互不相同的哨兵色 + 6 个系列哨兵色渲染，按图表的“角色表”断言每个适用角色出现在**对应元素/属性**上（不是全文任意位置），并列出不适用角色。
- [ ] pie（P1）：切片=series（超过 6 项按原生扩展规则）、切片描边=background、标题/图例文字、切片内标签按对比度算法、外置标签。
- [ ] info（P1）：标题=text、版本文字=secondaryText、根背景。
- [ ] flowchart（P2）：节点填充/边框、edge、箭头=accent、groupBackground、groupBorder、分组标题、edge label 文字/底色独立生效。
- [ ] sequence（P3）：actor、生命线、note=warning、activation=selection/accent 等按当前语义生效。
- [ ] 其余图表随实现补表。

### 5.3 行为

- [ ] resolved 输出无 `var(`、`color-mix(`、`light-dark(`、`@media`、`@import`。
- [ ] opaque/transparent 只差根背景矩形；宽高与其余内容相同；transparent 下内部仍使用 background。
- [ ] 两个浅色 palette、浅↔深切换、单字段修改、反复切换：颜色不串用；仅颜色变化时几何（宽高、路径、坐标）完全一致；身份键随任一颜色、字号、画布、模式变化。
- [ ] 字号变化影响测量与身份键。
- [ ] 明暗饼图标签、中明度 fallback（参考快照 `renderer.rs` 的样例：`#FAE7C9`/`#322A20`/`#5E7E51` 等），并记录与旧 renderer 0.92 不透明度的差别。
- [ ] 显式 `style`/`classDef`/`linkStyle` 在 resolved 模式保持优先。
- [ ] Zebra profile 输出通过移植的 `MermaidSVGSecurityPolicy` 检查（无 `<a>`、`<image>`、外部引用、`on*`）。
- [ ] Zebra 适配对不同 `target_width` 产生相同 SVG。

### 5.4 不在本项目范围

Zebra App 内主题连续性、xcframework 替换和 App 缓存测试属于集成阶段，需要在 Zebra 工程中运行；本项目单元测试不冒充。

## 6. CLI

- [ ] 文件/stdin → SVG，`-o` 输出，stdout 只含 SVG，诊断走 stderr。
- [ ] `check` 与 `--format json` 错误（code、message、line、column、字节位置、源文件行列）。
- [ ] 退出码：0 成功、2 参数错误、3 不支持/语法/语义、4 资源限制、5 I/O。
- [ ] 输出用临时文件原子替换；默认拒绝覆盖，`--force` 允许。
- [ ] `batch`：保持相对目录、`--jobs`、输出冲突与输入输出目录重叠检查、汇总状态、任一失败非零。
- [ ] 主题参数：`--theme`、`--color-mode`、`--palette`、`--canvas-background`；非法 palette 报参数错误。

## 7. 资源与安全

- [ ] 源长度（UTF-16）、输出长度、边数上限；深度嵌套不导致栈溢出（每个递归点的显式深度限制）。
- [ ] 恶意标签/链接/样式（`url(`、`expression(`、`javascript:`）、组合边扩张。
- [ ] proptest：任意输入不 panic，输出 XML 可解析。

## 8. 性能

方案第 10 节：同一语料、同一测量器、同一选项对比 Pele 批次中位数；Criterion 定位回归。P3 后执行。
