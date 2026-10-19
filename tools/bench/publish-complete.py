#!/usr/bin/env python3
import datetime
import hashlib
import json
import math
import os
import pathlib
import re
import statistics
import subprocess

root = pathlib.Path(__file__).resolve().parents[2]
read = lambda name: json.loads((root / name).read_text())
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
validation_steps = []
for name, command in [
    ('parity', ['node', 'tools/test-parity/check.mjs', '--require-complete']),
    ('workspace', ['cargo', 'test', '--workspace', '--all-features', '--', '--test-threads=1']),
    ('output-sample', ['cargo', 'test', '-p', 'zebra-mermaid', '--test', 'compat_harvest', '--test', 'quadrant_harvest', '--test', 'treeview_harvest', 'sample', '--', '--test-threads=1', '--nocapture']),
]:
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    print(f'{started}: {name}', flush=True)
    with (root / 'target/bench-complete' / f'{name}.log').open('w') as log:
        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
    expected_incomplete = False
    if name == 'parity' and result.returncode == 1:
        report = read('target/test-parity/report.json')
        expected_incomplete = (not report['complete'] and report['execution']['isolated']
                              and report['summary']['failed'] == report['summary']['unverified'] == 0
                              and report['summary']['unmapped'] + report['summary']['skipped'] > 0)
    validation_steps.append({'step': name, 'command': command, 'startedAt': started,
                             'finishedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                             'exitCode': result.returncode, 'expectedIncomplete': expected_incomplete})
    (root / 'target/bench-complete/validation-execution.json').write_text(json.dumps(validation_steps, indent=2) + '\n')
    if result.returncode != 0 and not expected_incomplete:
        raise SystemExit(f'{name} failed; see target/bench-complete/{name}.log')
extended = read('target/bench/comparison.json')
upstream = read('target/bench-upstream/comparison.json')
resources = read('target/profiling/resources.json')
cross = read('target/cross-bench/comparison.json')
verification = read('target/cross-bench/verification.json')
equality = read('target/cross-bench/pele-zebra-equality.json')
parity = read('target/test-parity/report.json')
workloads = read('target/bench/cases.json')
suffix = os.environ.get('BENCH_REPORT_SUFFIX', '')
assert not suffix or re.fullmatch(r'-[a-z0-9-]+', suffix), 'Invalid report suffix'
tag = os.environ.get('BENCHMARK_TAG', '')
assert re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', tag), 'Set BENCHMARK_TAG to the intended snapshot tag'
directory = 'complete-benchmark-' + extended['createdAt'][:10] + suffix
destination = root / 'docs/research' / directory
assert not destination.exists(), 'Do not overwrite published evidence'
assert len(extended['rows']) == 39 and len(extended['enabledTypes']) == 33
assert all(row['consistent'] and row['eligible'] for row in extended['rows'])
assert digest(root / 'target/bench/cases.json') == extended['inputSha256']
assert upstream['validation']['success'] and upstream['validation']['diagrams'] == 118
assert len(resources['cases']) == 39
assert [row['case'] for row in resources['cases']] == [row['name'] for row in extended['rows']]
for row in resources['cases']:
    assert len(row['runs']) == 3 and row['rustAllocation']['retainedBytesAfterBatch'] == 0
    for trial in row['runs']:
        assert trial['rust']['batches'] > 0 and trial['pele']['batches'] > 0
        for key in ('renderedPerBatch', 'rejectedPerBatch', 'svgBytesPerBatch'):
            assert trial['rust'][key] == trial['pele'][key], (row['case'], key)
assert len(equality) == 167 and all(row['equal'] for row in equality)
assert len(cross['coverage']) == 33 and sum(row['total'] for row in cross['coverage']) == 167
assert digest(root / 'target/cross-bench-build/release/zebra-mermaid-cross-bench') == cross['nativeBinarySha256']
assert parity['execution']['isolated'] and parity['summary']['failed'] == parity['summary']['unverified'] == 0
for name, expected in parity['sourceHashes'].items():
    assert digest(root / name) == expected, ('Parity source changed after validation', name)
workspace_log = (root / 'target/bench-complete/workspace.log').read_text()
counts = [tuple(map(int, match)) for match in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', workspace_log)]
assert counts and 'test result: FAILED' not in workspace_log
workspace = dict(zip(('passed', 'failed', 'skipped'), map(sum, zip(*counts))))
sample_log = (root / 'target/bench-complete/output-sample.log').read_text()
sample = re.search(r'^harvest: (\d+) cases match$', sample_log, re.M)
assert sample and int(sample[1]) == 13194
documents = [row for row in extended['rows'] if row['name'].startswith('corpus:') and row['name'].endswith('-docs')]
assert len(documents) == 33
geometric = lambda values: math.exp(statistics.mean(math.log(value) for value in values))
render_ratio = geometric(row['render']['rustOverPele'] for row in documents)
parse_ratio = geometric(row['parse']['rustOverPele'] for row in documents)
common_count = sum(row['common'] for row in cross['coverage'])
success = sum(row['renderInputs'] for row in extended['rows'])
rejection = sum(row['rejectInputs'] for row in extended['rows'])
engines = ['mermaid', 'pele', 'zebra', 'mmdr']
names = {'mermaid': 'Mermaid.js', 'pele': 'Pele', 'zebra': 'zebra-mermaid', 'mmdr': 'mermaid-rs-renderer'}
coverage = {engine: sum(row['ok'] for row in verification[engine].values()) for engine in engines}
playwright_version = subprocess.check_output(
    ['node', '-e', 'console.log(require((process.env.PLAYWRIGHT_MODULE || "playwright") + "/package.json").version)'],
    cwd=root, text=True,
).strip()
execution = read('target/bench-complete/execution.json')
measurement_steps = [record for record in execution if record['step'] in ('upstream', 'extended', 'resources', 'cross')]
assert len(measurement_steps) == 4 and all(record['exitCode'] == 0 for record in measurement_steps)
source_files = sorted({*root.glob('crates/**/*.rs'), *root.glob('crates/**/Cargo.toml'), *root.glob('tools/profiler/src/*.rs'),
                       *root.glob('tools/cross-bench/src/*.rs'), *root.glob('tools/bench/*.py'), *root.glob('tools/bench/*.mjs'),
                       root / 'Cargo.toml', root / 'Cargo.lock', root / 'tools/cross-bench/Cargo.toml', root / 'tools/cross-bench/Cargo.lock',
                       root / 'tools/profiler/Cargo.toml'})
metadata = {
    'createdAt': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'releaseTag': tag,
    'baseCommit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
    'sourceIdentity': 'The working tree is identified by sourceFiles SHA-256; the release tag points to the commit containing these files and reports.',
    'sourceFiles': [{'path': path.relative_to(root).as_posix(), 'sha256': digest(path)} for path in source_files],
    'binaries': {name: digest(root / name) for name in ['target/release/zebra-mermaid', 'target/release/examples/bench',
                 'target/release/examples/bench_upstream', 'target/release/examples/profile', 'target/release/zebra-mermaid-profiler',
                 'target/cross-bench-build/release/zebra-mermaid-cross-bench']},
    'peleBundleSha256': digest(root / '.research/pele/dist/pele.js'), 'inputSha256': extended['inputSha256'],
    'referenceFiles': [{'path': name, 'sha256': digest(root / name)} for name in
                       ['.research/pele/package-lock.json', '.research/pele/website/pnpm-lock.yaml',
                        '.research/pele/website/node_modules/mermaid/package.json']],
    'versions': {**cross['versions'], 'playwright': playwright_version}, 'measurementSteps': measurement_steps,
    'validationSteps': validation_steps,
    'method': 'Production builds finish before measurement. All benchmark and resource subprocesses run sequentially; no parallel compilation or tests. Portable metadata excludes machine identity and configuration.',
}
validation = {'upstreamTests': parity['summary'], 'upstreamComplete': parity['complete'], 'timeoutExecution': parity['execution'],
              'workspace': workspace, 'outputSampleMatches': int(sample[1]), 'outputSampleUnexecutedSurrogates': 6,
              'fullHarvestRerun': False, 'extendedGroups': len(extended['rows']), 'extendedConsistentGroups': 39,
              'samplePeleZebraExactMatches': len(equality), 'fourEngineQualifiedCounts': coverage}
summary = {'releaseTag': tag, 'documentationFamilies': 33, 'renderGeometricRatio': render_ratio, 'parseGeometricRatio': parse_ratio,
           'rustFasterRenderFamilies': sum(row['render']['rustOverPele'] < 1 for row in documents),
           'rustFasterParseFamilies': sum(row['parse']['rustOverPele'] < 1 for row in documents),
           'inputOccurrences': success + rejection, 'uniqueSources': len({source for case in workloads for source in case['inputs']}),
           'successfulRenderOccurrences': success, 'expectedRejections': rejection, 'commonSampleInputs': common_count,
           'commonSampleGroups': len(cross['rows']), 'validation': validation}
for value in (metadata, validation, summary, resources):
    assert not re.search(r'/Users/|/home/|localhost|127\.0\.0\.1', json.dumps(value))
destination.mkdir()
for name, value in [('metadata.json', metadata), ('validation.json', validation), ('summary.json', summary), ('resources.json', resources)]:
    (destination / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
extended_directory = 'benchmark-' + extended['createdAt'][:10] + '-33-types' + suffix
upstream_directory = 'upstream-benchmark-' + upstream['createdAt'][:10] + '-33-types' + suffix
cross_directory = 'four-engine-benchmark-' + cross['createdAt'][:10] + suffix
sections = []
sections.append(f'''# 完整性能对比：{tag}

本报告针对当前提交的 Rust 实现重新测量，覆盖全部 33 类图表、39 组 Rust/Pele 工作负载、Pele 原脚本五组基准、四引擎共同样本，以及 39 组 CPU、RSS 和 Rust 分配统计。所有构建先完成，测量进程串行运行，没有并行编译或测试。

性能快照标签：`{tag}`。扩展基准时间：{extended['createdAt']}；四引擎基准时间：{cross['createdAt']}。日期和时钟记录使用 UTC；源码和二进制以 [SHA-256 清单](research/{directory}/metadata.json) 标识，标签指向包含实现、测试与报告的同一提交。

## 1. 主要结果与验收边界

| 指标 | 本次实际结果 |
| --- | --- |
| Rust 图类型覆盖 | 33/33 |
| Rust/Pele 扩展输入行为一致性 | 39/39 组；按工作负载累计 {success} 次成功渲染、{rejection} 次预期拒绝 |
| 33 类文档完整渲染 Rust/Pele 比值几何平均 | {render_ratio:.4f}；小于 1 表示 Rust 耗时较低 |
| 33 类文档仅解析 Rust/Pele 比值几何平均 | {parse_ratio:.4f} |
| 完整渲染更快的文档图类型 | {summary['rustFasterRenderFamilies']}/33；其余慢项原样保留 |
| 四引擎共同成功样本 | {common_count}/167 个 Sample，{len(cross['rows'])} 个图表组 |
| Pele/Rust Sample SVG 字节与尺寸完全一致 | 167/167 |
| 原始测试 1:1 验收 | {parity['summary']['passed']}/{parity['summary']['total']} 通过，{parity['summary']['skipped']} 跳过，{parity['summary']['unmapped']} 未映射，完整验收未通过 |
| 常规工作区回归 | {workspace['passed']} 通过、{workspace['failed']} 失败、{workspace['skipped']} 跳过 |
| 已提交输出样本差分 | 13,194/13,194 一致；另 6 条孤立代理项未执行、不计通过 |

三个覆盖指标分别统计：图类型接通、SVG/错误对照通过、原始测试逐项通过互不替代。该标签是性能快照，不能解读为全部 6,810 项测试已移植完成。本轮没有重新生成或运行本地缺失的 70,183 条采集全集，历史全集结果不算本次验证。

几何平均对每种图表等权，仅概括这 33 类文档工作负载，不是任意应用的总加速倍数。39 组共有 {success + rejection} 次输入出现、{summary['uniqueSources']} 份不同源码；流程图文档在原脚本组与 corpus 组中重复，不能相加当作独立测试覆盖。

## 2. 版本、测量方法与单位

- Pele 1.6.0，固定提交 `44da152bbb79b91f03e68016f42499bb25bf6e82`；重新执行其原始构建脚本生成生产 bundle。
- zebra-mermaid 0.1.0；{extended['runtime']['rustc']}；Node {extended['runtime']['node']}。Rust release、thin LTO、codegen-units=1，启用全部图类型。
- Mermaid.js {cross['versions']['mermaid']}、Chromium {cross['versions']['browser']}、Playwright {playwright_version}；mermaid-rs-renderer {cross['versions']['mmdr']}，固定提交 `{cross['versions']['mmdrCommit']}`。
- Rust/Pele 采用相同输入、CSS 输出、默认 Arial 字宽表，固定 now=1792368000000、UTC。完整 render 包含解析、布局和 SVG 生成，没有预解析或输出缓存命中。
- 扩展与四引擎基准每轮先预热 5 批，校准到计时片段至少约 10 ms，再采集至少 9 个样本且累计至少 300 ms。每轮取排序样本的下中位数，再取 3 个独立进程轮次的中位数；引擎顺序交替。三轮比值范围是观测范围，不是置信区间。
- 除资源表的特别说明外，单位为 **ms/整批输入**。同一行两侧输入完全相同；不同图类型的批次大小不同，不宜按绝对耗时排名。
- 比值一律为 **Rust/Pele**，小于 1 表示 Rust 更快；表中保留所有大于 1 的慢项。硬件、系统版本、主机名和个人路径未进入报告，结果不作跨平台性能保证。

## 3. Pele 原脚本五组基准

直接执行固定版本 `scripts/bench.mjs`；Rust 使用同一脚本求值获得的输入，并按原计时循环实现。一批预热，至少 300 ms 且至少 5 个样本，最多 2,000 个样本，偶数样本取上中位数。此表为原协议单轮结果，与后面的三轮扩展表分开。

| 工作负载 | 输入数 | Pele | zebra-mermaid |
| --- | ---: | ---: | ---: |''')
for row in upstream['rows']:
    sections.append(f"| {row['case']} | {row['diagrams']} | {row['pele']} | {row['zebra-mermaid']} |")
sections.append('''
118 个输入全部成功，计时外逐项比较完整 SVG 字符串与尺寸相等。第一行是全部流程图文档的整批耗时，其余每行一张；格式沿用上游显示精度。

## 4. 33 类、39 组完整渲染与解析

验证成功输入的 SVG UTF-8 FNV-1a-64 和尺寸；拒绝输入逐项比较错误码、消息、行、列。成功渲染与拒绝计时分离。完整字节对照另由 118 项原协议、167 项 Sample 和已提交样本回归提供。

### 完整渲染

| 工作负载 | 成功/总输入 | Pele ms | Rust ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | ---: | ---: | ---: | ---: |''')
for row in extended['rows']:
    timing = row['render']
    sections.append(f"| {row['name']} | {row['renderInputs']}/{row['diagrams']} | {timing['peleMs']:.5f} | {timing['rustMs']:.5f} | {timing['rustOverPele']:.3f} | {min(timing['ratios']):.3f}–{max(timing['ratios']):.3f} |")
sections.append('''
### 仅解析

| 工作负载 | 有效输入 | Pele ms | Rust ms | Rust/Pele | 三轮比值范围 |
| --- | ---: | ---: | ---: | ---: | ---: |''')
for row in extended['rows']:
    timing = row['parse']
    sections.append(f"| {row['name']} | {row['renderInputs']} | {timing['peleMs']:.5f} | {timing['rustMs']:.5f} | {timing['rustOverPele']:.3f} | {min(timing['ratios']):.3f}–{max(timing['ratios']):.3f} |")
sections.append('''
### 预期拒绝

| 工作负载 | 非法输入 | Pele ms | Rust ms | Rust/Pele |
| --- | ---: | ---: | ---: | ---: |''')
for row in extended['rows']:
    if row['rejectInputs']:
        timing = row['rejection']
        sections.append(f"| {row['name']} | {row['rejectInputs']} | {timing['peleMs']:.5f} | {timing['rustMs']:.5f} | {timing['rustOverPele']:.3f} |")
sections.append('''
## 5. 四引擎同输入横向对比

四个引擎各自尝试同一批 33 类、167 个 Pele Playground Sample，不改写源码迁就某个引擎。计时只使用四者共同成功的输入；无共同输入的类型仍列出。Mermaid 在真实浏览器中测量，启动、模块加载、验证、IPC 和写盘位于计时之外，DOM 清理计入调用；mmdr 的 options 克隆计入原生 API 调用。

请求 Arial 16；Pele/Rust 使用字宽表和 CSS 变量，Mermaid 使用浏览器字体度量，mmdr 使用默认原生字体度量、fast_text_metrics=false；后两者采用 dark 主题。四者布局、样式和语义实现不同，表格不表示输出视觉等价。第三方引擎仅属于独立开发驱动，不进入 Rust 生产依赖。

| 引擎 | 返回 SVG / 167 | 通过类型、XML 与尺寸检查 / 167 |
| --- | ---: | ---: |''')
for engine in engines:
    returned = sum(row.get('returnedSvg', False) for row in verification[engine].values())
    sections.append(f'| {names[engine]} | {returned} | {coverage[engine]} |')
sections.append(f'''
四者共同集合为 {common_count} 个输入、{len(cross['rows'])} 组。mmdr 将其他图表误识别为 Flowchart 的结果不计合格成功。Mermaid SVG 的 DOM 序列化只用于计时外验证；Info 无 viewBox 时使用实际 viewport。Pele/Rust 的 167 项 SVG 字节和尺寸相等，其他两者仅验证可渲染性与类型，完整语义一致性未获证明。

| 图类型 | 共同/原样本 | Mermaid.js ms | Pele ms | Rust ms | mmdr ms |
| --- | ---: | ---: | ---: | ---: | ---: |''')
for item in cross['coverage']:
    row = next((row for row in cross['rows'] if row['name'] == item['name']), None)
    values = [f"{row['timings'][engine]['ms']:.5f}" if row else '—' for engine in engines]
    sections.append('| ' + item['type'] + f" | {item['common']}/{item['total']} | " + ' | '.join(values) + ' |')
sections.append('''
### 全部输入覆盖

| 图类型 | 总数 | Mermaid.js 合格 | Pele 合格 | Rust 合格 | mmdr 合格 |
| --- | ---: | ---: | ---: | ---: | ---: |''')
for row in cross['coverage']:
    sections.append('| ' + row['type'] + f" | {row['total']} | " + ' | '.join(str(row['success'][engine]) for engine in engines) + ' |')
sections.append('''
## 6. 39 组 CPU 与进程峰值 RSS

这是独立资源实验，不与纯渲染计时混用。每组每引擎 3 个新进程，交替顺序，预热 5 批后运行至少 1 秒；每批包括完整 render、成功/拒绝计数及 SVG 字节统计。Agentflow reference 在此处理全部合法及非法输入，渲染与拒绝的纯耗时仍见第 4 节。

CPU 取子进程 user+system 时间，包含启动、模块/文件读取与预热；表中 CPU ms/批以该时间除以计时循环批数，含上述摊销，不是纯 render CPU 时间。RSS 是整个进程的系统记录峰值，包括运行时、代码和已加载数据，不能解读为单图申请量。每列分别取 3 轮中位数；MiB=1,048,576 bytes。

| 工作负载 | Pele CPU ms/批 | Rust CPU ms/批 | Pele 峰值 RSS MiB | Rust 峰值 RSS MiB |
| --- | ---: | ---: | ---: | ---: |''')
for row in resources['cases']:
    cpu = {engine: statistics.median((trial[engine]['cpuUserSeconds'] + trial[engine]['cpuSystemSeconds']) * 1000 / trial[engine]['batches'] for trial in row['runs']) for engine in ('pele', 'rust')}
    rss = {engine: statistics.median(trial[engine]['peakRssBytes'] for trial in row['runs']) / 2**20 for engine in ('pele', 'rust')}
    sections.append(f"| {row['case']} | {cpu['pele']:.5f} | {cpu['rust']:.5f} | {rss['pele']:.2f} | {rss['rust']:.2f} |")
sections.append('''
## 7. Rust 分配与峰值存活量

使用单独的带分配计数器可执行文件，避免影响正常耗时。5 批预热后观测一整批；输入/options 不计入，输出逐次释放。累计申请量含 realloc 的完整新请求，不是实际 RSS；峰值为相对批次开始的新增存活请求字节。没有同口径的 JS 分配计数，因此不计算 Rust/JS 分配倍率。

| 工作负载 | alloc 次数 | realloc 次数 | 累计申请 KiB | 新增存活峰值 KiB | 批后净保留 bytes |
| --- | ---: | ---: | ---: | ---: | ---: |''')
for row in resources['cases']:
    allocation = row['rustAllocation']
    sections.append(f"| {row['case']} | {allocation['allocations']} | {allocation['reallocations']} | {allocation['requestedBytes'] / 1024:.2f} | {allocation['peakAdditionalLiveBytes'] / 1024:.2f} | {allocation['retainedBytesAfterBatch']} |")
sections.append('''
39 组批次结束后净保留均为 0 字节。此结果仅覆盖这些批次，不是长期运行或所有输入的无泄漏证明；此前持续内存/系统泄漏实验保留在独立历史报告，本次没有将其计为重新执行。

## 8. 进程启动与分发体积

同一张四节点流程图，预运行后分别采样 30 次，交替进程启动顺序，完整 SVG 相等。墙钟含进程创建、模块加载、标准输入输出和一次渲染，文件系统缓存已热；Node 参照脚本不是 Pele 官方 CLI，不能将倍率视作渲染算法加速。

| 路径 | 中位 ms | 最小–最大 ms |
| --- | ---: | ---: |''')
for name, timing in extended['cold'].items():
    sections.append(f"| {'Rust CLI' if name == 'rust' else 'Node + Pele 参照脚本'} | {timing['medianMs']:.3f} | {timing['minMs']:.3f}–{timing['maxMs']:.3f} |")
sections.append(f'''
| 分发产物 | 原始 bytes | gzip bytes |
| --- | ---: | ---: |
| Rust 全图类型 CLI（未单独 strip） | {upstream['size']['rustCliBytes']} | {upstream['size']['rustCliGzipBytes']} |
| Pele minified JS bundle | {upstream['size']['peleMinifiedBytes']} | {upstream['size']['peleGzipBytes']} |

原生可执行文件与 JS bundle 的运行时依赖和分发形式不同，不据此推断部署体积优势。Rust 生产运行时不依赖 JS、Node、浏览器或外部布局程序；这些仅用于开发验证。

## 9. 可复现入口与原始证据

准备固定 Pele checkout 及其锁定依赖、固定 mmdr checkout、Rust 工具链、Node，以及与本报告版本一致的 Playwright/Chromium。浏览器模块可通过 `PLAYWRIGHT_MODULE` 指定，浏览器安装目录可通过 `PLAYWRIGHT_BROWSERS_PATH` 指定；机器路径不写入仓库。

```sh
git clone https://github.com/obsidianmd/pele.git .research/pele
git -C .research/pele checkout 44da152bbb79b91f03e68016f42499bb25bf6e82
npm ci --prefix .research/pele
pnpm --dir .research/pele/website install --frozen-lockfile
git clone https://github.com/1jehuang/mermaid-rs-renderer.git .research/mermaid-rs-renderer
git -C .research/mermaid-rs-renderer checkout {cross['versions']['mmdrCommit']}
npm install --prefix target/bench-browser playwright@{playwright_version}
target/bench-browser/node_modules/.bin/playwright install chromium
export PLAYWRIGHT_MODULE="$PWD/target/bench-browser/node_modules/playwright"
# 入口在计时全部结束后自动执行原始测试门禁、工作区与输出样本回归。
BENCHMARK_TAG={tag} BENCH_REPORT_SUFFIX=-reproduction python3 tools/bench/run-complete.py
```

入口依次构建、运行原脚本基准、运行三轮扩展基准、采集全部资源组、运行四引擎验证与计时，再执行测试验收和回归，最后生成本文及分项 Markdown。`BENCH_REPORT_SUFFIX` 选择新的证据目录，发布器拒绝覆盖已有四引擎/完整快照。逐项测试验收使用 `node tools/test-parity/check.mjs --require-complete`，当前因未全量移植而退出 1；入口仅在失败和待验证均为 0、确有未映射或跳过项时接受这个预期的非完整状态，仍在报告中明确未通过完整验收。常规回归使用 `cargo test --workspace --all-features -- --test-threads=1`。

- [完整汇总](research/{directory}/summary.json)、[版本/源码/驱动/二进制指纹](research/{directory}/metadata.json)、[测试与输出对照统计](research/{directory}/validation.json)。
- [39 组资源原始三轮数据及分配计数](research/{directory}/resources.json)。
- [33 类扩展计时与逐输入验证](research/{extended_directory}/comparison.json)，同目录保存输入及六份压缩原始采样报告。
- [Pele 原协议数据](research/{upstream_directory}/comparison.json)，同目录保存原控制台表、输入及完整 SVG 验证结果。
- [四引擎汇总](research/{cross_directory}/comparison.json)、[全部成功/失败及排除原因](research/{cross_directory}/verification.json)、[实际共同输入](research/{cross_directory}/common.json)，同目录保存十二份压缩计时记录及 SVG。

分项阅读：[Pele 原协议](BENCHMARK-PELE-PROTOCOL.zh-CN.md)、[33 类扩展报告](BENCHMARK.zh-CN.md)、[四引擎报告](BENCHMARK-FOUR-ENGINES.zh-CN.md)、[实施状态](IMPLEMENTATION-STATUS.md)、[原始测试逐项验收](UPSTREAM-TEST-PARITY.md)。
''')
(root / 'docs/BENCHMARK-COMPLETE.zh-CN.md').write_text('\n'.join(sections))
print(json.dumps(summary, ensure_ascii=False, indent=2))
