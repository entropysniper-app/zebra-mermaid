#!/usr/bin/env python3
"""Paired process CPU and OS peak RSS checks, run separately from latency benchmarks."""
import hashlib
import json
import os
import pathlib
import statistics
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[2]
cases_path = root / os.environ.get('ZM_RESOURCE_CASES', 'target/bench/cases.json')
destination = root / os.environ.get('ZM_RESOURCE_REPORT', 'target/profiling/paired-resources.json')
binaries = {phase: root / os.environ[f'ZM_RESOURCE_{phase.upper()}'] for phase in ['before', 'after']}
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
binary_hashes = {phase: sha(path) for phase, path in binaries.items()}
rows = []

for case in json.loads(cases_path.read_text()):
    row = {'case': case['name'], 'runs': []}
    expected = None
    for trial in range(3):
        run = {}
        for phase in (['before', 'after'] if trial % 2 == 0 else ['after', 'before']):
            command = [sys.executable, 'tools/bench/profile-resources.py', '--child',
                       str(binaries[phase]), case['name'], '1']
            value = json.loads(subprocess.check_output(
                command, cwd=root, text=True,
                env={**os.environ, 'TZ': 'UTC', 'ZM_PROFILE_CASES': str(cases_path)},
            ))
            digest = [value[key] for key in ['renderedPerBatch', 'rejectedPerBatch', 'svgBytesPerBatch']]
            if expected is None:
                expected = digest
            assert digest == expected, (case['name'], phase, digest, expected)
            assert value['elapsedMs'] >= 1000 and value['batches'] > 0, (case['name'], value)
            run[phase] = value
        row['runs'].append(run)
    row['peakRssBytes'] = {
        phase: statistics.median(run[phase]['peakRssBytes'] for run in row['runs'])
        for phase in binaries
    }
    rows.append(row)
    print(case['name'], flush=True)

assert all(sha(path) == binary_hashes[phase] for phase, path in binaries.items())
report = {
    'method': 'Three fresh processes per version and case, alternating order; five warm-up batches and >=1 second complete-render loop. OS peak RSS and process CPU include startup, input-file loading and warm-up. CPU seconds are process totals, not the latency benchmark. No overlap with timing, tests, CPU sampling or allocation probes.',
    'inputSha256': sha(cases_path),
    'binaryHashes': binary_hashes,
    'sourceHashes': {
        path: sha(root / path) for path in ['tools/bench/compare-resources.py', 'tools/bench/profile-resources.py']
    },
    'interpretation': 'Peak RSS is an OS process measurement, separate from requested allocator bytes and peak additional live allocations. Each case runs in a new process; RSS medians are not summed across cases. All runs and regressions are retained; small RSS differences can reflect page and allocator variability.',
    'rows': rows,
}
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_text(json.dumps(report, indent=2) + '\n')
print(f'{len(rows)} paired resource groups verified.', flush=True)
