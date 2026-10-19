#!/usr/bin/env python3
"""Save the four-engine benchmark, all coverage outcomes and portable provenance."""
import gzip, hashlib, json, os, pathlib, re, shutil
root=pathlib.Path(__file__).resolve().parents[2]
source=root/'target/cross-bench'
read=lambda name:json.loads((source/name).read_text())
report=read('comparison.json');verification=read('verification.json')
suffix=os.environ.get('BENCH_REPORT_SUFFIX','')
assert not suffix or re.fullmatch(r'-[a-z0-9-]+',suffix), 'Invalid report suffix'
directory='four-engine-benchmark-'+report['createdAt'][:10]+suffix
destination=root/'docs/research'/directory
assert not destination.exists(), 'Do not overwrite published evidence'
equality=read('pele-zebra-equality.json');assert len(equality)==167 and all(r['equal']for r in equality)
for row in report['rows']:
    coverage=next(c for c in report['coverage']if c['name']==row['name'])
    assert row['inputs']==coverage['common']>0
    for e in ['mermaid','pele','zebra','mmdr']:
        assert all(verification[e][id]['ok']for id in coverage['commonIds'])
for e in ['mermaid','pele','zebra','mmdr']:
    assert len(verification[e])==167
files=['comparison.json','verification.json','coverage.json','cases.json','common.json','pele-zebra-equality.json']
for name in files:
    assert not re.search(r'/Users/|/home/|127\.0\.0\.1|localhost', (source/name).read_text()),name
destination.mkdir()
for name in files:shutil.copy2(source/name,destination/name)
for e in ['mermaid','pele','zebra','mmdr']:
    for trial in range(1,4):
        name=f'{e}-{trial}.json';data=(source/name).read_bytes();json.loads(data)
        (destination/(name+'.gz')).write_bytes(gzip.compress(data,mtime=0))
shutil.copytree(source/'svg',destination/'svg')
paths=[root/'tools/cross-bench/Cargo.toml',root/'tools/cross-bench/Cargo.lock',root/'tools/cross-bench/src/main.rs']
paths.extend(root/'tools/bench'/name for name in ['run-cross.py','cross-pele.mjs','cross-mermaid.mjs'])
metadata={'files':[{'path':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in paths],
    'nativeBinarySha256':report['nativeBinarySha256'],'sourceSelection':'Original pinned Pele Playground sampleGroups: 33 groups, 167 samples. No sample source rewriting.',
    'mmdrSource':'https://github.com/1jehuang/mermaid-rs-renderer/tree/'+report['versions']['mmdrCommit'],
    'nativeBuild':'Both native libraries in one isolated development driver, release, thin LTO, codegen-units=1; mmdr default-features=false (SVG only), zebra all-diagrams. Runtime renderer has no mmdr dependency.'}
(destination/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
names={'mermaid':'Mermaid.js','pele':'Pele','zebra':'zebra-mermaid','mmdr':'mermaid-rs-renderer'}
engines=list(names)
totals={e:sum(r['ok']for r in verification[e].values())for e in engines}
returned={e:sum(r.get('returnedSvg',False)for r in verification[e].values())for e in engines}
matrix='| 图表场景 | 共同样本 / 原样本 | Mermaid.js ms | Pele ms | zebra-mermaid ms | mermaid-rs-renderer ms |\n| --- | ---: | ---: | ---: | ---: | ---: |\n'
for c in report['coverage']:
    r=next((r for r in report['rows']if r['name']==c['name']),None)
    values=[f"{r['timings'][e]['ms']:.5f}" if r else '—' for e in engines]
    matrix+='| '+c['type']+' | '+str(c['common'])+' / '+str(c['total'])+' | '+' | '.join(values)+' |\n'
coverage_table='| 引擎 | 返回 SVG / 167 | 通过类型与 SVG 检查 / 167 |\n| --- | ---: | ---: |\n'
for e in engines:coverage_table+=f'| {names[e]} | {returned[e]} | {totals[e]} |\n'
common=sum(c['common']for c in report['coverage'])
text=f'''# 四引擎性能横向对比

测量时间：{report['createdAt']}。

本次实际比较 Mermaid.js、Pele、zebra-mermaid 和 [mermaid-rs-renderer](https://github.com/1jehuang/mermaid-rs-renderer/tree/{report['versions']['mmdrCommit']})。输入为固定 Pele Playground 的 **33 类、167 个 Sample**，四个引擎逐项尝试完全相同的源码，不为某个引擎改写语法。

## 版本与口径

- Mermaid.js {report['versions']['mermaid']}，Chromium {report['versions']['browser']}，真实浏览器中调用 `mermaid.render`。
- Pele {report['versions']['pele']}，固定提交 `{report['versions']['peleCommit']}`；zebra-mermaid {report['versions']['zebra']}，本轮优化后的 Rust 库。
- mermaid-rs-renderer {report['versions']['mmdr']}，固定提交 `{report['versions']['mmdrCommit']}`；直接调用原版 `render_with_options`，保留默认原生字体测量，`fast_text_metrics=false`。
- 两个 Rust 库使用相同 release / thin LTO / codegen-units=1 编译配置。第三方引擎仅进入独立开发驱动，不进入 zebra-mermaid 生产依赖。{report['versions']['rustc']}；Node {report['versions']['node']}。
- 请求 Arial 16；Pele/Rust 保留 Pele CSS 变量，Mermaid.js/mmdr 使用 dark 主题；输入自身指令保留。各引擎的字体测量、布局和样式实现不同，输出尺寸和视觉效果不保证一致。

计时为**完整 parse + layout + SVG 生成的稳态墙钟耗时**，单位为同一行全部共同样本的 ms。每轮预热五批，校准到至少 10 ms，每次采样至少九个样本且累计至少 300 ms，三个独立进程轮次交错引擎顺序。每轮取样本中位数，再取三轮中位数。

浏览器启动、模块加载、字体首次预热、验证、跨进程通信及写盘不计时；Mermaid 的测量在浏览器内部完成。DOM 清理计入 Mermaid 调用循环；mmdr 的 API 消费 options，因此配置克隆计入其调用成本。未测 CLI 冷启动，本表不能用于宣称 CLI 启动速度。

## 支持范围与比较集合

{coverage_table}
最终共同集合为 **{common} 个 Sample、{len(report['rows'])} 个图表组**。每行四个引擎都只测该行同一组共同输入；没有共同输入的行保留 `—`。原始 167 项的成功、失败和排除原因全部保存。

SVG 检查包含 XML、有限正尺寸，以及 mmdr 的 `DiagramKind` 是否符合输入图表类型。验证中发现 mmdr 会将部分带 `-beta` 的其他图表头识别为 Flowchart；这些结果记录为“返回 SVG、类型不匹配”，不计入合格成功或耗时排名。Mermaid 的 HTML 形式 SVG 在计时外通过 DOM 序列化；Info 没有 viewBox，使用浏览器实际 viewport 尺寸。

Pele 与 zebra-mermaid 的 **167/167 SVG 字节与尺寸完全一致**。另外两个引擎只进行了可渲染性与类型校验，不能由此断言所有节点、边、文字、布局或像素语义完全等价。可在原始 SVG 中进一步核对。

## 同输入耗时

{matrix}
只应在同一行内比较。单个样例数量、图复杂度、实现的布局和字体行为会影响比值，不把某一行的倍率推广为所有 Mermaid 输入的性能结论。

## 覆盖明细

| 图表场景 | 原样本数 | Mermaid.js 合格 | Pele 合格 | zebra-mermaid 合格 | mmdr 合格 |
| --- | ---: | ---: | ---: | ---: | ---: |
'''
for c in report['coverage']:text+='| '+c['type']+' | '+str(c['total'])+' | '+' | '.join(str(c['success'][e])for e in engines)+' |\n'
text+=f'''
## 复现与证据

```sh
git clone https://github.com/1jehuang/mermaid-rs-renderer.git .research/mermaid-rs-renderer
git -C .research/mermaid-rs-renderer checkout 3726ccbffe0e8032361eb9668694b24f77858060
CARGO_TARGET_DIR=target/cross-bench-build cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml
# 需要固定 Mermaid.js 12.1.0 和已安装 Chromium 的 Playwright。
# PLAYWRIGHT_MODULE / PLAYWRIGHT_BROWSERS_PATH 可指向自己的安装目录。
python3 tools/bench/run-cross.py
python3 tools/bench/publish-cross.py
```

结果见 [汇总及版本](research/{directory}/comparison.json)、[逐输入错误与 SVG 路径](research/{directory}/verification.json)、[共同输入](research/{directory}/common.json)、[全部原始输入](research/{directory}/cases.json)、[源码/依赖/二进制指纹](research/{directory}/metadata.json)。同目录有十二份压缩原始计时、SVG 文件与 Pele/Rust 一致性记录。

这份报告与 [Pele 原脚本五组协议](BENCHMARK-PELE-PROTOCOL.zh-CN.md)、[39 组 Rust 优化前后测量](PROFILING-MULTI-DIAGRAM.zh-CN.md) 使用不同的输入集合，分别保存，不拼接比值。图表渲染成功也不增加 6,810 项原始测试的迁移通过数。
'''
(root/'docs/BENCHMARK-FOUR-ENGINES.zh-CN.md').write_text(text)
readme=root/'README.md';body=readme.read_text()
section='<!-- four-engine-benchmark:start -->\n## 四引擎性能对比\n\n同一批 Pele Playground Sample，比较 Mermaid.js / Pele / zebra-mermaid / mermaid-rs-renderer。下面每行单位为共同输入整批的 ms，三轮独立进程中位数，启动与预热不计时。\n\n'+matrix+f'\n四者共同通过类型与 SVG 检查的样本为 **{common}/167**；mmdr 对其他图表的 Flowchart 误识别单独排除，完整覆盖率、失败原因、测量限制与源码版本见 [四引擎报告](docs/BENCHMARK-FOUR-ENGINES.zh-CN.md)。SVG 返回成功不等于四者视觉或语义完全一致。\n<!-- four-engine-benchmark:end -->\n'
if '<!-- four-engine-benchmark:start -->' in body:body=re.sub(r'<!-- four-engine-benchmark:start -->.*?<!-- four-engine-benchmark:end -->\n',lambda _:section,body,flags=re.S)
else:body=body.replace('## 文档\n',section+'\n## 文档\n')
readme.write_text(body)
print(json.dumps({'common':common,'groups':len(report['rows']),'returnedSvg':returned,'eligible':totals},ensure_ascii=False))
