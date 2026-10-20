#!/usr/bin/env python3
"""Compare API-return and UTF-8-ready output contracts on every original Playground sample."""
import gzip, hashlib, json, math, os, pathlib, statistics, subprocess

root = pathlib.Path(__file__).resolve().parents[2]
destination = root / os.environ.get('ZM_OUTPUT_REPORT', 'target/output-benchmark')
destination.mkdir(parents=True, exist_ok=True)
read = lambda path: json.loads(path.read_text())
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
samples = read(root / 'docs/render-comparison/results.json')
cases = []
for row in samples['results']:
    if not cases or cases[-1]['type'] != row['type']:
        cases.append({'name': 'samples: ' + row['type'], 'type': row['type'], 'inputs': []})
    cases[-1]['inputs'].append({'id': row['id'], 'source': row['source']})
case_path = destination / 'cases.json'
case_path.write_text(json.dumps(cases, ensure_ascii=False) + '\n')
assert len(cases) == 33 and sum(len(c['inputs']) for c in cases) == 167
native = root / os.environ.get('ZM_OUTPUT_NATIVE', 'target/cross-bench-build/release/zebra-mermaid-cross-bench')
before = root / os.environ['ZM_OUTPUT_BEFORE'] if os.environ.get('ZM_OUTPUT_BEFORE') else None
reference = subprocess.check_output(['git', '-C', '.research/pele', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
assert reference == samples['metadata']['pele']['commit'], 'Reference differs from sample selection'
subprocess.run(['git', '-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src'], cwd=root, check=True)

def execute(engine, mode, name):
    file = destination / (name + '.json')
    if engine in ('before', 'rust'):
        command = [str(before if engine == 'before' else native), 'zebra', mode, str(case_path), str(file)]
    else:
        command = ['node', 'tools/bench/cross-pele.mjs', mode, str(case_path), str(file), engine]
    with (destination / (name + '.log')).open('w') as log:
        subprocess.run(command, cwd=root, env={**os.environ, 'TZ': 'UTC'}, stdout=log, stderr=subprocess.STDOUT, check=True)
    print(name, flush=True)
    return read(file)

verification = {}
for engine in (['before'] if before else []) + ['rust', 'api']:
    original = execute(engine, 'verify', engine + '-verify')
    verification[engine] = {r['id']: r for c in original['cases'] for r in c['results']}
for id, expected in verification['api'].items():
    assert expected['ok'], (id, expected)
    for engine in (['before'] if before else []) + ['rust']:
        actual = verification[engine][id]
        assert actual['ok'] and actual['svg'] == expected['svg'], (engine, id)
verification_summary = [{'id': id, 'svgSha256': hashlib.sha256(v['svg'].encode()).hexdigest()} for id, v in verification['api'].items()]
(destination / 'verification.json').write_text(json.dumps(verification_summary, indent=2) + '\n')
orders = [['before', 'rust', 'api', 'utf8'], ['utf8', 'api', 'rust', 'before'], ['rust', 'before', 'utf8', 'api']]
if not before:
    orders = [[engine for engine in order if engine != 'before'] for order in orders]
runs = []
for trial, order in enumerate(orders, 1):
    runs.append({engine: execute(engine, 'bench', f'{engine}-{trial}') for engine in order})
rows = []
for index, case in enumerate(cases):
    timings = {}
    for engine in orders[0]:
        values = [r[engine]['cases'][index] for r in runs]
        assert all(v['name'] == case['name'] and v['timing']['inputs'] == len(case['inputs']) for v in values)
        ms = [v['timing']['ms'] for v in values]
        timings[engine] = {'ms': statistics.median(ms), 'runsMs': ms}
    row = {'type': case['type'], 'inputs': len(case['inputs']), 'timings': timings,
           'apiSpeedup': timings['api']['ms'] / timings['rust']['ms'],
           'utf8Speedup': timings['utf8']['ms'] / timings['rust']['ms']}
    if before:
        row['afterOverBefore'] = timings['rust']['ms'] / timings['before']['ms']
    rows.append(row)
summary = {'apiSpeedupGeomean': math.exp(statistics.mean(math.log(r['apiSpeedup']) for r in rows)),
           'utf8SpeedupGeomean': math.exp(statistics.mean(math.log(r['utf8Speedup']) for r in rows))}
if before:
    summary['afterOverBeforeGeomean'] = math.exp(statistics.mean(math.log(r['afterOverBefore']) for r in rows))
sources = sorted((root / 'crates/zebra-mermaid/src').rglob('*.rs'))
sources += [root / p for p in ['Cargo.toml', 'Cargo.lock', 'crates/zebra-mermaid/Cargo.toml', 'tools/cross-bench/src/main.rs', 'tools/cross-bench/Cargo.lock', 'tools/bench/cross-pele.mjs', 'tools/bench/compare-output.py']]
report = {'upstream': {'version': samples['metadata']['pele']['version'], 'commit': reference},
          'runtime': {'node': subprocess.check_output(['node', '--version'], text=True).strip(), 'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()},
          'sourceHashes': {p.relative_to(root).as_posix(): sha(p) for p in sources},
          'binarySha256': sha(native), 'beforeBinarySha256': sha(before) if before else None,
          'inputSha256': sha(case_path), 'verified': len(verification_summary),
          'method': {'processRuns': 3, 'orders': orders, 'warmups': 5, 'calibrationMs': 10, 'minimumSamples': 9, 'minimumSamplingMs': 300,
                     'unit': 'ms per batch of identical sample inputs',
                     'api': 'render(source) followed by svg.length in Node; Rust returns an owned UTF-8 String.',
                     'utf8': 'Node render(source) followed by Buffer.from(svg, utf8); includes materialization, encoding and Buffer allocation. Rust String already owns contiguous UTF-8 bytes, so its measured time is shared by both comparisons.',
                     'excluded': 'process launch, imports, verification, disk I/O and warm-up; no parsing or result cache'},
          'summary': summary, 'rows': rows}
(destination / 'comparison.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(summary), flush=True)
