#!/usr/bin/env python3
"""Alternate frozen baseline and current Rust drivers, preserving the extended benchmark protocol."""
import json, os, pathlib, statistics, subprocess

root = pathlib.Path(__file__).resolve().parents[2]
dest = root / os.environ.get('ZM_BENCH_OUTPUT', 'target/profiling/paired')
dest.mkdir(parents=True, exist_ok=True)
target = root / os.environ.get('CARGO_TARGET_DIR', 'target')
binaries = {'before': root / os.environ.get('ZM_BENCH_BEFORE', 'target/profiling/before/bench'),
            'after': root / os.environ.get('ZM_BENCH_AFTER', str(target / 'release/examples/bench'))}
runs = []
for trial in range(3):
    results = {}
    for engine in (['before', 'after'] if trial % 2 == 0 else ['after', 'before']):
        path = dest / ('%s-%d.json' % (engine, trial + 1))
        subprocess.run([str(binaries[engine]), str(path)], cwd=root, check=True, env={**os.environ, 'TZ': 'UTC'})
        results[engine] = json.loads(path.read_text())
        print('trial %d: %s' % (trial + 1, engine), flush=True)
    assert results['before']['enabledTypes'] == results['after']['enabledTypes']
    runs.append(results)
rows = []
for index, original in enumerate(runs[0]['before']['cases']):
    row = {'case': original['name'], 'diagrams': original['diagrams'], 'consistent': True}
    for run in runs:
        for engine in ('before', 'after'):
            case = run[engine]['cases'][index]
            for key in ('name', 'diagrams', 'verification', 'renderInputs', 'rejectInputs'):
                assert case[key] == original[key], (original['name'], key)
    for operation in ('render', 'parse', 'rejection'):
        if original.get(operation) is None:
            continue
        before = [r['before']['cases'][index][operation]['ms'] for r in runs]
        after = [r['after']['cases'][index][operation]['ms'] for r in runs]
        row[operation] = {'beforeMs': statistics.median(before), 'afterMs': statistics.median(after),
                          'afterOverBefore': statistics.median(after) / statistics.median(before),
                          'beforeRuns': before, 'afterRuns': after}
    rows.append(row)
(dest / 'comparison.json').write_text(json.dumps({'method': 'Original extended Rust bench.rs driver: 5 warm-ups, >=10ms calibration, >=9 samples and >=300ms per operation per run. Three independent processes per version, alternating order. Every verification record must match across all six runs.', 'rows': rows}, indent=2)+'\n')
print('All %d workload groups preserve their verification records.' % len(rows))
