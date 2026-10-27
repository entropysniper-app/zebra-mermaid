# Pele 全量测试 1:1 移植与验收

2026-10-19 用户新增硬性要求：**Pele 中所有测试都要 1:1 平移，并通过。** 输入采集与 SVG 差分是补充验证，不能代替原始测试。

## 完整参照

- 固定版本：Pele 1.12.0，`8aff183be57021b533f319a29341a529be934b08`（按用户 2026-10-27 要求升级；此前版本的证据保留）。
- 逐项清单：[`tests/upstream/inventory.json`](../tests/upstream/inventory.json)，保存文件、SHA-256、完整名称、位置、文件内序号、执行模式、原始 Vitest 超时和稳定 ID。参数化行分别展开，重复名称由文件内序号区分。
- 完整原始测试和辅助文件：`tests/upstream/source/tests/`，保留上游版权和许可证。
- 范围包含全部 `tests/**/*.{test,spec}.{ts,js}`，包括网站、mount、zoom、browser、字体加载与入口打包测试，不沿用 `npm test` 排除 website 的范围。

上游全套复跑：**296 个文件、7,089 项；6,984 passed、103 skipped、2 todo**。这是 TypeScript 参照基线，不是 Rust 的通过数。

## 1:1 的含义

每项输入、参数值、预期值、断言关系、顺序、循环次数、随机算法和调用顺序、性能阈值和失败行为都要保留。Rust Result、Option、枚举和字段断言对应 JS 异常、undefined、类型名和对象字段，逐项在映射中解释。

第三方参照继续独立生成原断言需要的预期结果，不能用 Rust 输出生成自己的期望值。DOM/浏览器行为需要宿主适配测试，不能改成仅检查 SVG。禁止删减断言、减少样本、放宽门槛、增加跳过或生成空 TODO 测试凑通过数。原始 skip/todo 保留来源且单独计数；Rust 实际执行成功也不把原 skip 算成验收通过。

## 可执行验收

```sh
# 固定参照的全部测试与清单（Pele 和 website 的依赖均按锁文件安装）
node tools/test-parity/inventory.mjs

# 已映射子集；成功不等于完整验收
node tools/test-parity/check.mjs --run

# 每项独立进程、执行原 Vitest 超时，启动时间计入时限
node tools/test-parity/check.mjs --run --isolate

# 未映射、失败、待验证或跳过均使完整门禁非零退出
node tools/test-parity/check.mjs --require-complete
```

[`ports.json`](../tests/upstream/ports.json) 严格一项对应一个实际测试；审计校验源哈希、目标唯一性、编译后的真实测试列表和实际结果。九类转换器的源与断言校验必须通过，补充 golden fixture 不接受为原测试映射。报告和本地日志在 `target/test-parity/`，映射不缓存“已通过”状态。

## 当前移植结果

2026-10-27 原超时独立进程审计：**2,934 passed、0 failed、7 保留原 skip、4,148 未映射、0 待验证**，通过率 **41.39%**。`complete: false`，完整门禁退出码为 1。

本次增加 94 项独立映射：全部 93 项新增核心用例，以及 1 项改名的 Railroad 模型用例；另按 1.12.0 原文更新 3 项已有映射的完整断言。上游净增 96 项，新增的 3 项 Zoom 浏览器行为用例保留为宿主未映射项。84 个回调手写平移；8 个 ink 回调通过符号转换保留全部 497 次调用、350 个断言及 148 个颜色名称的循环，2 个 URL 回调保留原先全部编码地址与布尔断言。全部保留输入、调用关系、预期、循环、参数和时限。

3 个修改过的已映射回调分别为：Sequence break hints 在间距扩展后改为两行的原预期；Flowchart 末尾注释/换行前的 EOF 位置；Radar 注释之后的 EOF 位置。上游修改了断言，Rust 跟随固定源码更新，没有用 Rust 输出生成预期。其他 8 个修改过的未映射原回调保留来源，未计通过。

[验证汇总](research/pele-1.12.0-alignment-2026-10-27/validation.json) · [逐项报告](research/pele-1.12.0-alignment-2026-10-27/upstream-report.json.gz) · [源码与测试增量](research/pele-1.12.0-alignment-2026-10-27/source-delta.json)。316 个发布补充完整响应、761 个文档响应、167 个样本和 39 组负载单独报告，不计作原测试映射。

用户指定的默认 RaTeX 与移除数学回调属于有意的宿主 API 差异。上游回调测试完整保留且未映射，默认数学补充测试不冒充等价移植。独立 Jison oracle、网站、宿主和打包行为继续在全套验收内。

| 原文件 | 已映射项数 | Rust 验收通过 | 保留上游跳过 |
| --- | ---: | ---: | ---: |
| `tests/agentflow/stress.test.ts` | 69 | 69 | 0 |
| `tests/class/stress.test.ts` | 56 | 56 | 0 |
| `tests/compat/class/classDiagram.spec.ts` | 382 | 381 | 1 |
| `tests/compat/class/classTypes.spec.ts` | 93 | 93 | 0 |
| `tests/compat/er/erDiagram.spec.js` | 632 | 631 | 1 |
| `tests/compat/flowchart/flow-arrows.spec.js` | 14 | 14 | 0 |
| `tests/compat/flowchart/flow-comments.spec.js` | 9 | 9 | 0 |
| `tests/compat/flowchart/flow-direction.spec.js` | 4 | 4 | 0 |
| `tests/compat/flowchart/flow-edges.spec.js` | 293 | 293 | 0 |
| `tests/compat/flowchart/flow-interactions.spec.js` | 13 | 13 | 0 |
| `tests/compat/flowchart/flow-lines.spec.js` | 12 | 12 | 0 |
| `tests/compat/flowchart/flow-md-string.spec.js` | 2 | 2 | 0 |
| `tests/compat/flowchart/flow-node-data.spec.js` | 31 | 30 | 1 |
| `tests/compat/flowchart/flow-singlenode.spec.js` | 148 | 148 | 0 |
| `tests/compat/flowchart/flow-style.spec.js` | 24 | 24 | 0 |
| `tests/compat/flowchart/flow-text.spec.js` | 342 | 342 | 0 |
| `tests/compat/flowchart/flow-vertice-chaining.spec.js` | 7 | 7 | 0 |
| `tests/compat/flowchart/flow.spec.js` | 26 | 25 | 1 |
| `tests/compat/flowchart/flowDb.spec.ts` | 31 | 31 | 0 |
| `tests/compat/flowchart/subgraph.spec.js` | 24 | 23 | 1 |
| `tests/compat/info/info.spec.ts` | 4 | 2 | 2 |
| `tests/compat/info/info.test.ts` | 12 | 12 | 0 |
| `tests/compat/sequence/sequenceDiagram.spec.js` | 125 | 125 | 0 |
| `tests/er/stress.test.ts` | 47 | 47 | 0 |
| `tests/flowchart/parser.test.ts` | 6 | 6 | 0 |
| `tests/gantt/dates.test.ts` | 90 | 90 | 0 |
| `tests/info/stress.test.ts` | 13 | 13 | 0 |
| `tests/layout/compound.test.ts` | 27 | 27 | 0 |
| `tests/layout/lanes.test.ts` | 24 | 24 | 0 |
| `tests/layout/layered.test.ts` | 1 | 1 | 0 |
| `tests/railroad/model.test.ts` | 1 | 1 | 0 |
| `tests/render/agentflow.test.ts` | 4 | 4 | 0 |
| `tests/render/architecture.test.ts` | 1 | 1 | 0 |
| `tests/render/block.test.ts` | 16 | 16 | 0 |
| `tests/render/break-hints.test.ts` | 11 | 11 | 0 |
| `tests/render/c4.test.ts` | 3 | 3 | 0 |
| `tests/render/cjk-wrap.test.ts` | 6 | 6 | 0 |
| `tests/render/class.test.ts` | 4 | 4 | 0 |
| `tests/render/edges.test.ts` | 6 | 6 | 0 |
| `tests/render/er.test.ts` | 6 | 6 | 0 |
| `tests/render/eventmodeling.test.ts` | 2 | 2 | 0 |
| `tests/render/flowchart.test.ts` | 9 | 9 | 0 |
| `tests/render/fonts.test.ts` | 6 | 6 | 0 |
| `tests/render/gantt.test.ts` | 2 | 2 | 0 |
| `tests/render/git.test.ts` | 1 | 1 | 0 |
| `tests/render/info.test.ts` | 5 | 5 | 0 |
| `tests/render/ink.test.ts` | 8 | 8 | 0 |
| `tests/render/journey.test.ts` | 2 | 2 | 0 |
| `tests/render/mermaid-type.test.ts` | 16 | 16 | 0 |
| `tests/render/pie.test.ts` | 1 | 1 | 0 |
| `tests/render/quadrant.test.ts` | 2 | 2 | 0 |
| `tests/render/radar.test.ts` | 3 | 3 | 0 |
| `tests/render/railroad.test.ts` | 1 | 1 | 0 |
| `tests/render/requirement.test.ts` | 1 | 1 | 0 |
| `tests/render/sankey.test.ts` | 6 | 6 | 0 |
| `tests/render/security.test.ts` | 1 | 1 | 0 |
| `tests/render/sequence.test.ts` | 9 | 9 | 0 |
| `tests/render/state.test.ts` | 3 | 3 | 0 |
| `tests/render/swimlane.test.ts` | 2 | 2 | 0 |
| `tests/render/timeline.test.ts` | 2 | 2 | 0 |
| `tests/render/treemap.test.ts` | 16 | 16 | 0 |
| `tests/render/treeview.test.ts` | 3 | 3 | 0 |
| `tests/render/usecase.test.ts` | 4 | 4 | 0 |
| `tests/render/venn.test.ts` | 6 | 6 | 0 |
| `tests/render/xychart.test.ts` | 2 | 2 | 0 |
| `tests/requirement/stress.test.ts` | 36 | 36 | 0 |
| `tests/sequence/stress.test.ts` | 47 | 47 | 0 |
| `tests/state/stress.test.ts` | 53 | 53 | 0 |
| `tests/stress.test.ts` | 1 | 1 | 0 |
| `tests/util/preprocess.test.ts` | 9 | 9 | 0 |
| `tests/util/url.test.ts` | 34 | 34 | 0 |
| `tests/util/yaml.test.ts` | 29 | 29 | 0 |
| **合计** | **2,941** | **2,934** | **7** |

未映射分为 Rust 核心 **4,079**、宿主/打包 **59**、网站 **10**。未映射不等于实现一定失败，也不能算作验收通过。未执行的 UTF-16 孤立代理项等既有差异不构成完整验收豁免。

以下保留此前迁移批次的历史结果，不能累计成当前通过数。当前清单与结果以前述固定版本和逐项报告为准。

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
