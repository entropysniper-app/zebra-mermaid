# Pele 全量测试 1:1 移植与验收

2026-10-19 用户新增硬性要求：**Pele 中所有测试都要 1:1 平移，并通过。** 原来的输入采集与 SVG 差分仍是补充验证，不能代替原始测试。

## 完整参照

- 固定版本：Pele 1.9.0，`e7f092a4de147cc32210913c20e36eaa5fcd0ef1`（按用户 2026-10-23 要求升级；原版本的证据保留）。
- 逐项清单：[`tests/upstream/inventory.json`](../tests/upstream/inventory.json)。保存文件、SHA-256、完整测试名称、位置、文件内序号、执行模式、原始 Vitest 超时值和稳定 ID。
- 完整原始测试及辅助文件：`tests/upstream/source/tests/`，保留上游版权注释，许可证在 `tests/upstream/source/LICENSE` 与 `THIRD_PARTY_NOTICES.md`。
- 参数化测试展开为独立项；重复名称通过文件内序号区分。
- 测试范围是全部 `tests/**/*.{test,spec}.{ts,js}`，包含 `tests/website/`、mount、zoom、browser、字体加载与入口打包测试。不能沿用 `npm test` 排除 website 的范围。

完整上游复跑结果：**295 个文件，6,971 个测试项；6,866 通过、103 跳过、2 todo**。这是 TypeScript 参照基线，不是 Rust 结果。网站依赖按其现有锁文件安装，未改动测试或实现源文件。

## 1:1 的含义

每个测试的原始输入、参数值、预期值、断言关系、顺序、循环次数、随机数生成算法及调用顺序、性能阈值和失败行为都要保留。Rust 的 Result/静态类型可对应 JS 的异常类型/运行时类型断言，必须在映射记录中解释。

原本调用第三方解析器作参照的测试，仍须使用该独立参照产生的预期结果，并保留原断言；不能用被测 Rust 实现生成自己的期望值。原始 DOM/浏览器行为需要相应宿主适配测试，未实现时保持未移植状态，不能改成仅检查 SVG 字符串。

不得通过减少样本、放宽时间或数值阈值、修改期望值、增加 ignore、吞掉 panic、提前返回成功来通过。已知差异和采集工具排除清单不自动豁免本要求。原本的 skip/todo 单独登记，不能宣称为通过；当前严格门禁对它们也不会给出“全部完成”。

## 可执行验收

```sh
# 从固定参照运行全部测试并重新生成清单（需先安装 Pele 与 website 的锁定依赖）
node tools/test-parity/inventory.mjs

# 执行已映射的真实 Rust 测试；报告仍明确显示未完成数量
node tools/test-parity/check.mjs --run

# 原时限审计：每个测试在独立进程运行，强制执行原始超时
node tools/test-parity/check.mjs --run --isolate

# 最终完整门禁：包含原时限审计；未移植、未运行、失败或跳过都使其非零退出
node tools/test-parity/check.mjs --require-complete
```

映射保存在 [`tests/upstream/ports.json`](../tests/upstream/ports.json)，Rust 入口是 [`upstream_parity.rs`](../crates/zebra-mermaid/tests/upstream_parity.rs)。审计工具检查：

1. 冻结源文件哈希、测试总数及唯一 ID。
2. 上游测试与目标测试严格一一对应，禁止多个上游项合并为一个目标测试。
3. 映射中的目标确实出现在编译后的 libtest 列表中，不能因 feature 未开启而消失。
4. 实际执行结果；测试源码和测试二进制 SHA-256 一同写入报告。完整门禁为每个用例启动独立进程，使用参照运行记录的原始超时值；启动时间计入时限，不添加宽限。
5. 有未移植项时，子集测试即使全部通过，报告 `complete` 仍为 `false`。

报告及日志在 `target/test-parity/`。该目录是运行产物；映射本身不缓存“已通过”状态，必须实际运行才能记为 passed。

## 当前移植结果

2026-10-23 的逐项独立进程原超时审计：2,808 通过、0 失败、7 保留原 skip、4,156 未映射、0 待验证。完整门禁仍非零退出；[执行证据](research/pele-1.9.0-alignment-2026-10-23/validation.json)与[完整原始报告](research/pele-1.9.0-alignment-2026-10-23/upstream-report.json.gz)单独保存。

用户明确要求默认使用 RaTeX 并移除数学回调。这是有意的宿主 API 差异；上游数学回调用例完整保留，不以默认 RaTeX 的补充测试替代或计入等价移植通过。

| 原文件 | 已映射项数 | Rust 验收通过 | 保留上游跳过 |
| --- | ---: | ---: | ---: |
| `tests/agentflow/stress.test.ts` | 69 | 69 | 0 |
| `tests/class/stress.test.ts` | 56 | 56 | 0 |
| `tests/compat/class/classDiagram.spec.ts`（原文件 384 项） | 382 | 381 | 1 |
| `tests/compat/class/classTypes.spec.ts` | 93 | 93 | 0 |
| `tests/compat/er/erDiagram.spec.js`（原文件 634 项） | 632 | 631 | 1 |
| `tests/compat/flowchart/flow-arrows.spec.js` | 14 | 14 | 0 |
| `tests/compat/flowchart/flow-comments.spec.js` | 9 | 9 | 0 |
| `tests/compat/flowchart/flow-direction.spec.js` | 4 | 4 | 0 |
| `tests/compat/flowchart/flow-edges.spec.js` | 293 | 293 | 0 |
| `tests/compat/flowchart/flow-interactions.spec.js` | 13 | 13 | 0 |
| `tests/compat/flowchart/flow-lines.spec.js` | 12 | 12 | 0 |
| `tests/compat/flowchart/flow-md-string.spec.js` | 2 | 2 | 0 |
| `tests/compat/flowchart/flow-node-data.spec.js` | 31 | 30 | 1 |
| `tests/compat/flowchart/flow-singlenode.spec.js` | 148 | 148 | 0 |
| `tests/compat/flowchart/flow-style.spec.js`（原文件 25 项） | 24 | 24 | 0 |
| `tests/compat/flowchart/flow-text.spec.js` | 342 | 342 | 0 |
| `tests/compat/flowchart/flow-vertice-chaining.spec.js` | 7 | 7 | 0 |
| `tests/compat/flowchart/flow.spec.js` | 26 | 25 | 1 |
| `tests/compat/flowchart/flowDb.spec.ts`（原文件 32 项） | 31 | 31 | 0 |
| `tests/compat/flowchart/subgraph.spec.js` | 24 | 23 | 1 |
| `tests/compat/info/info.spec.ts` | 4 | 2 | 2 |
| `tests/compat/info/info.test.ts` | 12 | 12 | 0 |
| `tests/compat/sequence/sequenceDiagram.spec.js`（原文件 157 项） | 125 | 125 | 0 |
| `tests/er/stress.test.ts` | 47 | 47 | 0 |
| `tests/flowchart/parser.test.ts` | 6 | 6 | 0 |
| `tests/gantt/dates.test.ts` | 90 | 90 | 0 |
| `tests/info/stress.test.ts` | 13 | 13 | 0 |
| `tests/layout/compound.test.ts` | 27 | 27 | 0 |
| `tests/layout/layered.test.ts` | 1 | 1 | 0 |
| `tests/render/break-hints.test.ts` | 11 | 11 | 0 |
| `tests/layout/lanes.test.ts` | 24 | 24 | 0 |
| `tests/render/edges.test.ts` | 6 | 6 | 0 |
| `tests/render/fonts.test.ts`（原文件 7 项） | 6 | 6 | 0 |
| `tests/render/info.test.ts` | 5 | 5 | 0 |
| `tests/render/mermaid-type.test.ts` | 16 | 16 | 0 |
| `tests/render/cjk-wrap.test.ts` | 6 | 6 | 0 |
| `tests/requirement/stress.test.ts` | 36 | 36 | 0 |
| `tests/sequence/stress.test.ts` | 47 | 47 | 0 |
| `tests/state/stress.test.ts` | 53 | 53 | 0 |
| `tests/util/preprocess.test.ts` | 9 | 9 | 0 |
| `tests/util/url.test.ts` | 30 | 30 | 0 |
| `tests/util/yaml.test.ts` | 29 | 29 | 0 |
| **合计** | **2,815** | **2,808** | **7** |

未映射项按范围分为 **Rust 核心 4,090 项、宿主/打包 56 项、网站 10 项**，三个范围均保留在目标内。

还有 **4,156 项未建立逐项移植映射**；未映射不代表实现一定失败，但不能计为满足本验收要求。按全部 6,971 项计算，当前可计入验收的 1:1 通过率约 **40.28%**，与 33/33 的图表覆盖率、39/39 基准一致性是不同指标；本批已重新完成 39/39 组验证与计时，详见 [扩展报告](BENCHMARK.zh-CN.md)，仍不替代原测试逐项验收。

ER 文件共有 634 项。其中 632 项通过机械转换保留解析调用、字段访问、全部断言与异常检查；生成过程中模型观察值保持符号形式，未拿原版或 Rust 输出生成期望值。转换流分别在原版 Pele 和真实 Rust 原始解析器上运行通过。原先跳过的 Unicode 名称测试已启用并实际执行通过，但仍按原 skip 标记计入跳过，不计入正式验收通过数；class 前导换行测试同理。其余两项虽然在上游显示 passed，但函数体只有 `/* TODO */`，没有输入、断言或受测调用，当前保持未移植，不生成空测试凑数。

相关工具：`tools/test-parity/compile-er.mjs`（生成/校验逐项转换）、`tools/test-parity/verify-er-reference.mjs`（用原始 Pele 解析器验证同一断言流）。不支持的测试语法会记为未转换；审计运行前重新生成比对 IR 与测试映射，防止期望值被手动改成 Rust 输出。

首批测试暴露 `info` 默认版本文本与上游不同。库默认输出跟随固定参照，当前升级至 1.9.0 后为 `Pele v1.9.0`，原始断言不变；调用方仍可显式设置 `info_text`。CLI 自身版本继续使用项目版本。此前采集差分显式传入版本文本，不能证明默认行为一致。

33 类解析与渲染已接通；后续继续补齐原始测试逐项映射、宿主适配、网站和打包测试。全量完成前不宣布“完整替代 Pele”。

## 新增迁移批次：公共模块、日期和 class

本批新增 620 项，原始断言保持不变：URL/YAML/预处理 68 项、flowchart 原始解析器 6 项、Gantt 日期 90 项、class 成员 93 项、class 解析器 363 项。

- 公共模块的期望值来自原测试调用的独立参照：`@braintree/sanitize-url`、`js-yaml` JSON_SCHEMA，以及原测试中的 Mermaid 正则参考函数。预处理随机测试保留种子 11、全部 20,000 次迭代，Rust 重新生成并核对相同的随机输入序列；性能断言保留 200/500 ms 原门槛。
- Gantt 保留所有参数行、Date/NaN/undefined 类型、区间与日期数组断言。在固定 UTC 测试环境执行；4 个标题内插入的时间戳随时区变化，通过原文件序号与规范化标题关联原稳定 ID，清单本身不修改。90 项断言同时在原版日期实现上通过；这不代表完整 DST 支持已经完成。
- class 成员 93 项保留全部 125 个字符串断言，分别在原版和 Rust 上通过。
- class 解析器共有 384 项，382 项已迁移通过：379 项生成断言流，3 项独立原生配置/图结构测试。两项原始 skip 仍缺 Mermaid 特有快照字段和 JS 自有属性反射；不会以忽略断言的方式算通过。`class-parser.json` 还列出由独立原生测试覆盖的三个配置用例，实际映射以 `ports.json` 为准。
- 新启用的 class 前导换行测试原本为 skip，在原始 Pele 底层解析器上仍失败。Rust 修复了原始解析器在 `classDiagram` 头部前接受换行的行为，保持原 token 位置用于错误定位；公开渲染入口本来就会预处理这些空白。Rust 的原断言现已通过，上游 skip 标记仍保留，不能宣称原版也通过。

生成校验脚本为 `compile-utilities.mjs`、`compile-gantt-dates.mjs`、`compile-class-members.mjs` 和 `compile-class-parser.mjs`，均由审计门禁调用。Gantt、class 成员与 class 解析器另有对应 `verify-*-reference.mjs`，验证断言流的原版行为；原版跳过项的失败独立记录。

此前完整门禁实际执行了全部已映射项及各自原始超时：**2,629 通过、0 失败、5 跳过、4,176 未映射**。后续 ER/State/Requirement 压力测试批次新增 136 项，见本文末尾的验证记录；当时全量完成目标为 6,810 项；1.7.0 已扩展到 6,826 项。

## Flowchart 兼容测试批次

继续新增 783 项，完整覆盖 `flow-text.spec.js`、`flow-edges.spec.js`、`flow-singlenode.spec.js` 三个文件。原版和 Rust 分别执行通过全部 783 项断言流；保留注册循环、每个参数行、重复标题、解析调用、节点/边属性、普通 throw 以及指定错误消息检查。使用原始适配器默认的 500 条边限制，未改成无限制。

`compile-flow-parser.mjs --check` 在审计时检查转换结果；`verify-flow-parser-reference.mjs` 验证原版行为。转换器现在也检查 `it.each` 内部原始回调，避免只检查外层包装函数而遗漏控制流。其余文件的扩展结果见下文；不能把子集通过扩展为全部 983 项兼容测试通过。

## Flowchart 模型与交互扩展

Flowchart 兼容测试共 983 项，现已建立 980 项映射，其中 977 项通过，3 项保留原始跳过，另 3 项尚未映射。生成断言流覆盖 14 个文件的 949 项：946 通过、3 跳过；`flowDb.spec.ts` 另外逐项移植了 31 项运行用例。

本批保留元数据、展开/折叠组、父子组颜色顺序、边动画字段、链接/tooltip/回调调用参数、警告回调及局部对象匹配。调用记录来自真实 Rust 模型方法，未根据断言合成；回调名和参数经过实际解析器传递，渲染器继续不执行宿主回调。

三个原始跳过项在 Pele 参照侧也失败：节点元数据链末尾空白、裸 `<` 标签的转义预期，以及子图输入与预期 ID/顺序不一致。三个未映射项为 `flow-style.spec.js` 中只有外层 `expect(function)` 而没有执行 matcher 的用例、`flowDb.spec.ts` 的 JS 自有属性反射用例，以及与适配器 500 条边限制冲突的巨大图跳过用例。它们均未计为通过，也未改写原始预期。

## Sequence 全部运行用例与 Class 遗留断言

Sequence 原文件 157 项中，全部 125 项运行用例已经逐项在原版 Pele 与 Rust 两侧通过；其余 32 项上游 skip 保持未映射。主要缺口是 Mermaid 渲染器的 bounds/绘图模型与 JS 自有属性反射；固定 Pele 适配器自身没有这些 bounds 方法，不能用空实现记为通过。

转换器保留每个 Diagram 的独立状态、ActorView 副本、`filter`/`find`/`forEach` 的实际查询与遍历、渲染后的编号值和可见性、公用预处理解析入口，以及原始 try/catch 中的布尔值与错误消息断言。两个无显式 matcher 的用例实际调用解析器，按原来的成功解析要求执行；它们与没有受测调用的空 TODO 分开处理。125 项超时逐一与原始 Vitest 运行记录核对，默认 60 秒，防卡死用例为 5 秒；Rust 将超时或 panic 作为失败，不转成预期解析异常。

Class 解析器现为 382/384 项逐项通过：379 项生成断言流，加上 3 项独立原生命名空间配置测试。新增调用 spy 验证链接、tooltip、回调名及任意数量参数；省略参数和显式参数保持不同。紧凑模式与默认层级模式使用真实图构建器检查分组、标签、显式标记和父节点。原版执行生成断言流时 378 项通过、1 项原 skip 的前导换行仍失败；Rust 该项已按原断言修复并通过。尚未移植的两项原 skip 涉及 Mermaid 特有的 `domId`/实例快照与 JS 自有属性反射，原预期保持原样。


## 原始压力测试与 Class 样式缓存

新增完整迁移 `tests/sequence/stress.test.ts` 47 项、`tests/class/stress.test.ts` 56 项和 `tests/agentflow/stress.test.ts` 69 项，共 **172 项**。固定 Pele 源码直接复跑 172/172 通过，Rust 逐项独立进程也为 172/172 通过。原始 3,000 ms 内层性能门槛、30,000 ms 外层超时，以及最后的默认输入限制测试均保留。

`compile-stress.mjs` 根据原始输入构造 AST 生成 Rust 操作：包括字符串 repeat 的截断、Array.from 后 join、模板插值、条件分支、双层 complete-graph 循环及每次调用的操作类型。输入构造仍在计时区间；每份输入另与原始 JavaScript 构造器生成的 FNV-64 指纹核对，核对位于计时区间之外。未预先存好字符串来省略原来的构造工作。Infinity 仅对应源输入限制，其余输出/边数限制保留默认值；只接受原生 Error，panic 不伪装成预期异常。测试按原 Vitest 行为在每个源文件内顺序执行。

首轮 Rust 暴露两个原始 Class 压力测试超时：1,600 个 classDef 应用于 1,600 个类，以及 5,000 项声明应用于 800 个类。CPU/分配采样显示重复调用样式解析造成大量临时分配。现已按 Pele 的行为加入每次绘制内的声明缓存和 CSS 类组合缓存，保留属性第一次出现的输出顺序与最后有效声明的覆盖规则；这些缓存不跨渲染保留。原始断言和输入未改，修复后两个场景均满足原门槛。

压力测试生成记录见 `tests/upstream/ir/stress.json`，专用逐项执行入口为 `node tools/test-parity/run-stress.mjs`；该入口记录实际耗时和失败，完整清单仍以总门禁为准。

## ER、State、Requirement 压力测试扩展

2026-10-20 新增三个原始文件的全部 **136 项**：ER 47 项、State 53 项、Requirement 36 项。固定 Pele 源码直接复跑 **136/136 通过**，Rust 全部六个生成压力测试文件合计 **308/308 通过**。既有 Info 的 13 项独立压力测试仍通过原来的入口执行，不包含在 308 项中。

转换器新增 ER `genericTypes` 对共享泛型解析器的 API 映射，以及 Requirement `Array.from({ length }, callback).join(separator)` 的原生构造。保留全部回调、元素顺序、分隔符、循环和原始输入指纹；构造仍在原计时区间内。State 的原文件没有末尾默认长度限制用例，不额外伪造；每项保留 **60,000 ms** 原始超时，ER/Requirement 的计时用例保留 **30,000 ms** 超时，全部保留 **3,000 ms** 内层门槛。

新增测试暴露 ER 属性词法分析的重复空白/泛型查找、State 长行的重复行尾/注释/左括号查找，以及 State 描述累积与节点快照的重复复制。现分别缓存扫描位置、就地追加描述，并在遍历结束时把最终描述移入节点；保留节点顺序、同名 note 节点的覆盖顺序和组描述语义错误。另有 4 项项目回归测试验证这些图模型边界，期望值在固定 Pele 上独立核对，不计入 6,810 项的逐项移植数量。

复现：`node tools/test-parity/compile-stress.mjs --check`、`node tools/test-parity/run-stress.mjs`。Node v26.3.0、Rust 1.96.1；Rust debug 构建，逐项独立进程串行执行。修复后单次诊断耗时：ER 10,000 个属性键 6.85 ms；State 同行 3,000 次 direction 12.13 ms、未闭合字符串 41.50 ms、重复描述用例 63.69 ms。这些是门槛验证中的单次读数，不是多轮性能基准。报告保存在忽略的 `target/test-parity/stress-debug-results.json` 和 `target/test-parity/continued-upstream-stress.json`。

本批最终严格门禁 `node tools/test-parity/check.mjs --require-complete` 实际结果为 **2,765 通过、0 失败、5 跳过、4,040 未映射、0 待验证**，`complete: false`，按预期退出 1。常规工作区回归为 **3,163 通过、0 失败、5 跳过**；已提交采集样本独立对照 **13,194/13,194 一致**，另 6 条孤立代理项输入未执行，不计通过，也不豁免原测试验收。本轮未复跑本地缺失的采集全集，亦未刷新此前 39/39 性能基准。

后续按用户发布要求完成了历史完整性能复测（原始证据见 `research/complete-benchmark-2026-10-19-codex-takeover/`），刷新当时的 39/39 组基准与四引擎数据，并在测量后重跑上述严格门禁和工作区回归，结果数量不变。性能标签 `v0.1.0-benchmark.20261020` 不增加原始测试映射或通过数量；全量验收仍未完成。


## 2026-10-20：Pele 1.7.0

基线升级至 `8603bf3313add9143cd417015d4a6bd6c522dada`。原有 6,810 项全部保留，新增 `mermaidType` 的 16 项后清单为 6,826 项。新增 16 项类型测试和 6 项 CJK 换行测试均逐项映射并执行；info 模型断言同步增加 `mermaidType`。

最终 `node tools/test-parity/check.mjs --require-complete` 结果：**2,785 验收通过、0 失败、7 保留上游跳过、4,034 未映射、0 待验证**，每项采用原 Vitest 超时、独立进程执行，`complete: false`，退出 1。实际 Rust 执行是 2,787 passed、5 ignored；其中 Class 前导换行与 ER Unicode 两个测试在上游仍标记 skip，因此分别保留 `executionState: passed` 和验收 `state: skipped`，不计入 2,785。旧报告的执行通过数不追溯改写，当前统计遵守原 skip/todo 不计通过的约束。

图表覆盖 **33/33**，本轮新参照完整响应 **761/761**，原始测试正式验收通过率 **40.80%**，三种指标互不替代。宿主入口与 mount 新增断言仍保持未移植，源文件和哈希已同步。完整变更与复现见 [PELE-1.7.0.md](PELE-1.7.0.md)。


## 字体与根样式原始测试

`tests/render/fonts.test.ts` 的前六项现已分别对应六个独立 Rust 测试，新增正式通过六项。保留全部 **37 个原断言条件、19 次原 render 调用**、四项继承属性循环、五种标题/栏目图表输入、全部字体参数及预期字符串。正则通过精确子串或等价字节识别器表达；`[\d.]+` 保留 ASCII 数字/点的非空范围，`--_t?h?w:` 保留四种展开，标题标签与圆角标签的边界以及否定前瞻均保留。Rust 显式传入 `MetricsMeasurer`，不添加测试依赖，也不调用 JS 作为受测渲染器。

第七项依赖完整 `assertInert` / XML helper，仍明确未映射；没有删掉它的安全断言、用较弱检查替代或新增跳过。另保留四项项目自身的 scanner 状态/边界回归，它们不增加原始测试验收数。

已在两种实际状态验证：当前标签运行时候选，以及仅包含测试变更的隔离提交快照（已排除尚未完成性能测量的标签运行时改动）。两者完整工作区均为 **3,236 通过、0 失败、5 ignored**；按每项原始时限独立执行的严格门禁均为 **2,791 通过、0 失败、7 上游跳过、4,028 未映射、0 待验证**，退出 1、`complete: false`。隔离快照首次构建缺少一份原有调色板文档 fixture，已从同一基线提交按原字节补齐后完整重跑；没有改变期望值或验收条件。

[逐项断言审计](research/font-test-parity-2026-10-20/assertion-audit.json) · [隔离提交验证](research/font-test-parity-2026-10-20/head-validation.json) · [当前候选验证](research/font-test-parity-2026-10-20/candidate-validation.json)。完整原时限结果保存在同目录的 gzip JSON 中。
