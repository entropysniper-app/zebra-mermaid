#!/usr/bin/env python3
"""Measure each engine in a fresh child: CPU time and OS peak RSS, separate from timed benchmarks."""
import json, os, pathlib, resource, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

if len(sys.argv) > 1 and sys.argv[1] == '--child':
    completed = subprocess.run(sys.argv[2:], cwd=ROOT, capture_output=True, text=True, check=True, env={**os.environ, 'TZ': 'UTC'})
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    result = json.loads(completed.stdout)
    result.update(cpuUserSeconds=usage.ru_utime, cpuSystemSeconds=usage.ru_stime,
                  peakRssBytes=int(usage.ru_maxrss * (1 if sys.platform == 'darwin' else 1024)))
    print(json.dumps(result))
else:
    target = ROOT / os.environ.get('CARGO_TARGET_DIR', 'target')
    output = ROOT / 'target/profiling'
    output.mkdir(parents=True, exist_ok=True)
    selected = sys.argv[1:]
    cases = json.loads((ROOT / 'target/bench/cases.json').read_text())
    reports = []
    for case in cases:
        name = case['name']
        if selected and name not in selected:
            continue
        row = {'case': name, 'runs': []}
        for trial in range(3):
            run = {}
            for engine in (['rust', 'pele'] if trial % 2 else ['pele', 'rust']):
                command = [str(target / 'release/examples/profile')] if engine == 'rust' else ['node', 'tools/bench/profile-pele.mjs']
                run[engine] = json.loads(subprocess.check_output([sys.executable, __file__, '--child', *command, name, '1'], cwd=ROOT, text=True))
            for key in ('renderedPerBatch','rejectedPerBatch','svgBytesPerBatch'):
                assert run['rust'][key] == run['pele'][key], (name, key)
            row['runs'].append(run)
        row['rustAllocation'] = json.loads(subprocess.check_output([str(target / 'release/zebra-mermaid-profiler'), name], cwd=ROOT, text=True))
        reports.append(row)
        print(name, flush=True)
    (output / 'resources.json').write_text(json.dumps({'method': 'Three fresh processes per engine, five warm-up batches, >=1s workload loop; OS peak RSS and process CPU include startup, file loading and warm-up. No machine identity recorded. Allocation instrumentation is a separate executable.', 'cases': reports}, indent=2)+'\n')
