#!/usr/bin/env python3
"""Publish the current output-contract comparison without replacing past measurement evidence."""
import datetime, gzip, hashlib, json, os, pathlib, re, shutil
root=pathlib.Path(__file__).resolve().parents[2]
source=root/os.environ.get('ZM_OUTPUT_REPORT','target/output-benchmark')
report=json.loads((source/'comparison.json').read_text())
suffix=os.environ.get('BENCH_REPORT_SUFFIX','')
assert re.fullmatch(r'-[a-z0-9-]+',suffix), 'Set a fresh BENCH_REPORT_SUFFIX'
directory='output-benchmark-'+datetime.datetime.now(datetime.timezone.utc).date().isoformat()+suffix
destination=root/'docs/research'/directory
assert not destination.exists(), 'Do not overwrite published evidence'
assert report['verified']==167 and len(report['rows'])==33
for name,expected in report['sourceHashes'].items():
    assert hashlib.sha256((root/name).read_bytes()).hexdigest()==expected, ('Source changed after measurement',name)
destination.mkdir()
for name in ['comparison.json','cases.json','verification.json']:
    raw=(source/name).read_bytes()
    assert b'/Users/' not in raw and b'/home/' not in raw,name
    shutil.copy2(source/name,destination/name)
for engine in report['method']['orders'][0]:
    for trial in range(1,4):
        name=f'{engine}-{trial}.json'
        (destination/(name+'.gz')).write_bytes(gzip.compress((source/name).read_bytes(),mtime=0))
rows='| Diagram | Samples | Pele API (ms) | Pele UTF-8 (ms) | Rust (ms) | API speedup | UTF-8 speedup |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n'
for row in report['rows']:
    t=row['timings']
    rows+=f'| {row["type"]} | {row["inputs"]} | {t["api"]["ms"]:.5f} | {t["utf8"]["ms"]:.5f} | {t["rust"]["ms"]:.5f} | {row["apiSpeedup"]:.2f}× | {row["utf8Speedup"]:.2f}× |\n'
summary=report['summary']
text=f'''# Rust/Pele output contracts

Current implementations on **167 unchanged samples across all 33 diagram types**. Pele {report['upstream']['version']}, commit `{report['upstream']['commit']}`; {report['runtime']['node']}; {report['runtime']['rustc']}.

Two endpoints are measured separately:

- **API return:** `render(source)` followed by reading the SVG length. V8 may return a concatenation tree without materializing one contiguous buffer.
- **UTF-8 ready:** Node additionally obtains `Buffer.from(svg, 'utf8')`. This includes materialization, encoding, and Buffer allocation. Rust already returns an owned contiguous UTF-8 `String`; its measurement is shared by both comparisons.

The second endpoint measures a different output contract, not string flattening in isolation. Both exclude disk I/O, process startup, imports, validation, and warm-up. Neither uses a parsed-model or output cache. All 167 complete SVG strings are checked against the independent reference before timing.

Each engine runs in three independent processes with alternating order. Five warm-up batches, at least 10 ms calibration, and at least nine samples plus 300 ms per group. Values are lower medians within runs, then the median of three runs, in milliseconds per complete row batch. The two endpoints use the same sources, Arial metrics, Arial 16, CSS output, and fixed UTC clock.

The equal-weight geometric mean speedup across the 33 types is **{summary['apiSpeedupGeomean']:.2f}× for API return** and **{summary['utf8SpeedupGeomean']:.2f}× for UTF-8-ready output**. These averages describe this sample set, not an arbitrary application's workload.

{rows}
## Evidence and reproduction

[Summary, source hashes and binary hashes](research/{directory}/comparison.json) · [Exact inputs](research/{directory}/cases.json) · [Verified SVG hashes](research/{directory}/verification.json). The same directory contains each process's raw sample timings.

```sh
cargo build --release --locked --manifest-path tools/cross-bench/Cargo.toml --target-dir target/cross-bench-build
python3 tools/bench/compare-output.py
BENCH_REPORT_SUFFIX=-new-measurement python3 tools/bench/publish-output.py
```

`ZM_OUTPUT_REPORT` selects another measurement directory. `ZM_OUTPUT_NATIVE` selects the current native driver. Optionally set `ZM_OUTPUT_BEFORE` to a frozen native driver to retain a paired optimization comparison in the raw data; the README shows only current engine results.
'''
(root/'docs/OUTPUT-BENCHMARK.md').write_text(text)
readme=root/'README.md';body=readme.read_text()
section=f'''<!-- output-contract-benchmark:start -->
Across all **167 samples and 33 types**, the equal-weight geometric mean Rust/Pele speedup is **{summary['apiSpeedupGeomean']:.2f}× at API return** and **{summary['utf8SpeedupGeomean']:.2f}× when the output must be ready as contiguous UTF-8 bytes**. The latter includes Node's encoding and Buffer allocation; Rust already returns UTF-8. See the [output-contract benchmark](docs/OUTPUT-BENCHMARK.md) for the complete table and method.
<!-- output-contract-benchmark:end -->'''
if '<!-- output-contract-benchmark:start -->' in body:
    body=re.sub(r'<!-- output-contract-benchmark:start -->.*?<!-- output-contract-benchmark:end -->',lambda _:section,body,flags=re.S)
else:
    body=body.replace('<!-- four-engine-benchmark:end -->','<!-- four-engine-benchmark:end -->\n\n'+section)
readme.write_text(body)
print(json.dumps({'directory':directory,**summary}))
