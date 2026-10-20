# zebra-mermaid 实施状态

本文件记录 Rust 重写的实际进度，供中断后恢复执行。只记录已执行并有证据的结果；计划项明确标为“待做”。

**验收标准：Pele 全部测试 1:1 移植且通过。** 已按用户要求将基线升级到 Pele 1.7.0，完整清单为 292 个文件、6,826 项；原有 2,770 项映射全部保留，并新增 16 项 `mermaidType` 与 6 项 CJK 换行映射。逐项独立进程超时验收为 2,785 通过、0 失败、7 保留上游跳过、4,034 未映射，正式通过率 40.80%。两个原 skip 测试的实际执行通过仍计为验收跳过。完整验收未完成，详情见 [UPSTREAM-TEST-PARITY.md](UPSTREAM-TEST-PARITY.md)。图类型覆盖为 33/33，本轮 1.7.0 文档完整响应对照为 761/761，原始测试通过率另行统计。

- 最后更新：2026-10-20（新增[编译产物体积目标与发布基线](CODE-SIZE-OPTIMIZATION.md)；最新已发布性能仍为 [SVG 适配器轮次](SVG-ADAPTER-OPTIMIZATION.md)）。首页仅展示当前结果，另列 [API/UTF-8 两种输出口径](OUTPUT-BENCHMARK.md)。
- 方案：[RUST-REWRITE-PLAN.zh-CN.md](RUST-REWRITE-PLAN.zh-CN.md)
- Zebra 主题契约（必要验收项，自 P1 起）：[ZEBRA-THEME-COMPATIBILITY.md](ZEBRA-THEME-COMPATIBILITY.md)
- 测试计划：[TEST-PLAN.md](TEST-PLAN.md)
- 上游参照：Pele `8603bf3313add9143cd417015d4a6bd6c522dada`（package 1.7.0，Mermaid 语法目标 12.1.0），快照在 `.research/pele/`（不入库）
- Zebra 主题参考快照：`.research/zebra-theme-contract/`（只读；Zebra 原工程不修改）

## 1. 目标

性能门槛已按用户最新目标调整为 **2×**；下文历史轮次保留当时 10× 目标的测量记录，不能将历史目标当作现行门槛。编译产物体积优化继续独立跟踪。

1. 纯 Rust 库 + CLI，运行时不调用 Node/JS/外部布局程序；Node 只作开发期 oracle。
2. 按 P0–P6 覆盖上游全部 33 类图表；移植 Pele 的手写解析器、YAML 子集、文本估算与 layered/compound/lanes 布局，保持语义与几何。
3. 两种颜色输出：`css`（Pele 原样 `--pele-*` 变量，供差分与网页嵌入）和 `resolved`（具体颜色，CLI 默认与 Zebra 原生接入模式）。
4. 支持 Zebra 现有主题接入方式：camelCase `MermaidPalette`（17 个颜色 + 6 系列色）、light/dark、fontSize、opaque/transparent 画布；语义角色贯穿各图表。
5. MVP（flowchart/sequence/pie/info）只是中间里程碑，不得报告为完整重写。
6. 全部 167 个原始样本、33 类图表的完整解析、布局与 SVG 生成，在 API 返回口径下相对固定 Pele 1.7.0 的逐类型等权几何平均达到 **2×**；UTF-8 就绪口径、逐类型分布、39 组扩展负载及全部回退分别报告，当前尚未达标。
7. **新增编译产物体积优化目标**：在相同工具链、锁定依赖、release 配置与完整 33 类功能下，独立跟踪发布 CLI、FFI 动态库和静态库的文件字节数与实际代码段字节数；库归档、基准驱动及符号/调试信息裁剪单列。保留体积与速度、分配、RSS 的取舍，不以关闭图类型、功能或改变工作量达标。用户未指定体积数值门槛；首轮发布产物基线已建立，现有驱动文件大小不冒充发布产物代码体积。

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

以下保留升级前 `44da152b` 的 33 类采集全集结果，本轮未重跑全集，均使用固定 Pele 提交、固定测量与时间参数；比较 SVG、尺寸、链接和错误。数量包含与上游一致的成功渲染及预期错误，不等于全部输入都应成功渲染。主题输出另外验证。

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

新增体积目标（基线 `4a03a25`，生产代码与 `e0a9e60` 相同）：已从固定提交的独立源码快照构建完整 33 类 CLI/FFI，使用锁定依赖、release/thin LTO/codegen-units=1 和一致的源码路径映射；未 strip。文件 / 指令 text 分别为 **CLI 5,210,752 / 3,853,944 bytes**、**FFI 动态库 4,798,544 / 3,561,420 bytes**、**FFI 静态库 41,708,256 / 4,273,968 bytes**。静态库另含 26,008,880 bytes 的 LLVM bitcode，归档大小不等同于最终宿主链接增量。本次是新增目标与基线，未声称已缩小代码；下一优化轮次将独立报告体积、速度与内存取舍。[方法与复现](CODE-SIZE-OPTIMIZATION.md)、[源码/产物指纹与逐段大小](research/code-size-baseline-2026-10-20/summary.json)、[单列的驱动体积](research/code-size-baseline-2026-10-20/drivers.json)。

scanner / 交叉计数实验（基线 `4a03a25`，候选未作为生产优化提交）：全 167 Sample / 33 类配对耗时 **0.99953（−0.05%，基本持平）**，16 类下降、17 类回退；33 类文档 render **+0.06%**、parse **−0.08%**。三种发布产物代码段均增加：CLI **+1,524**、FFI 动态库 **+1,516**、静态库 **+1,440 bytes**。12 类分配次数减少不替代现行 2× 速度目标；Sample RSS 23 降/1 平/9 增，扩展 19 降/4 平/16 增，大 mesh **+352 KiB**。候选工作区 **3,230 通过、0 失败、5 ignored**，761/167/39 对照全一致，原始验收仍 2,785 通过、7 跳过、4,034 未映射。完整参数、回退、原始轮次、体积与内存证据保存在[实验报告](SCANNER-CROSSING-EXPERIMENT.md)；四个源文件的精确候选 patch 亦保留。下一候选改用更简单的索引式 scanner 存储，交叉计数继续独立评估，不将本实验计为整体提速。

最新 SVG 适配器轮次（基线 `2b1fda6`，代码提交 `e0a9e60`）：**主集合耗时下降 3.25%，存在较大扩展回退，10× 未达到**。

- 根据 Info/C4/Block/Sequence/Swimlane 新采样，内部居中/边缘/左对齐适配器改为借用标签或视图，公共辅助函数签名、公开适配器字段、模型所有权保持不变。当前构建 LabelView 72 bytes、引用 8 bytes；居中/视图适配器 48、边缘 80、左对齐 40 bytes。仅为目标构建诊断，不作为 ABI 或独立速度结论。解析、测字、浮点运算、完整 SVG 与限额均保留，不缓存标签/模型/输出。
- Sequence 转换 **16 处构造 + 25 处追加**，另有 **21 个渲染文件、47 处**已知标签/箭头追加改为直接 Part 写入。参数求值、临时值和释放顺序由现有拼接契约保持；格式标记与非 SVG 构造保留标准格式化。新增两项实际路径测试读取未修改的独立 Node 预期：6,048 组居中标签（含有/无图标 resolver 的两种 Part）、444 组左对齐（公开/内部标签存储）；不增加原测试验收映射。
- 全部 167 Sample / 33 类三轮交替配对 render after/before **0.96753（-3.25%）**；26 类观测中位数下降、7 类回退：**Pie +7.92%、State +4.97%、Packet +2.72%、Flowchart +2.31%、Railroad +2.31%、Swimlane +1.70%、Radar +0.39%**。Class -15.55%、Block -15.36%、C4 -14.10%、Git -14.01%、Ishikawa -9.66%、Sequence -7.23%。33 类文档 render **0.98471（-1.53%）**、parse **0.99252（-0.75%，解析未改）**，Sequence 文档 render -9.99%。不作小幅差异显著性声明，不把子集替代主指标。
- 39 组扩展全部保留：全 114 个 Flowchart 文档批次 **+30.73%**，50-node tree +10.04%、小 labelled mesh +7.41%、500-node tree +7.03%、大 mesh +18.65%；Venn 文档 +20.02%、Timeline +13.35%、Treemap +9.57%、Flowchart +8.09%、Wardley +7.93%。部分未修改的解析路径亦波动，各进程有离群值；未删除或用复测替换任何结果，未确定全部回退的根因，后续需单独核对。
- 当前当轮 Rust/Pele 等权加速 **API 1.79×、UTF-8 1.89×**；API 范围 **Agentflow 1.21×…Sankey 2.86×**，23 类低于 2×、10 类至少 2×、0 类达到 10×。Node 每轮重测，不跨轮比值推导本轮增益；四引擎共同成功子集 110 Sample / 22 类独立报告。
- 分配次数 Sample 全 33 类、扩展全 39 组不变。只有 Sequence 的扩容、累计申请与额外存活峰值下降：Sample 扩容 **501 → 433**、申请 **390,194 → 363,097 bytes**，文档扩容 **2,652 → 2,291**、申请 **1,661,097 → 1,508,089 bytes**，两者峰值 **36,631 → 36,487 bytes**；其余 32 类/38 组完全不变。批后净保留均为 0，不作通用泄漏声明。
- 另测真实 OS RSS/CPU：每组每版本 3 个新进程、交替顺序、5 批预热、至少 1 秒完整 render 循环，包含启动/输入清单加载/预热。33 Sample RSS 中位数 **30 降/1 平/2 增**，39 扩展组 **34 降/1 平/4 增**；大 labelled mesh 峰值 **11,386,880 → 11,960,320 bytes（+560 KiB）**；不以申请字节代替 RSS、不跨进程相加峰值，完整进程记录与回退均保留。
- 跨引擎驱动 **9,362,432 → 9,363,088 bytes**，扩展 **5,068,128 → 5,068,880 bytes**，CPU **5,014,832 → 5,015,584 bytes**，分配器 **5,532,336 → 5,516,912 bytes**。构建参数不变；计时与构建、测试、CPU 采样、分配和 RSS 测量分离。
- 完整工作区 **3,226 通过、0 失败、5 ignored**；无默认 feature 4 项通过；Clippy 警告类别和数量不变。761/761 完整响应、167/167 SVG、39/39 扩展行为一致。原始独立门禁 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，严格完整门禁退出 1、`complete: false`。原测试正文、fixture 预期、断言、种子、次数、阈值与映射未改。
- [报告](SVG-ADAPTER-OPTIMIZATION.md)、[全部测量证据](research/svg-adapter-optimization-2026-10-20/summary.json)、[验证](research/svg-adapter-optimization-2026-10-20/validation.json)与[当前终态性能表](OUTPUT-BENCHMARK.md)已更新。源码、输入和二进制指纹再次核对；消费者远程安装锁定 `e0a9e60`。
- 下一轮重点复核 Flowchart/压力负载与视图按引用传递的代价，并检查尚未改动的 scanner 工作数组。保留全 33 类主目标与全部原门禁；完整原测试迁移仍未完成。

紧凑 token / 静态语法历史轮次（基线 `36631d1`，代码提交 `35db1ee`）：**主集合原生耗时下降 1.23%，存在回退，10× 未达到**。

- 新采样确认 Info/Architecture/Packet/Treemap/Railroad 的公共词法器开销；12 类、15 套固定语法采用一次准备的首字符候选表和 12-byte 内部 token。公开 TokenType/Pattern/Scanner/Tokens/Reader API 与所有权保持不变，公开动态表继续原有查找；两种存储共用匹配循环。超过紧凑边界仍用原生宽整数表示，不新增输入上限。候选表仅保存规则元数据，每次仍完整扫描输入、解析、布局和生成 SVG，不缓存输入/token/模型/输出。原有 boxed scanner 状态尚未替换。
- 独立固定 Node 参照 **2,008 组、15 表**：934 成功 token 流、1,074 预期词法拒绝、2,153 Reader 错误，含每表 96 次种子 `0x70c3a9` 变异。公开向量、内部紧凑存储、强制宽整数回退三条路径比较 kind/EOF/文本/UTF-16 位置和完整错误字段。新增 15 个实际表测试及 2 个整数边界测试，不增加原测试验收映射。
- 全部 167 个 Sample / 33 类三轮交替配对 render after/before **0.98768（-1.23%）**；21 类观测中位数下降、12 类增加。全部 Sample 回退：**Treemap +2.21%、Ishikawa +2.09%、XYChart +1.32%、Venn +0.61%、State +0.56%、Journey +0.51%、Usecase +0.41%、C4 +0.41%、Timeline +0.26%、Sequence +0.20%、ER +0.08%、Gantt +0.01%**。33 类文档 render **0.98108（-1.89%）**、parse **0.96773（-3.23%）**；受影响的 12 类文档 parse 全部下降、几何均值 **0.90408（-9.59%）**，该子集不替代全 33 类主指标。Info 文档 parse -28.77%、Sample render -10.53%。全部 39 组和单轮值保留，不将其他 21 类的变化归因为其解析算法，也不作小幅变化的显著性声明。
- 当前当轮 Rust/Pele 等权加速 **API 1.73×、UTF-8 1.83×**；API 分布 **Agentflow 1.21×…Sankey 2.86×**，25 类低于 2×、8 类至少 2×、0 类达到 10×。Node 每轮重测，不用跨轮绝对加速比相除推导本轮收益。四引擎共同成功子集仍为 110 Sample / 22 类，单独复测。
- 分配器测量：Sample 的分配、扩容、累计申请均 **12 类下降、21 类不变**，额外存活峰值 33 类不变；39 扩展组三项均 12 降/27 平，峰值全不变。Info 分配 **36 → 32**，Packet 扩容 **122 → 92**、申请 **99,693 → 93,741 bytes**；Wardley 申请 **295,698 → 278,322 bytes**。全部批后净保留为 0，不作通用泄漏结论。
- 另行测量真实 OS 进程峰值 RSS 和 CPU：每组每版本 3 个新进程、交替顺序、5 批预热、至少 1 秒完整 render 循环，包含启动/输入清单加载/预热。33 Sample RSS 中位数 **8 降/0 平/25 增**；39 扩展组 **11 降/4 平/24 增**。不支持整体 RSS 下降结论，不用申请字节代替 RSS，也不跨独立进程相加峰值；完整每进程结果与回退保留。
- 驱动体积：跨引擎 **9,360,112 → 9,362,432 bytes（+0.025%）**，扩展 **5,066,064 → 5,068,128 bytes（+0.041%）**；CPU/分配器驱动体积与全部指纹另存。
- 完整工作区 **3,224 通过、0 失败、5 ignored**；无默认 feature 4 项通过；Clippy 警告类别和数量不变。761/761 完整响应、167/167 SVG、39/39 扩展行为一致；原始独立超时门禁 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，严格完整门禁退出 1、`complete: false`。原测试正文、fixture 预期、断言、种子、次数、阈值和映射不变。
- 测量与构建/测试/CPU 采样/分配/RSS 探针分离，末尾核对源码、输入、二进制指纹。[报告](COMPACT-TOKEN-OPTIMIZATION.md)、[原始证据](research/compact-token-optimization-2026-10-20/summary.json)、[验证](research/compact-token-optimization-2026-10-20/validation.json)与[当前终态性能表](OUTPUT-BENCHMARK.md)已更新；消费者远程安装锁定 `35db1ee`。
- 下一轮依据新的调用栈检查剩余 scanner 工作数组、token 写入调用及共享 SVG 路径，继续以全部 33 类为主集合，保留 Treemap 等回退。完整原测试迁移仍未完成。

普通标签内联历史轮次（基线 `b202b94`，代码提交 `f1c5a83`）：**主 Sample 集合耗时下降 4.49%，存在类型回退，10× 未达到**。

- 根据 Info/C4/Mindmap/Block/Sequence 的新采样，将渲染内部普通单 span 标签的三层容器分配改为内联存储；复杂标签继续保留原有 Vec，公开 `Label` 字段与所有权不变。两种表示共用布局算法和 SVG view，保留测量回调参数、顺序、浮点运算及完整渲染工作量，不使用标签/模型/输出缓存。结构变大带来的集合内存代价完整保留。
- 新增独立固定 Node 参照：**6,048 组标签、18,144 组 Requirement restyle、4,108 个舍入值**，精确比较 binary64、完整测量轨迹与有/无图标 resolver 的 SVG。该参照在修复前证明旧公开 Label 对负字号空白输入返回 +0，而 Node 返回 -0；修正 `Math.round(-0.1)` 等保留负零的语义。保留旧源码指纹与失败事实，不改原 fixture 预期。3 项补充测试不计入原测试映射。
- 全部 167 个 Sample / 33 类三轮交替配对 render after/before **0.95511**（耗时 -4.49%），27 类观测中位数下降、6 类增加。全部 Sample 回退：**Treeview +4.44%、Eventmodeling +1.03%、Wardley +0.42%、Packet +0.34%、Agentflow +0.14%、Timeline +0.06%**。33 类文档 render **0.98839**（-1.16%），24 类下降；Journey +12.70%、C4 +9.64%、Flowchart +5.16%，全 Flowchart 文档批次 +9.29%。parse **1.00351**（+0.35%，解析算法未变）。全部 39 组与单轮数据保留，不作小幅变化显著性的声明。
- 当前主集合 Rust/Pele 等权加速 **API 1.74×、UTF-8 1.84×**；API 分布 **Agentflow 1.22×…Sankey 3.26×**，26 类低于 2×、7 类至少 2×、0 类达到 10×。四引擎共同成功的 110 个 Sample / 22 类单独复测，不代替全类型主指标。
- Sample 分配次数 **30 类下降、3 类不变**；扩容 33 类不变；累计申请 26 类下降、2 类不变、5 类增加；额外存活峰值 28 类下降、4 类不变、1 类增加。C4 分配 **4,972 → 4,348**、Sequence **1,132 → 988**、Info **42 → 36**。Treeview 申请 **159,650 → 163,490 bytes**、峰值 **16,764 → 17,148 bytes**；Block/Kanban/State/Agentflow 的申请增加亦完整记录。39 扩展组分配 36 降/3 平，申请 31 降/3 平/5 增，峰值 33 降/5 平/1 增；批后净保留全为 0。分配器字节不代表 RSS，不作通用泄漏声明。
- 二进制代价：跨引擎驱动 **9,310,832 → 9,360,112 bytes（+0.53%）**，扩展驱动 **5,017,008 → 5,066,064 bytes（+0.98%）**，其他两个驱动各约增加 49 KB。
- 完整工作区 **3,207 通过、0 失败、5 ignored**；无默认 feature 4 项通过；Clippy 完成，无新增警告类别。761/761 完整响应、167/167 SVG、39/39 扩展行为一致。计时与构建、测试、CPU/分配采样分离，末尾核对测量源码与二进制指纹。
- 原始独立超时门禁 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，严格完整门禁退出 1、`complete: false`，完整原测试验收未完成。原测试正文、预期、断言、种子、次数、阈值和映射未修改。[报告](COMPACT-LABEL-OPTIMIZATION.md)、[原始证据](research/compact-label-optimization-2026-10-20/summary.json)、[验证](research/compact-label-optimization-2026-10-20/validation.json)与[当前终态性能](OUTPUT-BENCHMARK.md)已更新，远程 Git 安装锁定 `f1c5a83`。
- 下一步仍需以新 profiling 确认瓶颈，处理 Treeview 等类型的结构体体积回退，并评估临时集合/输出生命周期；未完成的 Sequence/Swimlane 标准格式化保留待办。Ondine 的 Wasm arena 仅作为研究方向，尚未集成，不将其预编译、等宽测量或共同成功子集计入本项目 10× 目标。

SVG 拼接历史轮次（基线 `0a3d676`，代码提交 `9f1eefa`）：**全部 33 类 Sample 中位数下降，主目标仍未达到**。

- 根据 C4/Agentflow/Info/Block/Sequence 的新调用栈，将已知字符串、坐标、标签与箭头的输出改为私有静态分发；38 个源文件中的 340 处格式化和 298 处追加调用已转换，另外 12 处候选调用保留标准库实现。参数先求值再写出，临时值、释放顺序、数值格式、公开 SVG API、解析与布局工作量保持不变；不使用图表/标签/输出缓存。Sequence、Swimlane 仍有较多标准 SVG 格式化，未声称已全部替换。
- 全部 167 个 Sample / 33 类三轮交替配对 render after/before **0.91387**（耗时 -8.61%），**33/33 类**观测中位数下降。33 类文档 render **0.91846**（-8.15%），32 类下降，Venn +0.012%；parse **0.99490**（-0.51%，解析算法未改，不将小幅差异归因为解析优化）。Timeline 文档 -18.48%、Mindmap -15.26%、Journey -14.74%；全部回退和每轮数据保留，不作小幅差异的统计显著性声明。
- 当前主集合 Rust/Pele 等权加速 **API 1.67×、UTF-8 1.76×**；API 分布 **Info 1.18×…Sankey 2.79×**，27 类低于 2×、6 类至少 2×、0 类达到 10×。四引擎共同可比子集仍为 110 个 Sample / 22 类，独立报告。
- Sample 分配次数 **33 类全不变**；扩容 28 类下降、5 类不变；累计申请 30 类下降、1 类不变、2 类增加；额外存活峰值 10 类下降、21 类不变、2 类增加。Timeline 扩容 **807 → 527**，但累计申请 **389,754 → 426,621 bytes**、峰值 **29,217 → 34,399 bytes**；Kanban 申请 **133,658 → 135,474 bytes**，Gantt 峰值增加 3 bytes。39 组扩展数据亦完整保留；批后净保留全为 0，不用请求字节替代 RSS，也不作通用泄漏结论。
- 静态展开的体积代价：跨引擎驱动 **9,128,416 → 9,310,832 bytes（+2.00%）**，扩展驱动 **4,833,424 → 5,017,008 bytes（+3.80%）**；其余两个驱动约 +3.8%。所有构建使用相同 release/thin LTO/codegen-units=1 设置。
- 完整工作区 **3,204 通过、0 失败、5 ignored**；无默认 feature 4 项通过，Clippy 完成且无新增警告类别。761/761 完整响应、167/167 SVG、39/39 扩展行为一致，测量后源码与二进制指纹再次核对。新增 2 项项目测试验证拼接契约，不增加上游映射。
- 原始超时门禁 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**；严格完整门禁退出 1、`complete: false`。原测试正文、fixture、断言、种子、次数、门槛与映射不变。[本轮报告](SVG-CONCAT-OPTIMIZATION.md)、[验证证据](research/svg-concat-optimization-2026-10-20/validation.json)和[完整终态性能表](OUTPUT-BENCHMARK.md)已更新；消费者远程安装锁定已验证的 `9f1eefa`。
- 下一轮：当前所有 Sample 分配次数仍未减少，优先根据余下 CPU/分配证据检查共享标签与小集合的多次堆分配，继续完整 33 类验证；未完成的 Sequence/Swimlane 格式化也保留在待办范围。

literal label 历史轮次（基线 `2e81b44`，代码提交 `062c752`）：**C4 局部受益，全类型整体基本持平，10× 未达到**。

- 根据 C4 左对齐 SVG 调用栈去除临时文本/标签缓冲区，C4/Treeview 追加到已有输出，Eventmodeling 直接格式化。literal label 的试探前缀借用源切片；字符换行复用当前字符串，保留原测量字符串、参数、顺序与浮点运算。独立 Node 参照的 1,008 组逐字换行、1,184 组字符换行及 444 组 SVG 全部一致，含完整回调轨迹和精确 binary64 值；3 项补充项目测试不计入上游映射。
- 完整工作区 **3,202 通过、0 失败、5 ignored**；无默认 feature 回归 4 项通过，Clippy 完成并保留既存警告。761/761 完整响应、167/167 SVG、39/39 扩展负载行为一致，源码与二进制指纹在测量后再次核对。
- 三轮交替计时：33 类文档 render after/before **0.99536**（-0.46%），parse **0.98770**（-1.23%；解析源码未变，不将其归因为本轮渲染修改）；22/33 类文档 render 中位数较低。全部 167 个 Sample render **1.00112**（+0.11%），仅 9/33 类中位数较低；完整保留 Timeline 文档 +1.54%、XYChart Sample +1.25% 等回退，不作统计显著性声明。
- C4 文档 render **-3.45%**、Sample **-2.79%**。六输入 Sample 批次分配 **5,111 → 4,972**、扩容 **1,937 → 1,844**、累计申请 **1,202,099 → 1,187,061 bytes**，额外存活峰值 **112,833 bytes 不变**。其余 32 类 Sample、38 组扩展的分配数据不变；可选窄布局的两个换行分支不声称已获代表性吞吐收益。批后净保留均为 0，不等同于 RSS 或通用泄漏验证。
- 当前全部 167 个 Sample / 33 类 Rust/Pele 等权加速 **API 1.52×、UTF-8 1.61×**；API 分布 **Info 1.13×…Sankey 2.37×**，29 类低于 2×、4 类至少 2×、0 类达到 10×。四引擎共同成功子集 110 个 Sample / 22 类另行复测，不替代主目标集合。
- 原始超时门禁仍为 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，`complete: false`、退出 1。原测试、fixture 与映射未改。[本轮报告](LITERAL-LABEL-OPTIMIZATION.md)和[验证证据](research/literal-label-optimization-2026-10-20/validation.json)已保存；README 仅显示终态数据，消费者远程依赖锁定已验证的 `062c752`。
- 后续共享 SVG 拼接轮次已完成，见上方最新记录。

文本测量实验历史记录（基线 `377cfc8`）：**候选实现已撤回，不计为优化收益**。

- 根据 C4/Kanban 调用栈尝试整数 advance 与增量前缀测宽；33 类文档 render after/before **1.00253**（+0.25%）、parse **1.00807**（+0.81%），全部 167 个 Sample render **1.00677**（+0.68%）。未观察到整体加速，不保留额外运行分支与状态。
- 候选工作区 3,201 通过、0 失败、5 ignored；761/761 完整响应、167/167 SVG、39/39 扩展组一致。33 类 Sample 和 39 组扩展的分配次数、扩容、累计申请量及额外存活峰值全部不变；批后净保留全部为 0，不作 RSS 或通用泄漏结论。
- 保留来自固定干净 Node 上游的 **309,890 项精确 binary64 测宽对照**（4,427 个有效 Rust 字符串、10 种样式、7 个字号，固定种子 `0x51a710`）；这 1 项补充测试不计入原测试映射。撤回候选后完整工作区 **3,199 通过、0 失败、5 ignored**；生产源码和依赖指纹与基线相同，当前有效性能仍为 API **1.52×**、UTF-8 **1.62×**，10× 未达到。
- 原始超时门禁仍为 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**；完整验收未完成。候选 patch、三轮采样、分配、CPU 栈聚合及验证结果均保留在[实验报告](TEXT-METRICS-EXPERIMENT.md)和[原始证据](research/text-metrics-experiment-2026-10-20/summary.json)中。
- 后续 literal label 轮次已完成，见上方最新记录。

已保留的十进制转换轮次（基线 `97b7488`，代码提交 `87baab2`）：

- 最短十进制输出使用锁定的纯 Rust `zmij 1.0.23`；精度格式化使用栈缓冲区并预留最终字符串容量。固定/有效数字的中点判断改用精确的二进制有效数计算，修复旧实现把高精度候选转回 f64 后无法区分中点、以及负小数位舍入的问题。
- 新增独立 Node 参照：6,905 个值、48,335 个有效数字输出、34,525 个固定小数位输出，共 **89,765** 项比对，包含 4,096 个固定种子随机位模式、十进制指数邻居、2 的幂及 100 位精度。原有 fixture 和断言未改；3 项补充项目测试不计入上游原测试映射。
- 工作区 **3,198 通过、0 失败、5 ignored**；无默认 feature 回归 4 项通过。761/761 完整响应、167/167 SVG、39/39 扩展负载行为一致。Clippy 可完成，保留既有警告。
- 三轮交替测量的 33 类文档 render 等权 after/before 为 **0.99778**（-0.22%），parse 为 **0.99957**（-0.04%）；16/33 类观察到较低 render 中位数。Sankey render -5.22%、Treemap -3.70%；整体基本持平，不宣称普遍加速。
- 167 个 Sample 的配对 render 比值 **1.00703**（+0.70%），当前 Rust/Pele **API 1.52×、UTF-8 1.62×**。10× 主目标仍未达到。全部回退行、三轮采样和二进制指纹保留在报告；不同样本集合不混合计算。
- 分配次数和累计申请量在 4/33 类样本下降，29 类不变，无增加。Sankey 分配 **1,302 → 1,044**，Treemap **923 → 851**；1 类额外存活峰值下降、32 类不变。全部样本和扩展负载测量批后净保留 0，不等同于通用泄漏检测或 RSS。
- 原始超时门禁仍为 **2,785 通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**；`complete: false`、退出 1。原测试输入、断言、种子、循环次数、阈值与映射不变。[完整验证记录](research/decimal-formatting-optimization-2026-10-20/validation.json)。
- 后续文本测宽实验已完成并因无整体收益撤回，见上方最新记录。

全图类型共享路径优化历史记录（基线 `a937258`，代码提交 `6df32f8`）：

- 普通输入在预处理阶段借用源文本，含前置配置、指令、注释、换行归一化或属性改写候选的输入保留完整处理流程；源位置映射首个点内联存储，解析前复用已有文本缓冲区。SVG 坐标整段写入，标签换行改用迭代器并原地裁剪尾部空白。公开预处理结果、模型所有权、数值舍入、文本测量顺序和布局算法不变。
- `cargo test --workspace --all-features --locked -- --test-threads=1`：**3,195 通过、0 失败、5 ignored**；无默认 feature 的 4 项回归通过。新增两项项目回归覆盖源映射空步骤、复制与替换、末尾钳制，以及借用/完整预处理的双文本视图和全部字节位置；不计入原测试映射。
- 三轮交替扩展基准 **39/39 组行为一致**。33 类文档 render 等权 after/before 为 **0.97335**（耗时 -2.67%），parse 为 **0.93186**（-6.81%）；33/33 类观察到更低的渲染中位数。Info render -15.91%、parse -34.41%；Block render -4.38%；Mindmap parse -20.63%。C4 parse +2.19%、50 节点树 render +1.09% 等结果完整保留，不将小幅变化声明为统计显著性。
- 全部 167 个 Sample 的配对 render 耗时比为 **0.98229**（-1.77%），30/33 类中位数下降；flowchart +0.46%、ER +0.54%、swimlane +0.12%。当前 Rust/Pele 等权加速 **API 返回 1.53×、UTF-8 就绪 1.62×**。四引擎共同可比的 110 个 Sample/22 类另外复测，不混合两种输入集合。
- 分配探针覆盖全部 33 类 Sample 和 39 组扩展负载，分配次数与累计申请字节全部下降。Info 分配 **62 → 42**，Mindmap **855 → 776**，Block **1,240 → 1,170**。Sample 额外存活峰值 27 类持平、6 类略增（最多 157 bytes），不宣称峰值或 RSS 下降；全部测量批后净保留均为 0。
- 761/761 完整响应、167/167 Sample SVG 对照通过。原始超时门禁：**2,785 验收通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**；`complete: false`、退出 1。原有测试、fixture、映射、断言、种子、次数和阈值均未改变；完整[验证记录](research/shared-pipeline-optimization-2026-10-20/validation.json)另存。
- 最终构建后未修改生产源码，计时与编译、测试、CPU 采样及分配探针错开。Clippy 可完成，保留仓库已有警告。首页仍为英文并只显示终态对比；CLI 和库依赖均使用锁定当前代码提交的远程 Git 地址。

第二轮渲染优化历史记录（基线 `488bdee`）：

- 上一轮优化已提交并推送到 `main`：`488bdee08ea9d6c19fe034c6c1596ee9c90cd349`。
- `cargo test --workspace --all-features --locked -- --test-threads=1`：**3,193 通过、0 失败、5 ignored**；无默认 feature 的 4 项回归通过。新增独立上游 2,902 组样式对照和 Sequence UTF-16 全边界回归，均为补充项目测试，不增加原测试映射数量。
- 39/39 组行为验证相同；33 类文档 render 等权 after/before 为 **0.99070**，parse 为 **0.99053**。Sequence 渲染耗时 -10.72%、解析 -27.96%；Block 渲染 -6.20%、解析 -5.68%。整体约 1% 的差异不作统计显著性声明，所有略慢的组仍保留在报告。
- 同一样本批次的分配次数：Block **1,563 → 1,240**，Sequence **1,114 → 810**；额外存活峰值基本不变，不宣称 RSS 按相同比例下降。六组样本与 39 组扩展批次的批后净保留均为 0，仅代表本次探针结果。
- 当前四引擎终态表、双输出终点表已复测；全部 167 个 Sample、33 类的 API 返回 Rust/Pele 等权加速 **1.50×**，UTF-8 就绪 **1.60×**。167/167 SVG 与 761/761 完整响应对照通过。
- 原测试输入、断言、种子、阈值、映射与既有 fixture 不变。原始超时门禁：**2,785 验收通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**；`complete: false`、退出 1。完整[验证记录](research/render-optimization-round2-2026-10-20/validation.json)另存。
- Clippy 可完成，仓库既有警告仍未全量清理；最终构建和测量后未修改生产源码。

第一轮渲染优化历史记录（基线 `0bbe183`）：

- `cargo test --workspace --all-features --locked -- --test-threads=1`：**3,191 通过、0 失败、5 ignored**。新增两项项目回归覆盖独立上游的 512 组 SVG 根节点输入和长 Unicode 字符串搜索；不计入原始测试映射。
- 原有测试源文件、映射、断言、随机种子和阈值不变；761/761 完整响应与 167/167 Sample SVG 相等。
- 相对 `0bbe183` 的三轮交替 39 组扩展基准：39/39 行为一致；33 类文档 render 等权 after/before 为 **0.87447**，parse 为 **0.93572**。Journey render -36.73%、Timeline -21.48%、Sequence -16.94%、Block -13.72%；小幅变化不作统计显著性声明。
- 同一 Journey 样例：分配 188 → 114，扩容 68 → 48，累计申请 55,658 → 46,216 bytes；六组独立分配探针批后净保留均为 0。不是完整泄漏验证。
- 当前四引擎终态表已复测；新增全部 167 个 Sample、33 类的双输出终点报告：API 返回 Rust/Pele 等权加速 **1.48×**，UTF-8 就绪 **1.57×**。该集合与四引擎共同成功的 110 个 Sample/22 类不同，不拼接比值。
- 原始超时门禁实际结果：**2,785 验收通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，`complete: false`、退出 1。真正无默认图表构建的三项项目回归通过。完整[验证记录](research/render-optimization-2026-10-20/validation.json)另存。
- 新增直接依赖 memchr 2.8.3，版本锁定、许可记录完整；运行时仍为纯 Rust。Clippy 可完成，仓库已有警告仍未全量清理。


Pele 1.7.0 本轮：

- 英文首页与交互画廊已刷新到当前版本：33 类、167 个原始 Sample，三种渲染器全部成功，Rust/Pele 167/167 SVG 字节及尺寸一致。99 张首页预览已重生成。
- 当前四引擎三轮基准已完成：共同可比集合 110/167 个 Sample、22 类图表；完整终态表格、口径与证据见 [英文性能报告](BENCHMARK.md)。未用历史优化前后数据替代当前测量。
- 完整上游（含网站/宿主）：292 个文件、6,826 项；6,721 通过、103 skip、2 todo。不能计作 Rust 通过。
- `cargo test --workspace --all-features --locked -- --test-threads=1`：3,189 通过、0 失败、5 保留跳过。
- 新增原始测试映射：16 项 `mermaidType` 全部参数行与 6 项 CJK 换行测试。保持原输入、断言、标点字符集和 40…240、步长 8 的宽度循环。
- `tests/fixtures/pele-1.7.0.jsonl.gz`：由固定上游源码独立生成，761 条完整响应一致，含 702 成功和 59 预期拒绝；比较 SVG、尺寸、链接、类型名和完整错误字段。旧版输出不改写。
- 下述性能、采集全集与其他批次数字均为历史记录，不冒充本轮 1.7.0 全套验收。完整更新见 [PELE-1.7.0.md](PELE-1.7.0.md)。

升级前历史验证：

- `.research/all-33-verification.log`：全部 features 的工作区测试 1,096 通过、0 失败、2 跳过；包括 70,183 条采集全集对照和 13,194 条抽样对照。抽样是全集子集，不能相加。
- `.research/default-33-verification.log`：切换默认全类型后，工作区回归 1,100 通过、0 失败、2 跳过；包含后来增加的 4 项新图类型主题测试。
- `.research/state-full-verification.log`：state 4,443 条采集输入一致；22 条文档输入与主题矩阵通过。
- `.research/usecase-full-verification.log`：usecase 2,676 条采集输入一致；24 条文档输入与主题矩阵通过。
- 33 类主题矩阵共 702 条成功文档/参考输入，各用两套调色板验证具体属性和尺寸稳定性。
- 13 条源输入包含 Rust UTF-8 字符串无法表示的孤立 UTF-16 代理项，按精确记录清单排除，未算通过；抽样中有 6 条。该排除不是全量原始测试验收的豁免。
- 原始测试逐项移植：2,770 项有映射，2,765 通过、5 保留原跳过，4,040 未映射。ER 兼容解析器的 632 项有实际操作/断言的测试在原版和 Rust 两侧都通过，其中原跳过的 Unicode 测试已经启用；本批另增 ER 47 项、State 53 项、Requirement 36 项原始压力测试。

性能验证已保存两种口径：

1. [Pele 原脚本口径](BENCHMARK-PELE-PROTOCOL.zh-CN.md)：原脚本直接执行，5 组输入和统计方式完全沿用；118 个输入完整 SVG 与尺寸一致。
2. [全图类型扩展基准](BENCHMARK.zh-CN.md)：33 类构建、39/39 组一致，共 820 个成功渲染和 59 个预期拒绝；成功渲染与拒绝处理分别计时，三轮独立进程交替顺序。

以上都不是“6,810 项原测试已全部移植”。上游原始基线为 6,705 通过、103 跳过、2 todo（包含网站测试）；另发现两个显示 passed 的 ER 测试体只有 TODO 注释，未生成空测试凑数。

首轮 profiling 后的回归快照：`.research/profiling-parity-full.log` 记录全部 features 工作区 **2,503 通过、0 失败、2 跳过**，开启采集全集对照。逐项迁移新增公共模块、flowchart 原始解析器与 783 项兼容测试、Gantt 日期和 class 测试。class 底层解析器的前导换行问题已由原始跳过项暴露并修复；上游 skip 来源仍保留。

CPU/内存 profiling 已覆盖 39 组工作负载，并对 3 组采集函数调用栈。SVG 优化前后三轮交替计时的 39/39 组验证记录相同；33 类文档 render 耗时比几何平均 0.944。三个 flowchart 基线的累计申请量下降 7.74%–16.29%，额外存活峰值下降 5.57%–16.11%。详见 [profiling 与优化报告](PROFILING-AND-OPTIMIZATION.zh-CN.md)，其中明确区分批次分配、进程 RSS、CPU 时间及纯渲染耗时。

## 6. 已知差异与决定

| 项 | 说明 |
| --- | --- |
| info 文本 | 新的原测试移植要求暴露默认行为差异，1.7.0 默认显示 `Pele v1.7.0`（旧版 fixture 显式指定 `Pele v1.6.0`）；`RenderOptions::info_text` 仍可由宿主显式指定。CLI 自身版本不变。 |
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

1. 继续按 6,826 项全量测试清单逐项移植断言与修复实现；图类型本身已全部接通。
2. 收敛 Clippy 风格告警，完成跨平台构建、性能优化与发布验证。当前有部分类型慢于 Pele，不宣称全面更快。
3. 原生 Zebra App/xcframework 接入尚未执行；当前工作集中在本 Rust 库、CLI 与兼容适配层。

## 9. 主题范围澄清

用户补充：Pele CSS 主题若足以接入 Zebra，则无需增加 JSON 主题配置。不会要求用户另行维护 JSON 主题文件；现有 palette JSON 是 Zebra 已有的 Swift → Rust ABI 传输方式。

已沿当前 macOS 展示路径做最小实测（`NSImage(data:)`，见 `.research/native-theme-probe/`）：字面量 `#CC3311` 与 `currentColor` 正确；CSS `var()`、嵌套别名、变量 fallback 和 `color-mix()` 输出黑色。因此保留网页 Pele CSS 输出，以及原生具体颜色输出；原生颜色来自 Zebra 当前主题自动转换。该探针不代表完整 App 集成验收。


此前原始测试门禁已使用逐项独立进程，按全部 6,810 项清单中的原始 Vitest 超时运行已映射项：2,629 通过、0 失败、5 跳过、4,176 未映射，门禁因目标尚未全覆盖而按预期非零退出。Flowchart 兼容测试 977/983 通过（另 3 跳过、3 未映射）；Class 解析器 382/384 通过；Sequence 125/157 通过，已覆盖该文件全部 125 项运行用例，另外 32 项原始跳过仍未映射。详见逐项验收文档中的缺口说明。


上次测试扩展快照的完整工作区回归：`ZM_FULL_HARVEST=1 cargo test --workspace --all-features` 为 **2,841 通过、0 失败、5 跳过**，包含采集全集、主题与 ABI 验证。工作区测试数量包含项目自身测试，与 6,810 项原始测试的逐项进度分别统计。常规 Clippy 检查仍有既存警告；额外尝试 `-D warnings` 时因警告退出，未将其报告为通过。

Class 样式缓存改进前的性能快照：39/39 组配对验证一致，33 类文档 render 耗时等权几何平均比原始 Rust 基线低约 5.57%；另复测 4 组进程 CPU/RSS/分配和 3 组 CPU 栈。详见 [性能复测报告](PROFILING-TEST-EXPANSION.zh-CN.md)。这些是当前快照的实测，不是额外叠加 5.57% 的优化。


新增 Sequence/Class/Agentflow 全部 172 项原始压力测试，原版与 Rust 两侧分别全通过。压力测试保留源构造、重复/循环规模、类型检查、3 秒性能门槛和 30 秒超时，并据其暴露的重复样式解析开销补齐 Class 绘制缓存。修复后的原始测试严格门禁为 **2,629 通过、0 失败、5 跳过、4,176 未映射**；当前完整工作区回归为 **3,013 通过、0 失败、5 跳过**，包含采集全集对照。目标仍为全部 6,810 项通过。


随后基于采样继续优化：Class 声明使用 `Arc<String>` 共享存储，Agentflow 使用插入序号保留序列化顺序并改为常数时间成员删除，去掉临时引用数组。相对缓存版本，两个大样式场景峰值存活内存下降 70.58% / 72.19%；Agentflow global 压力测试约从 2.55 秒降到 0.87 秒，同时峰值增加约 28 KiB。模型 JSON、SVG 与 Zebra ABI 均经验证。最新工作区回归为 **3,015 通过、0 失败、5 跳过**；原始测试仍为 2,629/6,810 通过。原脚本与 39 组 Rust/Pele 对照已用当前代码刷新；详细结果与 Rust 模型类型变更见 [CPU/内存优化报告](PROFILING-CLASS-AGENTFLOW.zh-CN.md)。


最新性能/内存轮次：Agentflow 私有成员索引记录所属 flow，单节点 global 保序删除、批量 global 仅过滤受影响的 flow。目标 release 压力场景 52.85 → 9.57 ms（耗时低 81.89%，5.52 倍速度），峰值增加 28 KiB；39/39 组配对验证一致，33 类文档 render 比值几何平均 0.98561。Pele 原脚本五组已用当前代码刷新，扩展 Rust/Pele 表保留此前测量快照。持续内存检查合计 1,408,680 个输入处理轮次，各轮净保留 0 字节/0 块；四条系统 leaks 路径均为 0，受控泄漏成功检出。RSS 波动另行记录，不等同于请求字节或对象泄漏。最新全工作区回归 **3,016 通过、0 失败、5 跳过**；172 项原始压力测试全部通过；逐项独立超时审计仍为 **2,629/6,810 通过、5 跳过、4,176 未映射**。详见 [CPU 再优化与持续内存验证](PROFILING-CPU-MEMORY.zh-CN.md)。本轮未提交或推送。

## 10. Codex 接管后的原始测试扩展（2026-10-20）

保留既有未提交工作，继续全量原测试移植，未恢复 Claude 会话。新增 ER 47 项、State 53 项、Requirement 36 项，共 **136 项独立映射**，全部沿用固定提交的输入、断言和计时门槛。原版 Pele 直接复跑 136/136 通过；六个生成压力测试文件的 Rust 独立进程执行合计 308/308 通过。详情与复现方式见 [逐项验收记录](UPSTREAM-TEST-PARITY.md#erstaterequirement-压力测试扩展)。

修复新测试暴露的三处性能问题：ER 属性块缓存泛型/空白扫描位置；State 缓存行尾、注释标记和左括号位置；State 描述就地累积，最终节点描述仅在遍历结束时移入。State 长行问题另以 1 秒、1 ms 间隔的外部 CPU 采样定位到 `Lexer::line_end`；原始采样只保留在忽略的 `target/test-parity/`。增加 4 项项目回归测试核对描述顺序、note 同名覆盖与组描述错误，期望值来自固定 Pele 的独立核对，不计入上游映射数。

本批实际验证：

- `node tools/test-parity/check.mjs --require-complete`：**2,765 通过、0 失败、5 跳过、4,040 未映射、0 待验证**。逐项使用原始超时；`complete: false`，因未完成全量而按预期退出 1。
- `cargo test --workspace --all-features -- --test-threads=1`：**3,163 通过、0 失败、5 跳过**，是常规工作区结果，包含项目自身测试，不等于上游逐项数量。
- 33 类已提交采集样本的 SVG、尺寸、链接及错误对照：**13,194/13,194 一致**；另有 **6 条孤立 UTF-16 代理项输入未执行、不计通过**。Quadrant/TreeView 专用复核与这批样本重叠，不重复累计。
- 本轮本地 `target/compat/harvest/fixtures/` 不存在，未重新执行 70,183 条采集全集；此前的全集和 39/39 基准结果保留为历史快照。样本排除仍不是 6,810 项原测试验收的豁免。
- Node v26.3.0、Rust 1.96.1；生成内容校验、修改文件的格式检查与差异空白检查通过。日志保存在 `target/test-parity/continued-*.log`，原始门禁报告为 `target/test-parity/report.json`。

当前全量原测试通过率为 **40.60%**，图类型覆盖率保持 **33/33**，本轮输出样本对照为 **13,194/13,194**。剩余 **4,040 项未映射**包含 Rust 核心 3,977 项、宿主/打包 53 项、网站 10 项；全量目标及原始 skip/todo 要求不变。未提交或推送。

## 11. 完整性能复测与发布快照

按用户要求生成 [完整性能对比 Markdown](BENCHMARK-COMPLETE.zh-CN.md)，性能快照标签为 `v0.1.0-benchmark.20261020`。报告覆盖 33 类/39 组完整渲染与仅解析、独立拒绝计时、Pele 原脚本五组、四引擎同输入对比、39 组 CPU/RSS 与 Rust 分配、进程启动和构建体积。生产构建先完成，基准和资源采样串行执行，回归测试在全部性能测量结束后运行。

- Rust/Pele **39/39 组行为一致**，按工作负载累计 820 次成功渲染、59 次预期拒绝。33 类文档渲染耗时比几何平均 **0.6280**，仅解析为 **0.5198**；每类等权，只描述本次工作负载，不推广成任意输入或跨平台加速结论。
- Pele 原协议 **118/118** 完整 SVG 与尺寸相等。四引擎均尝试原始 **167 个 Sample**；Mermaid.js、Pele、Rust 各 167 项通过类型/输出检查，mmdr 110 项，实际共同计时集合为 **110 项、22 类**。Pele/Rust 的 **167/167** SVG 字节及尺寸一致。
- 全部 **39 组资源实验**各执行 3 轮新进程 CPU/RSS 采样；Rust 单独分配计数的批后净保留均为 0 字节。RSS、请求分配量及纯渲染耗时分别报告，不混合比较，也不替代长期泄漏验证。
- 性能测量后重新运行严格门禁：**2,765 通过、0 失败、5 跳过、4,040 未映射、0 待验证**，仍因未全覆盖而退出 1。常规工作区为 **3,163 通过、0 失败、5 跳过**；已提交样本为 **13,194/13,194 一致**，另 6 条孤立代理项仍未执行。不把这些结果计作完整 6,810 项验收。
- 本轮生成的新数据位于 `docs/research/*-codex-takeover/`，保留原始样本、验证结果、依赖/源码/二进制 SHA-256 和执行方法；不记录开发者机器身份或配置。已有性能快照保留历史含义，不与本次比值叠加。

串行复现入口：`python3 tools/bench/run-complete.py`；依赖准备、环境变量与新证据目录命名见完整报告。该标签记录实现、测试与性能文档的同一代码快照，不代表全量原始测试移植或原生 Zebra App 集成已经完成。
