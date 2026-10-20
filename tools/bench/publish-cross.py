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
paths.extend(sorted((root/'crates/zebra-mermaid/src').rglob('*.rs')))
paths.extend([root/'Cargo.toml',root/'Cargo.lock',root/'crates/zebra-mermaid/Cargo.toml'])
metadata={'files':[{'path':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in paths],
    'nativeBinarySha256':report['nativeBinarySha256'],'sourceSelection':'Original pinned Pele Playground sampleGroups: 33 groups, 167 samples. No sample source rewriting.',
    'mmdrSource':'https://github.com/1jehuang/mermaid-rs-renderer/tree/'+report['versions']['mmdrCommit'],
    'nativeBuild':'Both native libraries in one isolated development driver, release, thin LTO, codegen-units=1; mmdr default-features=false (SVG only), zebra all-diagrams. Runtime renderer has no mmdr dependency.'}
(destination/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
names={'mermaid':'Mermaid.js','pele':'Pele','zebra':'zebra-mermaid','mmdr':'mermaid-rs-renderer'}
engines=list(names)
totals={e:sum(r['ok']for r in verification[e].values())for e in engines}
returned={e:sum(r.get('returnedSvg',False)for r in verification[e].values())for e in engines}
matrix='| Diagram | Shared / total samples | Mermaid.js (ms) | Pele (ms) | zebra-mermaid (ms) | mermaid-rs-renderer (ms) |\n| --- | ---: | ---: | ---: | ---: | ---: |\n'
for c in report['coverage']:
    r=next((r for r in report['rows']if r['name']==c['name']),None)
    values=[f"{r['timings'][e]['ms']:.5f}" if r else '—' for e in engines]
    matrix+='| '+c['type']+' | '+str(c['common'])+' / '+str(c['total'])+' | '+' | '.join(values)+' |\n'
coverage_table='| Renderer | Returned SVG / 167 | Passed type and SVG checks / 167 |\n| --- | ---: | ---: |\n'
for e in engines:coverage_table+=f'| {names[e]} | {returned[e]} | {totals[e]} |\n'
common=sum(c['common']for c in report['coverage'])
text=f'''# Performance comparison

Measured: {report['createdAt']}.

The current library is compared with three other renderers using the same **167 Playground samples across 33 diagram types**. All four engines attempt every original input without syntax rewriting. Timing includes only samples that pass type and SVG validation in all four engines.

## Versions

| Renderer | Version | Execution |
| --- | --- | --- |
| zebra-mermaid | {report['versions']['zebra']} | Native Rust, release, thin LTO, codegen-units=1 |
| Pele | {report['versions']['pele']} | Node, commit `{report['versions']['peleCommit']}` |
| Mermaid.js | {report['versions']['mermaid']} | Chromium {report['versions']['browser']} |
| mermaid-rs-renderer | {report['versions']['mmdr']} | Native Rust, release, thin LTO, codegen-units=1; commit `{report['versions']['mmdrCommit']}` |

Runtime versions: {report['versions']['rustc']}; Node {report['versions']['node']}. The two native libraries share one development benchmark driver and matching optimization settings. The comparison renderer is not a dependency of the production library.

## Method

- Measure full parsing, layout, and SVG generation, with no parsed-model or output-cache reuse.
- Use identical inputs within each row. Values are milliseconds for the **entire batch**, not per diagram.
- Run five warm-up batches, calibrate each timed segment to at least 10 ms, then collect at least nine samples and 300 ms per group.
- Run three independent processes per engine, alternating engine order. Take the lower median within each run, then the median of the three runs.
- Exclude process/browser startup, imports, font warm-up, validation, IPC, and disk output. Mermaid DOM cleanup and mermaid-rs-renderer's owned-options clone remain inside their loops.
- Request Arial 16. Rust/Pele use the same Arial metrics and CSS output; Mermaid uses browser measurements and a dark theme; mermaid-rs-renderer uses its default native font measurement with `fast_text_metrics=false`. Authored directives remain active. The clock is fixed to 2026-10-19 00:00 UTC where supported.

These are steady-state rendering measurements. Layout and font behavior differ between engines, so timings are not a claim of identical work or pixel equivalence across all four engines. Compare values within a row; batch sizes and diagram complexity differ across rows.

## Coverage

{coverage_table}
The common set contains **{common}/167 samples across {len(report['rows'])} diagram types**. A dash means no shared eligible input, not a zero rendering cost. The complete success, failure, and exclusion records are retained.

SVG validation requires valid XML and positive finite dimensions. The native comparison renderer must also report the expected diagram kind: inputs misidentified as flowcharts are recorded as returned SVG but excluded from eligible success and timing. Mermaid HTML output is serialized through the DOM outside timing; its Info output uses the browser viewport when no viewBox is present.

Rust/Pele SVG bytes and dimensions match on **167/167 samples**. The other engines are checked for renderability and type; this does not establish full semantic or visual equivalence.

## Results

{matrix}
## Per-engine coverage

| Diagram | Samples | Mermaid.js | Pele | zebra-mermaid | mermaid-rs-renderer |
| --- | ---: | ---: | ---: | ---: | ---: |
'''
for c in report['coverage']:text+='| '+c['type']+' | '+str(c['total'])+' | '+' | '.join(str(c['success'][e])for e in engines)+' |\n'
text+=f'''
## Reproduce

Use the reference revisions listed above, the locked website dependencies, and Playwright with Chromium installed. From the repository root:

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/run-cross.py
BENCH_REPORT_SUFFIX=-pele-1-7-0 python3 tools/bench/publish-cross.py
```

`PLAYWRIGHT_MODULE` and `PLAYWRIGHT_BROWSERS_PATH` can point to a separate browser installation. Publishing requires a fresh report suffix to preserve existing measurement evidence.

[Summary and versions](research/{directory}/comparison.json) · [Per-input outcomes](research/{directory}/verification.json) · [Shared inputs](research/{directory}/common.json) · [All inputs](research/{directory}/cases.json) · [Source and binary fingerprints](research/{directory}/metadata.json)

The same directory contains twelve compressed raw timing reports, SVG output, and Rust/Pele equality records. Benchmark success is separate from [original-test acceptance](UPSTREAM-TEST-PARITY.md).
'''
(root/'docs/BENCHMARK.md').write_text(text)
readme=root/'README.md';body=readme.read_text()
section='<!-- four-engine-benchmark:start -->\n## Performance\n\nCurrent-version results on identical inputs, measured as **milliseconds per complete batch**. Each value is the median of three independent runs; parsing, layout, and SVG generation are included, while startup and warm-up are excluded. Lower is faster within the same row.\n\n'+matrix+f'\nThe shared benchmark set contains **{common}/167 samples across {len(report["rows"])} diagram types**. A dash means no input passed type and SVG checks in all four engines. Layout and font measurement differ between engines. See the [benchmark report](docs/BENCHMARK.md) for versions, coverage, methodology, and raw results.\n<!-- four-engine-benchmark:end -->\n'
assert '<!-- four-engine-benchmark:start -->' in body, 'README is missing performance comparison markers'
body=re.sub(r'<!-- four-engine-benchmark:start -->.*?<!-- four-engine-benchmark:end -->\n',lambda _:section,body,flags=re.S)
readme.write_text(body)
print(json.dumps({'common':common,'groups':len(report['rows']),'returnedSvg':returned,'eligible':totals},ensure_ascii=False))
