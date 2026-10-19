# Pele 全量测试 1:1 移植与验收

2026-10-19 用户新增硬性要求：**Pele 中所有测试都要 1:1 平移，并通过。** 原来的输入采集与 SVG 差分仍是补充验证，不能代替原始测试。

## 完整参照

- 固定版本：`44da152bbb79b91f03e68016f42499bb25bf6e82`。
- 逐项清单：[`tests/upstream/inventory.json`](../tests/upstream/inventory.json)。保存文件、SHA-256、完整测试名称、位置、文件内序号、执行模式和稳定 ID。
- 完整原始测试及辅助文件：`tests/upstream/source/tests/`，保留上游版权注释，许可证在 `tests/upstream/source/LICENSE` 与 `THIRD_PARTY_NOTICES.md`。
- 参数化测试展开为独立项；重复名称通过文件内序号区分。
- 测试范围是全部 `tests/**/*.{test,spec}.{ts,js}`，包含 `tests/website/`、mount、zoom、browser、字体加载与入口打包测试。不能沿用 `npm test` 排除 website 的范围。

完整上游复跑结果：**291 个文件，6,810 个测试项；6,705 通过、103 跳过、2 todo**。这是 TypeScript 参照基线，不是 Rust 结果。网站依赖按其现有锁文件安装，未改动测试或实现源文件。

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

# 最终完整门禁：未移植、未运行、失败或跳过都使其非零退出
node tools/test-parity/check.mjs --require-complete
```

映射保存在 [`tests/upstream/ports.json`](../tests/upstream/ports.json)，Rust 入口是 [`upstream_parity.rs`](../crates/zebra-mermaid/tests/upstream_parity.rs)。审计工具检查：

1. 冻结源文件哈希、测试总数及唯一 ID。
2. 上游测试与目标测试严格一一对应，禁止多个上游项合并为一个目标测试。
3. 映射中的目标确实出现在编译后的 libtest 列表中，不能因 feature 未开启而消失。
4. 实际执行结果；测试源码哈希一同写入报告。
5. 有未移植项时，子集测试即使全部通过，报告 `complete` 仍为 `false`。

报告及日志在 `target/test-parity/`。该目录是运行产物；映射本身不缓存“已通过”状态，必须实际运行才能记为 passed。

## 当前移植结果

| 原文件 | 展开项数 | Rust 通过 | 保留上游跳过 |
| --- | ---: | ---: | ---: |
| `tests/render/info.test.ts` | 5 | 5 | 0 |
| `tests/render/edges.test.ts` | 6 | 6 | 0 |
| `tests/compat/info/info.test.ts` | 12 | 12 | 0 |
| `tests/compat/info/info.spec.ts` | 4 | 2 | 2 |
| `tests/info/stress.test.ts` | 13 | 13 | 0 |
| `tests/layout/compound.test.ts` | 22 | 22 | 0 |
| `tests/layout/lanes.test.ts` | 24 | 24 | 0 |
| `tests/compat/er/erDiagram.spec.js`（有实际操作/断言的项） | 632 | 632 | 0 |
| **合计** | **718** | **716** | **2** |

还有 **6,092 项未建立逐项移植映射**；未映射不代表实现一定失败，但不能计为满足本验收要求。按全部 6,810 项计算，当前 1:1 通过率约 **10.51%**，与 33/33 的图表覆盖率、39/39 基准一致性是不同指标。

ER 文件共有 634 项。其中 632 项通过机械转换保留解析调用、字段访问、全部断言与异常检查；生成过程中模型观察值保持符号形式，未拿原版或 Rust 输出生成期望值。转换流分别在原版 Pele 和真实 Rust 原始解析器上运行通过。原先跳过的 Unicode 名称测试已启用并通过，原跳过标记仍保存在清单。其余两项虽然在上游显示 passed，但函数体只有 `/* TODO */`，没有输入、断言或受测调用，当前保持未移植，不生成空测试凑数。

相关工具：`tools/test-parity/compile-er.mjs`（生成/校验逐项转换）、`tools/test-parity/verify-er-reference.mjs`（用原始 Pele 解析器验证同一断言流）。不支持的测试语法会记为未转换；审计运行前重新生成比对 IR 与测试映射，防止期望值被手动改成 Rust 输出。

首批测试暴露 `info` 默认版本文本与上游不同。已修正库默认输出为固定参照的 `Pele v1.6.0`，原始断言不变；调用方仍可显式设置 `info_text`。CLI 自身版本继续使用项目版本。此前采集差分显式传入版本文本，不能证明默认行为一致。

33 类解析与渲染已接通；后续继续补齐原始测试逐项映射、宿主适配、网站和打包测试。全量完成前不宣布“完整替代 Pele”。
