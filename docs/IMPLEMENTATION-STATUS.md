# zebra-mermaid 实施状态

最后更新：2026-11-06（渲染器对齐验证于 2026-11-01 完成）。实现提交为 `40b9fa59c294068662df83e58262cb607db0913b`，固定参照为 **Pele 1.13.0**，提交 `3a2f075ee0c3d914f0fad3545fc0f193693975ac`，语法目标为 Mermaid 12.1.0。

本次从 1.12.0 对齐至 1.13.0，默认数学排版继续内置 RaTeX。**持续性能与代码体积优化仍停止；全量原测试 1:1 验收尚未完成。** 历史性能与体积数据保留原参照，不改标为新版本测量。

## crates.io 发布（2026-11-06）

用户要求将当前版本发布到 crates.io。库、CLI、FFI 均准备为 **0.1.0**：补齐仓库/文档元数据、明确内部注册表版本、随包 README、MIT 许可文本及第三方通知，FFI 包含 C 头文件；开发 profiler 维持不发布。包内运行时代码与仓库逐字节一致，渲染器逻辑未改。

三个包的默认及全 feature 发布预演通过，包括从解包后的归档独立构建。归档压缩后约 745 KiB、25 KiB、16 KiB；这些是源码发布包大小，不是编译体积。大型开发夹具和测试入口保留在仓库，不上传到 crates.io；仓库原测试未删除、未改断言或增加跳过。三个包已于 2026-11-06 由 `entropysniper-app` 从提交 `4f38aceb7b5e3af10d64ab89d5a8ba2ba5bbe7c7` 发布为 **0.1.0**（该提交的 `crates/*/src`、FFI 头文件和 `Cargo.lock` 与 tag `v0.1.0-pele1.13.0` 一致）。注册表 API 校验和、下载归档与本地打包三者一致，未 yank；`cargo install zebra-mermaid-cli --version =0.1.0 --locked` 成功并可渲染 SVG。记录见 [发布记录](research/crates-io-0.1.0-2026-11-06/publication.json)。

本轮工作区复验为 **3,416 passed、0 failed、5 ignored**；独立原时限门禁仍为 **2,952 passed、0 failed、7 保留跳过、4,151 未映射、0 待验证**，退出 1、`complete: false`。记录见 [发布准备验证](research/crates-io-0.1.0-2026-11-06/validation.json) 与同目录的 `package-audit.json`、`upstream-report.json.gz`。

## 当前验证结果

| 指标 | 实际结果 |
| --- | --- |
| 默认启用图类型 | **33/33** |
| 原始 Playground 样本 | **167/167** SVG 字节与尺寸相同，三引擎各 167 成功；原始输入和顺序未变 |
| 文档与参考完整响应 | **761/761**，含 702 成功、59 预期拒绝 |
| 已提交采集样本 | **13,194/13,194** UTF-8 可表示输入一致；6 条孤立 UTF-16 代理项未执行，不计通过 |
| 发布补充对照 | **316/316** 完整响应一致 |
| 扩展负载 | **39/39** 组一致，879 次输入：820 成功、59 预期拒绝 |
| 原脚本负载 | **118/118** 完整 SVG 和尺寸相同 |
| 工作区回归 | **3,416 passed、0 failed、5 ignored** |
| 原测试清单 | **298 个文件、7,110 项**；上游基线 **7,005 passed、103 skipped、2 todo** |
| Rust 原测试正式验收 | **2,952 passed、0 failed、7 保留跳过、4,151 未映射、0 待验证**，通过率 **41.52%** |

图类型覆盖、输出一致数量和原测试通过率分别统计，不能互相替代。完整门禁 `node tools/test-parity/check.mjs --require-complete` 保留每项原始 Vitest 时限并独立进程执行。存在未映射或保留跳过时 `complete: false`、退出 1，不能宣称全套通过。两项 Rust 实际执行成功的原 skip 仍不计验收通过。

本轮为全部 **18 项新增核心测试**建立独立映射：调色板 10、Compound 布局 3、Block 2、Quadrant 2、Flowchart 1。另按上游原文更新了 1 项已有 Flowchart 测试输入，保留所有断言。新增的 **3 项网站 source/CSS 分享链接测试未移植**，来源与状态完整保留。

证据见 [逐项测试记录](UPSTREAM-TEST-PARITY.md)、[本轮验证](research/pele-1.13.0-alignment-2026-10-31/validation.json)、同目录逐项报告与源码差异。格式、无默认 feature 类型检查与普通 Clippy 均退出 0；Clippy 有 75 项警告，未宣称严格无警告通过。

## 1.13.0 对齐内容

- Mermaid 调色板变量：Pie 的 pie1–pie8、Git 的 git0–git7、Timeline/Radar/Treemap 的 cScale0–cScale7、Mindmap 的 cScale1–cScale8、Venn 的 venn1–venn8、XY 的 plotColorPalette。Venn 与 XY 短列表重复；其他图的变量和非颜色值保持当前主题回退。
- 颜色校验：保留函数内部逗号，限制可用色函数与 `--pele-*` 变量；拒绝资源 URL、声明拼接、脚本、未配对括号及未知裸颜色。Pie、Git、Timeline、Treemap 的系列文字按指定颜色选择对比色，作者文字样式继续优先。
- 作用域：调色板归每次 Rust 渲染所有，不使用跨渲染可变全局状态。网页输出保留上游 CSS 声明。原生输出支持裸颜色、rgb/hsl、已知宿主变量和 light-dark；其他高级 CSS 色函数或无法解析的变量回退当前宿主系列色，不输出 CSS 运行时依赖，不要求额外主题 JSON。
- 布局：对齐子图边界与内部节点的连接端口；只有虚拟弯点的对齐块可移动到 24 px，含真实节点的块仍限 12 px。起始端口不再把其目标推下一层，未连接的子图成员保留同行；原数字精度和断言门槛不变。
- Block：跨多列的非固定形状利用额外宽度重新排标签，保持节点尺寸和连线路由；固定形状与显式换行保留原行为。
- Quadrant：支持 titleFontSize、quadrantLabelFontSize、xAxisLabelFontSize、yAxisLabelFontSize、pointLabelFontSize；有限数夹到 4–72，未设置时沿用 fontSize。点标签按绘图区宽度换行。
- 参照与工具：默认 info 为 Pele v1.13.0，更新库、CLI、测试快照、生成器、画廊与许可证。生成测试的 rustfmt 改用已关闭的临时输入文件，修复大 stdin 管道等待 EOF 的卡住；格式规则、生成断言和源审计保持原样。

本轮仅进行正确性验证，不重测基准速度、分配、RSS 或代码体积。为节省构建临时空间，开发验证使用 `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`，保留 dev 的 opt-level 0 与 debug assertions；release 配置未改。该构建参数不代表性能基准结果，原测试的输入、循环和性能门槛未放宽。

上游网站新增 CSS 编辑/分享、版本显示、键盘处理及样式调整保留在差异记录中；网站行为不作为 Rust 已实现通过。历史来源和 fixture 保持原样。

## 默认数学排版


按用户要求，**删除 math 回调 API，默认使用 RaTeX 0.1.15**。运行时不依赖 JS、Node、浏览器或外部布局程序。RaTeX 解析和排版，内置 KaTeX 字体生成 SVG 字形路径；普通公式无需宿主字体或 CSS。

`$$…$$` 标签公式启用于 flowchart、sequence、state、class、ER、mindmap、kanban、block、requirement、usecase、swimlane 共 11 类。公式尺寸参与换行、节点尺寸和行距；原生输出采用当前宿主主题的具体颜色，保留作者显式公式颜色。无效 TeX、不可用字形回退为源文本；原生禁图策略同样作用于公式。标题、class 成员和其他图类型保持文字行为。

这属于用户明确指定的宿主 API 差异：**Pele 的回调行为测试仍保留在清单中且未映射；默认 RaTeX 的补充测试不冒充其 1:1 移植。** 既有数学专项为 2 项单元测试、6 项集成测试，覆盖全部 11 类、两种颜色模式、尺寸、有效 XML、作者颜色与透明度、错误回退及原生图像限制。

[自包含数学 SVG 示例](math/flowchart.svg) · [源文件](math/flowchart.mmd) · [第三方许可](../THIRD_PARTY_NOTICES.md)

## 最近已发布的性能与体积测量

以下为 **2026-10-23、Pele 1.9.0** 的既有测量。本次仅进行正确性验证，未重测速度、分配、RSS 或代码体积，也未恢复持续优化。它们不代表 1.13.0 的性能或体积结果。


全部 **167 个原始样本、33 类**、完整解析/布局/SVG 生成的逐类型等权几何平均加速比：**API 返回 1.76×，UTF-8 就绪 1.89×**。API 范围为 agentflow **1.20×** 至 sankey **2.77×**，23 类低于 2×、10 类达到 2×。**2× API 目标未达到。** 未因升级或数学接入而恢复持续优化，也不把旧版结果改标为新版本。

三轮独立进程交替顺序，5 批预热、至少 10 ms 校准、至少 9 个样本且累计至少 300 ms；构建、测试、浏览器绘图和探针在计时之外。原始 167 个样本没有公式，当次性能数字不代表数学公式吞吐量。UTF-8 口径包含 Node 的编码和 Buffer 分配，不能当成单纯字符串展平收益。

39 组扩展负载全部保留，33 类文档的 Rust/Pele render 耗时比几何平均为 **0.53938**，parse 为 **0.50958**；这组输入与主目标样本不同，不能用它替代主指标。拒绝输入、每类数据和全部三轮样本分别保存。当次未重新测量分配、RSS 或 CPU 栈，旧版资源数据不作为当前内存结论。

[主口径完整表](OUTPUT-BENCHMARK.md) · [39 组扩展表](BENCHMARK.zh-CN.md) · [原脚本五组](BENCHMARK-PELE-PROTOCOL.zh-CN.md) · [本次报告索引](BENCHMARK-COMPLETE.zh-CN.md)

相同工具链与 release/thin LTO/单 codegen unit 配置、完整功能、各版本锁定依赖、未 strip 的发布产物：

| 产物 | 文件字节 | 指令代码段字节 |
| --- | ---: | ---: |
| CLI | 9,237,040 | 5,649,348 |
| FFI 动态库 | 8,811,776 | 5,326,912 |
| FFI 静态归档 | 62,027,464 | 6,400,400 |

升级和默认数学引擎/字体带来明显体积增长；相对前一发布基线，CLI 文件增加 4,076,960 字节、指令代码段增加 1,851,432 字节。该比较包含新增功能，不作为体积优化收益，也不单独归因于某一个依赖。静态库总量包含 LLVM bitcode 和链接前对象，不等于最终宿主链接增量。体积目标没有用户指定的数字门槛。完整三个产物的差值、分段、源码和二进制指纹见 [代码体积记录](CODE-SIZE-OPTIMIZATION.md)。

## Zebra 主题与宿主契约


复用 Zebra 当前主题，不要求用户另行维护 JSON 主题文件。现有 camelCase palette JSON 是既有 Swift → Rust ABI 的传输格式。网页模式保留 Pele CSS；原生模式解析具体颜色，不依赖 `var()`、`color-mix()` 或媒体查询。作者样式优先，透明画布只省略整幅背景，内部对比度继续使用宿主背景色。

默认全部 33 类启用。`MermaidPalette`、light/dark、fontSize、画布和颜色模式分别传入，颜色改变不改变几何。Zebra `target_width` 校验但不参与布局，保留 `renderingDependsOnTargetWidth = false`；普通调用的 `max_width` 独立生效。原生策略关闭链接与图像外部引用。

现有主题矩阵、适配与 C ABI v2 单元测试通过；完整原生 App/xcframework 替换、App 内主题切换与缓存验收尚未执行。主题契约和字段回退见 [ZEBRA-THEME-COMPATIBILITY.md](ZEBRA-THEME-COMPATIBILITY.md)，C 头文件见 [zebra_mermaid.h](../crates/zebra-mermaid-ffi/include/zebra_mermaid.h)。

## 保留的限制与未完成事项


- **4,151 项未映射**：Rust 核心 4,079、宿主/打包 59、网站 13；7 项保留上游跳过。未映射不等于实际失败，但不能算作验收通过。原始 skip/todo 和仅含 TODO 注释的空测试不用于凑通过数。
- UTF-8 字符串不能保存孤立 UTF-16 代理项。成对代理项可合并，既有替换/排除记录保留；6 条未执行采集样本不是全量验收的豁免。未重新执行本地缺失的旧版 70,183 条采集全集。
- `%%{init}%%` JSON 深度上限为 200；与 JS 依赖栈耗尽的失败边界存在差异。Zebra 源长上限沿宿主 262,144 个 UTF-16 单位，输出上限为 4 MiB UTF-8 字节。
- Gantt 对照固定 UTC；当前提供固定偏移，完整 DST 策略未完成。fdlibm/V8 对照保留，但平台 C `pow` 的舍入仍不保证跨平台一致。
- `TextMeasurer` 采用 `&self`；Zebra 标签直接使用 fontSize，旧宿主的 13/16 缩放与饼图切片透明度差异仍需 App 集成时确认。
- 尚未完成跨平台发布验收、严格无警告 lint、全量原测试迁移和原生宿主验收。2× 性能未达标；持续优化已停止。

[项目方案](RUST-REWRITE-PLAN.zh-CN.md) · [测试计划](TEST-PLAN.md) · [此前保留的性能轮次](LAZY-TOKEN-STATE-OPTIMIZATION.md) · [历史体积基线](research/code-size-baseline-2026-10-20/summary.json)。历史结果按原提交解释，不计入本次验收。
