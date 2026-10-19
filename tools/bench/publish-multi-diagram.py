#!/usr/bin/env python3
"""Freeze multi-diagram CPU/allocation results after full correctness and ownership checks."""
import gzip, hashlib, json, math, pathlib, re, statistics
root=pathlib.Path(__file__).resolve().parents[2]
base=root/'target/profiling/multi-diagram'
dest=root/'docs/research/multi-diagram-optimization-2026-10-19'
assert not dest.exists(), 'Do not overwrite published measurements'
read=lambda p:json.loads(p.read_text())
paired=read(base/'paired/comparison.json')
assert len(paired['rows'])==39 and all(r['consistent']for r in paired['rows'])
families=[r for r in paired['rows']if r['case'].startswith('corpus:')and r['case']!='corpus: agentflow-reference']
assert len(families)==33
ratio=math.exp(statistics.mean(math.log(r['render']['afterOverBefore'])for r in families))
allocation=read(base/'allocations.json');assert len(allocation['rows'])==39
for row in allocation['rows']:
    assert row['before']['retainedBytesAfterBatch']==row['after']['retainedBytesAfterBatch']==0
memory=read(base/'memory/summary.json')
assert len(memory['runs'])==7 and len(memory['systemLeaks'])==5
for run in memory['runs']:
    assert run['result']['retainedBytesPerRound']==run['result']['retainedBlocksPerRound']==0
for r in memory['systemLeaks']:
    assert r['leaks']>=1 if r['mode']=='negative-control' else r['leaks']==0
log=(root/'.research/multi-full-regression.log').read_text()
records=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
workspace={k:sum(int(r[i])for r in records)for k,i in [('passed',1),('failed',2),('ignored',3)]}
assert workspace=={'passed':3017,'failed':0,'ignored':5},workspace
audit=read(root/'target/test-parity/report.json')
assert audit['execution']['isolated']and audit['summary']['failed']==audit['summary']['unverified']==0
stress=[c for c in audit['cases']if c['file']in ['tests/sequence/stress.test.ts','tests/class/stress.test.ts','tests/agentflow/stress.test.ts']]
assert len(stress)==172 and all(c['state']=='passed'for c in stress)
after=read(base/'after/metadata.json')
for entry in after['sources']:assert hashlib.sha256((root/entry['path']).read_bytes()).hexdigest()==entry['sha256']
files={'benchmark-comparison.json':paired,'allocations.json':allocation,'memory.json':memory,
       'stage-a-comparison.json':read(base/'paired-stage-a/comparison.json'),
       'original-tests.json':{'summary':audit['summary'],'execution':audit['execution'],'executables':audit['executables'],'stress':stress},
       'summary.json':{'baseline':'Previous Agentflow owner-index optimization; no mmdr production dependency.',
           'renderRatio':ratio,'renderReductionPercent':100*(1-ratio),'improvedFamilies':sum(r['render']['afterOverBefore']<1 for r in families),
           'workspace':workspace,'originalTests':audit['summary'],'stressPassed':172,
           'memorySourceCycles':sum(r['result']['inputsPerRound']*r['result']['rounds']for r in memory['runs'])}}
focused=read(base/'focused/comparison.json')
assert all(r['consistent']for r in focused['rows'])
files['focused-comparison.json']=focused
for phase in ['before','after']:
    files[phase+'-metadata.json']=read(base/phase/'metadata.json')
    files[phase+'-cpu.json']=read(base/phase/'cpu/summary.json')
for value in files.values():assert not re.search(r'/Users/|/home/|127\.0\.0\.1|localhost',json.dumps(value))
dest.mkdir()
for name,value in files.items():(dest/name).write_text(json.dumps(value,indent=2)+'\n')
for p in (base/'paired').glob('*.json'):
    if p.name=='comparison.json':continue
    (dest/(p.name+'.gz')).write_bytes(gzip.compress(p.read_bytes(),mtime=0))
for p in (base/'focused').glob('case-*.json'):
    (dest/('focused-'+p.name+'.gz')).write_bytes(gzip.compress(p.read_bytes(),mtime=0))

matrix='| 图表场景 | 优化前 ms | 优化后 ms | 耗时变化 |\n| --- | ---: | ---: | ---: |\n'
for r in families:
    v=r['render'];matrix+=f"| {r['case'].removeprefix('corpus: ')} | {v['beforeMs']:.5f} | {v['afterMs']:.5f} | {(v['afterOverBefore']-1)*100:+.2f}% |\n"
text=f'''# 多图表 CPU 优化与回归

本轮在上一阶段 Agentflow 所属 flow 索引的基础上，测量 33 类图表、39 组工作负载，并针对公共输出路径及多个绘制器优化。没有改动基准输入或原始测试阈值。

## 改动

1. `util::js::to_string` 对安全整数直接进行整数格式化。尺寸、字号和整数刻度不再先创建指数形式字符串再拆解重组；分数、超出安全范围、指数边界和非有限值继续走既有路径。新增由 Node 独立产生的 2,580 个参考值，覆盖边界、符号和零。
2. 默认 CSS 根样式改为借用静态字符串，权重查找与声明使用常量，减少每张图的短生命周期字符串。CSS 内容和输出顺序保持不变。
3. ER 关系标记只构造其基数类型使用的路径，直接写入一个输出缓冲区，避免为每个坐标、线段以及未使用的叉脚创建临时字符串。
4. 103 处 `push_str(&format!(...))` 改为直接 `write!` 到 String，覆盖公共 SVG、Sequence、Timeline、Journey、XYChart、Railroad、Block。保留相同格式串和参数，不调整几何或数值精度。

优化沿用 Pele 的可观察行为，同时利用 Rust 的借用与可复用缓冲区。未新增跨渲染缓存，也未新增生产 unsafe 或第三方运行时依赖。主题、CSS 配置与 Zebra C ABI 契约保持原样。

## 39 组配对结果

三轮独立进程，前后顺序交替；五批预热，至少 10 ms 校准，每轮至少九个样本且累计至少 300 ms。表中为整批成功输入的 render 时间：先取每轮样本中位数，再取三轮中位数。CPU 采样、分配计数、编译和测试均在计时结束后执行。

**39/39 组全部行为验证一致**，含 820 个成功渲染和 59 个预期拒绝。33 类文档 render 的等权耗时比几何平均为 **{ratio:.5f}**，即本次测量低约 **{100*(1-ratio):.2f}%**；其中 {sum(r['render']['afterOverBefore']<1 for r in families)}/33 类中位耗时下降。所有变慢项也保留，不将等权平均推广为任意实际应用的总体收益。

{matrix}
完整 39 行及 parse/rejection 数据见 [配对 JSON](research/multi-diagram-optimization-2026-10-19/benchmark-comparison.json)。第一阶段仅做数字格式、根样式与 ER 标记时测得约 5.96% 的等权下降；该中间数据另存，不能与最终下降百分比相加。

## CPU 与分配证据

本轮对 ER、Sequence、Block、XYChart、Journey、Railroad、Timeline、Class、Info 九组分别采样优化前后 CPU 栈。采样为 5 秒、1 ms 间隔，计数包含子调用及可能的预热，不是互斥 CPU 百分比。原始系统日志保留在忽略目录，公开文件只包含符号汇总和哈希。

| 场景 | 分配次数变化 | 累计申请字节变化 | 额外存活峰值变化 |
| --- | ---: | ---: | ---: |
'''
for name in ['corpus: er-docs','corpus: sequence-docs','corpus: block-docs','corpus: xychart-docs','corpus: journey-docs','corpus: railroad-docs','corpus: timeline-docs','corpus: info-docs']:
    row=next(r for r in allocation['rows']if r['case']==name)
    values=[100*(row['after'][k]/row['before'][k]-1) for k in ['allocations','requestedBytes','peakAdditionalLiveBytes']]
    text+='| '+name.removeprefix('corpus: ')+' | '+' | '.join(f'{v:+.2f}%'for v in values)+' |\n'
text+=f'''
分配数据为独立计数器预热五批后的单批测量。累计申请量包括 realloc 的新请求，不是 RSS；每组结束净新增存活字节均为 0。全部 39 组的前后原始计数见 [分配记录](research/multi-diagram-optimization-2026-10-19/allocations.json)。

## 内存与正确性回归

- 内存探针重新执行常规渲染 1,000 轮、模型复用/缓存/C ABI 各 200 轮，以及三组大型输入；合计 **{files['summary.json']['memorySourceCycles']:,} 个输入处理轮次**。每轮净保留均为 0 字节、0 分配块。
- 系统 leaks 对 render、parsed、cache、ffi 四条路径均报告 0 泄漏；独立故意遗留分配的控制进程成功检出。仅对本次场景作出未发现泄漏的结论，RSS 波动另存 [内存报告](research/multi-diagram-optimization-2026-10-19/memory.json)。
- 全 features 工作区和采集全集：**3,017 通过、0 失败、5 原有跳过**，含 70,183 条可表示采集输入、主题与 ABI 回归。
- 逐项独立超时审计：**2,629/6,810 通过，0 失败，5 跳过，4,176 未映射**；其中全部 172 项已迁移原始压力测试通过。新增数字格式项目回归不计入原始测试通过数。
- 四引擎比较另外验证 Pele/zebra 的 167 个 Sample SVG 字节与尺寸一致；与 Mermaid.js/mmdr 的共同输入测量见 [四引擎横向报告](BENCHMARK-FOUR-ENGINES.zh-CN.md)，不混用其样本数与本表 879 个输入。

## 复现和数据

```sh
node tools/bench/generate-number-reference.mjs --check
cargo test -p zebra-mermaid --all-features --test number_format
cargo build --release -p zebra-mermaid --all-features --example bench --example profile --example bench_upstream
cargo build --release -p zebra-mermaid-cli -p zebra-mermaid-profiler --bins
# 改动前先冻结旧版 bench；当前版使用新的输出目录。
ZM_BENCH_BEFORE=target/profiling/multi-diagram/before/bench \\
ZM_BENCH_OUTPUT=target/profiling/new-comparison python3 tools/bench/compare-optimization.py
ZM_MEMORY_OUTPUT=target/profiling/new-memory python3 tools/bench/check-memory.py
ZM_FULL_HARVEST=1 cargo test --workspace --all-features
node tools/test-parity/check.mjs --run --isolate
```

支持 `CARGO_TARGET_DIR`。原始测量见 [摘要](research/multi-diagram-optimization-2026-10-19/summary.json)、[优化前 CPU](research/multi-diagram-optimization-2026-10-19/before-cpu.json)、[优化后 CPU](research/multi-diagram-optimization-2026-10-19/after-cpu.json)、[源码和二进制指纹](research/multi-diagram-optimization-2026-10-19/after-metadata.json)。同目录保存优化前元数据、六份完整计时、所有内存与原始测试审计摘要。
'''
(root/'docs/PROFILING-MULTI-DIAGRAM.zh-CN.md').write_text(text)
focused_text='''## 波动与逐组复核

首次整轮配对存在较大波动。例如 `docs: all flowchart examples` 与 `corpus: flowchart-docs` 的输入完全相同，但前者约低 3.76%，后者约高 15.51%；Usecase 的优化后单轮中位数分别约为 1.651 / 1.223 / 0.829 ms。因此另外对所有初测回退至少 3% 的场景，加 Flowchart/ER/Sequence 对照，逐组交替运行前后版本各五个独立进程。

复核使用原来的冻结 benchmark 二进制、同一输入顺序、选项与单轮采样规则，仅缩短前后测量的时间间隔。所有初测和复核记录同时保留；不把复核选择性合并为一个新的总体几何平均，也不将波动直接归因于某项实现。

| 场景 | 复核前 ms | 复核后 ms | 耗时变化 |
| --- | ---: | ---: | ---: |
'''
for r in focused['rows']:
    t=r['render'];focused_text+=f"| {r['case']} | {t['beforeMs']:.5f} | {t['afterMs']:.5f} | {(t['afterOverBefore']-1)*100:+.2f}% |\n"
focused_text+='\n[复核汇总与方法](research/multi-diagram-optimization-2026-10-19/focused-comparison.json)，同目录保存每次进程的完整样本。\n\n'
text=text.replace('## CPU 与分配证据',focused_text+'## CPU 与分配证据')
(root/'docs/PROFILING-MULTI-DIAGRAM.zh-CN.md').write_text(text)
print(json.dumps(files['summary.json'],indent=2))
