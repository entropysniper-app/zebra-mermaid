#!/usr/bin/env python3
"""Publish a portable snapshot of a completed paired benchmark and selected profiling run."""
import gzip, hashlib, json, math, os, pathlib, statistics, sys

root = pathlib.Path(__file__).resolve().parents[2]
if len(sys.argv) != 2:
    raise SystemExit('usage: publish-checkpoint.py <new-directory-under-docs/research>')
dest = (root / 'docs/research' / sys.argv[1]).resolve()
if dest.parent != (root / 'docs/research').resolve() or dest.exists():
    raise SystemExit('Choose a new direct child of docs/research; existing snapshots are immutable.')
paired = root / 'target/profiling/paired'
comparison = json.loads((paired / 'comparison.json').read_text())
resources = json.loads((root / 'target/profiling/resources.json').read_text())
cpu = json.loads((root / 'target/profiling/cpu/summary.json').read_text())
assert len(comparison['rows']) == 39 and all(r['consistent'] for r in comparison['rows'])
runs = {name: json.loads((paired / (name + '.json')).read_text()) for name in
        ['before-1', 'after-1', 'after-2', 'before-2', 'before-3', 'after-3']}
for run in runs.values():
    assert len(run['cases']) == 39
    for expected, actual in zip(runs['before-1']['cases'], run['cases']):
        for key in ('name', 'diagrams', 'verification', 'renderInputs', 'rejectInputs'):
            assert actual[key] == expected[key], (key, actual['name'])
for row in resources['cases']:
    assert len(row['runs']) == 3
    assert row['rustAllocation']['retainedBytesAfterBatch'] == 0
    for run in row['runs']:
        for key in ('renderedPerBatch', 'rejectedPerBatch', 'svgBytesPerBatch'):
            assert run['rust'][key] == run['pele'][key]
# Only these JSON data and compressed reports are public. Raw system sampler reports stay local.
def safe(data):
    text = json.dumps(data)
    assert '/Users/' not in text and '/home/' not in text and 'localhost' not in text
    return data
for value in [comparison, resources, cpu, *runs.values()]:
    safe(value)
dest.mkdir()
for name, value in [('comparison.json', comparison), ('resources.json', resources), ('cpu.json', cpu)]:
    (dest / name).write_text(json.dumps(value, indent=2) + '\n')
for name, value in runs.items():
    (dest / (name + '.json.gz')).write_bytes(gzip.compress(json.dumps(value).encode(), mtime=0))
hash_file = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
target = root / os.environ.get('CARGO_TARGET_DIR', 'target')
metadata = {
    'scope': 'Current implementation after Flowchart, Class and Sequence test expansion; paired against the frozen pre-optimization Rust baseline. This is a remeasurement, not an additional optimization claim.',
    'baselineCommit': '4761e84467016847a8f5f82f5d8ca69adb162f64',
    'inputSha256': hash_file(root / 'target/bench/cases.json'),
    'sourceFiles': [{'path': p.relative_to(root).as_posix(), 'sha256': hash_file(p)} for p in
                    sorted((root / 'crates/zebra-mermaid/src').rglob('*.rs'))],
    'driverFiles': [{'path': name, 'sha256': hash_file(root / name)} for name in [
        'crates/zebra-mermaid/examples/bench.rs', 'crates/zebra-mermaid/examples/profile.rs',
        'crates/zebra-mermaid/examples/support/profile.rs', 'tools/profiler/src/main.rs',
        'tools/bench/compare-optimization.py', 'tools/bench/profile-resources.py',
        'tools/bench/profile-cpu.py', 'tools/bench/profile-pele.mjs']],
    'binarySha256': {name: hash_file(path) for name, path in [
        ('beforeBenchmark', root / 'target/profiling/before/bench'),
        ('afterBenchmark', target / 'release/examples/bench'),
        ('afterCpu', target / 'release/examples/profile'),
        ('afterAllocator', target / 'release/zebra-mermaid-profiler')]},
    'measurement': 'Benchmark, allocation, CPU stacks and process resources are sampled separately; no test or compilation process runs concurrently.'
}
(dest / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
families = [r for r in comparison['rows'] if r['case'].startswith('corpus:') and r['case'] != 'corpus: agentflow-reference']
assert len(families) == 33
ratio = math.exp(statistics.mean(math.log(r['render']['afterOverBefore']) for r in families))
previous = json.loads((root / 'docs/research/profiling-2026-10-19/resources-before.json').read_text())
previous = {r['case']: r['rustAllocation'] for r in previous['cases']}
rows = {r['case']: r for r in comparison['rows']}
summary = {'groups': 39, 'documentationFamilies': 33, 'renderGeometricRatio': ratio,
           'renderReductionPercent': (1-ratio)*100, 'selectedResources': []}
for r in resources['cases']:
    current = r['rustAllocation']; before = previous[r['case']]
    summary['selectedResources'].append({
        'case': r['case'], 'renderBeforeMs': rows[r['case']]['render']['beforeMs'],
        'renderAfterMs': rows[r['case']]['render']['afterMs'],
        'requestedBytesReductionPercent': (1-current['requestedBytes']/before['requestedBytes'])*100,
        'peakAdditionalLiveBytesReductionPercent': (1-current['peakAdditionalLiveBytes']/before['peakAdditionalLiveBytes'])*100,
        'rustPeakRssMiB': statistics.median(run['rust']['peakRssBytes'] for run in r['runs'])/2**20,
        'pelePeakRssMiB': statistics.median(run['pele']['peakRssBytes'] for run in r['runs'])/2**20,
    })
(dest / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
