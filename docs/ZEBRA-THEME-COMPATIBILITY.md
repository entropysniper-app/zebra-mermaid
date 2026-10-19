# Zebra 现有主题接入兼容契约

2026-10-19 新增用户要求：**必须支持当前 Zebra 接入 Mermaid 时配置主题的方式。** 当前目标仍为 Rust 库 + CLI；主题接口在基础层落实，图表实现逐个接入。

后续澄清：Pele CSS 主题若能直接用于 Zebra，就不增加 JSON 主题配置。实现应复用宿主现有主题，不要求用户维护另一份 JSON 文件。下文的 palette JSON 是 Zebra 已有 Swift → Rust ABI 的传输格式，示例文件仅用于协议验证与 CLI 调试。

已核对 macOS Zebra 的实际路径：`MarkdownMermaidPresentation.swift` 用 `NSImage(data:)` 解码返回的 SVG。该解码路径的最小验证表明，字面量颜色与 `currentColor` 正确；Pele 的 CSS `var()`（含别名、fallback）及 `color-mix()` 均显示为黑色。探针及输出保存在 `.research/native-theme-probe/`。因此网页可沿用 Pele CSS；当前原生适配仍需输出具体颜色，并复用 Zebra 已有主题转换及 ABI，不新建独立主题配置系统。完整 App 视觉与缓存验收仍待接入阶段执行。

状态：已核对现有源码并补充实施要求；本文不表示新引擎已经通过兼容测试。

## 1. 已核对的现有数据链

```text
Zebra 主题配置 / 自定义覆盖
→ ThemeCatalog / 已解析主题
→ semantic.mermaid
→ MermaidRenderPalette（Swift）
→ JSONEncoder，camelCase 字段
→ zebra_mermaid_render_svg（C ABI v2）
→ MermaidPalette（Rust）
→ 图表角色及具体 SVG 颜色
```

现有代码位置（相对于 Zebra 仓库根目录）：

- `Zebra/Editor/MermaidRenderContracts.swift`
- `Zebra/Editor/MermaidRendererService.swift`：macOS 调色板投影。
- `ZebraMobile/MobileMarkdownSVGView.swift`：iOS 调色板投影。
- `Zebra/Editor/NativeMermaidRendererService.swift`
- `Vendor/ZebraMermaidBridge/include/ZebraMermaidBridge.h`
- `Vendor/ZebraMermaidBridge/renderer/src/theme.rs`
- `Vendor/ZebraMermaidBridge/renderer/src/renderer.rs`

本次按当前工作文件采集参考快照至 `.research/zebra-theme-contract/`。其中 `manifest.json` 记录各文件完整来源 SHA-256、快照 SHA-256、片段行号和 Git HEAD `c21cba1b78ddaa38014f017430852f6b3a2fcf30`；工作文件可能包含未提交修改，以哈希对应的内容为准。参考快照不随本仓库发布。

## 2. 调色板数据格式

Swift 序列化包含 **17 个不透明颜色字符串和 6 元素 series 数组**。颜色为大写、规范的 `#RRGGBB`。保留现有 JSON 字段名：

| JSON 字段 | 用途 |
| --- | --- |
| `background` | 主题画布底色，同时参与内部填充、遮罩与标签对比度 |
| `surface` / `surfaceSecondary` | 节点主表面、次级表面/分组标题等 |
| `text` / `secondaryText` | 正文与次级文字 |
| `accent` | 强调角色；现有原生风格也用于箭头 |
| `border` | 节点等常规边框 |
| `selection` | 选中/激活填充 |
| `warningBackground` / `warningText` | 警告与注释等角色 |
| `dangerBackground` / `dangerText` | 危险/关键状态 |
| `successBackground` / `successText` | 成功/完成状态 |
| `edge` | 连线，与 accent、border 分别控制 |
| `groupBackground` / `groupBorder` | 分组填充与边框 |
| `series` | 6 个有序系列色，维持调用方指定的顺序 |

Rust 当前边界 `MermaidPalette` 使用 `#[serde(rename_all = "camelCase")]`；前 14 个标量字段必需，`edge`、`groupBackground`、`groupBorder` 和 `series` 为 Optional，可省略/null。提供了 series 就必须恰好 6 项。

必须区分两层默认规则：

1. **Swift 构造器**会补齐字段：缺省 edge＝secondaryText，groupBackground＝background，groupBorder＝border；series＝`[accent, successText, warningText, dangerText, text, secondaryText]`。正常 Zebra 请求已包含完整值。
2. **直接调用现有 Rust/JSON 接口**且省略可选字段时，遵循 `renderer/src/theme.rs` 中的默认派生；例如缺省 edge 是现有视觉策略按 text/background 混合的结果，series 缺省也有独立的派生序列。这与 Swift 先补齐的请求不同，不能合并成同一种隐式默认。

缺省整个 palette 时，现有 renderer 按 `Light/Dark` 选择 `beautiful_light/beautiful_dark` 默认调色板；提供 palette 时，显式语义色具有权威性。不要根据 Zebra 的主题名称另存一组内置映射。

[示例 palette JSON](research/zebra-theme-palette.example.json) 是根据现有默认字段构造的协议示例，不代表用户当前选中的主题。

## 3. 新库和 CLI 的接口要求

库应暴露平台无关的 palette 数据类型及类型化选项，保留与当前字段的一一对应关系，建议提供 `MermaidPalette`、`MermaidThemeMode`、`CanvasBackground`；可在现有 `RenderOptions` 中组合，避免另起一套不同语义的配色字段。

最少具备：

- `theme_mode: Light | Dark`
- `palette: Option<MermaidPalette>`，支持原有 JSON 直接反序列化与往返。
- `font_size` 和 `font_family`，与调色板分离。
- `canvas_background: Opaque | Transparent`，与调色板背景分离。
- `output_color_mode: Resolved | Css`，与明暗主题分离。

CLI 计划接口调整为：

```sh
zebra-mermaid render diagram.mmd --theme dark \
  --palette zebra-palette.json --color-mode resolved \
  --canvas-background transparent -o diagram.svg
```

主方案原先 `--theme resolved` 的命名与明暗主题有冲突，改为 `--color-mode resolved`。palette 文件直接承接现有 Zebra JSON，不要求用户把字段翻译成 `--pele-*`。

接入原生端的最终适配需支持现有 C ABI v2 参数形状：source 指针/长度、dark_theme、palette JSON 指针/长度、font_size、target_width、canvas_background 和 output result。无需本轮替换 Zebra 的 xcframework，但需要保留兼容适配路径及所有权/错误约定；未经 ABI 验证不能声称可以直接替换旧二进制。

## 4. 主题解析与 SVG 规则

1. **具体颜色输出是 Zebra 模式的必需能力。** 颜色和主题相关长度/字重等引用在输出前解析完成，不依赖浏览器去解释 CSS `var()`、`color-mix()`、`light-dark()` 或动态媒体查询。默认 Pele CSS 输出作为独立模式保留。
2. **完整语义层，而非只改几个全局变量。** Pele 的默认角色不足以一一表示 Zebra 的 group、selection、状态色与 series，需要 `ResolvedTheme` 的角色访问器贯穿各图表。不得把 groupBorder 合并成所有节点 border，或把 edge 合并成 accent。
3. **显式颜色使用原值。** 已有调用方提供的 edge、group、series 等不做新的审美修正；派生与回退以现有算法/样例为参照。现有默认颜色派生采用分量混合；本次兼容任务不擅自改为另一种色彩空间。
4. **RGBA 只在宿主投影时合成一次。** macOS 与 iOS 已将半透明 token 合成到最终画布，再传入不透明 `#RRGGBB`。Rust 不重新合成，也不将背景切换为透明色后再计算内部角色。
5. **透明选项仅控制根画布。** 省略整张图的背景绘制/根背景样式，保留 theme.background 供标签底、内部填充和对比度计算使用。相同输入由 opaque 切到 transparent，几何尺寸不应变化。
6. **系列色与可读标签。** 支持调用方六个系列色，并按图表需要沿用现有扩展规则。饼图明暗切片标签、半透明填充的最终底色、必要的黑白 fallback 使用当前 `core/src/theme.rs::contrasting_text_on_fill` 的算法作为兼容参照；不能统一强制白字/黑字。现有 renderer 里的特定不透明度属于视觉策略，重写若调整应明确记录并验证最终填充对比行为。
7. **作者样式优先级保留。** 显式且允许的 node/edge `style`、`classDef`、`linkStyle` 可覆盖对应默认角色；不应通过最终全局字符串替换抹掉这些值。恶意值仍走安全过滤。
8. **原生安全输出 profile。** 当前 Zebra SVG validator 禁止 `<a>`、`<image>`、脚本及外部资源等，Pele 的通用模式允许部分链接/图片。Zebra 兼容模式需关闭这些输出并走现有安全约束，不能直接使用通用默认值。

## 5. 缓存、重绘与宽度契约

当前 Swift `MarkdownMermaidAttachmentRenderVariant` 包含完整 `theme`、`palette`、1/64 point 字号和 `canvasBackground`。同属 light 的两个自定义主题也必须产生不同渲染身份；palette 中任何实际颜色变化都不能命中旧的最终 SVG。

Rust 可以复用与颜色无关的解析结果或几何，但必须重新应用当前 palette 生成 SVG。若引入渲染缓存，key 使用完整规范化 palette 和相关选项，而非主题名或 light/dark 布尔值。

当前 NativeMermaidRendererService 的 `renderingDependsOnTargetWidth = false`：ABI v2 校验 target_width，但生成固有 SVG，原生视口负责适配。新引擎的通用 maxWidth/autoDirection 可保留；**Zebra 兼容适配不能悄悄启用随 target_width 变化的布局**，否则与宿主缓存身份冲突。保持旧适配行为，或将来在明确的集成变更中同步调整宿主缓存与测试。

## 6. 必须增加的验收用例

这些是待实施测试，不能因为写了本文而标记通过：

- 直接解码 Swift 形状的完整 palette JSON；camelCase 序列化往返；14 字段旧式 JSON 的 Optional 回退；palette 为 None 时 light/dark 默认行为。
- 拒绝非规范颜色、低位数 hex、小写 hex、rgba/CSS 表达式、非 6 元素 series 和缺失必需字段。未知字段处理遵循现有 serde 边界，不额外声称旧版使用 deny_unknown_fields。
- 用彼此独立、容易识别的哨兵色检验所有 17 个标量角色及 6 个系列色的实际消费；按图表类型列出不适用角色，不能只断言 SVG 随便含某个颜色字符串。
- 先在 pie/info 的首版闭环中落实主题选项，再对 flowchart/sequence 及每个后续图表补相应主题角色用例，避免最后整体补色。
- flowchart 的 edge、groupBackground、groupBorder、节点填充、箭头/强调独立生效；sequence 的 note/activation 等角色按当前语义生效。
- 两个浅色 palette、浅深切换、单字段修改和反复切换不复用错误颜色；仅颜色变化保持对应的布局结果稳定。
- opaque/transparent 输出差异仅限根画布，内部背景和对比度行为保持；字号变化应影响测量与正确的缓存身份。
- 明暗饼图标签、半透明切片、现有中明度 fallback 用例；样例源码可在参考快照 `renderer/src/renderer.rs` 的测试中找到。
- 显式节点/边样式保持优先级；resolved 输出无 CSS 运行时依赖，适用于原生 SVG 消费者。
- 集成阶段使用 Zebra 现有主题连续性、缓存与 SVG 安全测试验收；本项目的单元测试不冒充真实 Zebra App 验证。

## 7. 完成判定

只有当 palette 数据契约、默认规则、具体颜色输出与阶段内图表角色测试实际通过，才能把该阶段标为“支持 Zebra 主题”。完整集成仍需 ABI/原生消费与主题切换验证。Claude 会话已按用户要求停止，后续由 Codex 执行，代码修改集中在 zebra-mermaid 项目。
