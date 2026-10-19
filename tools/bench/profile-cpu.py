#!/usr/bin/env python3
"""Collect CPU stacks with the platform sampler; publish only aggregate symbols, never host metadata."""
import collections, hashlib, json, os, pathlib, re, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
output = ROOT / os.environ.get('ZM_CPU_OUTPUT', 'target/profiling/cpu')
output.mkdir(parents=True, exist_ok=True)
target = ROOT / os.environ.get('ZM_CPU_BINARY', str(pathlib.Path(os.environ.get('CARGO_TARGET_DIR', 'target')) / 'release/examples/profile'))
workloads = sys.argv[1:] or ['tree: 50 nodes', 'mesh: 500 nodes, 200 extra labelled edges', 'corpus: agentflow-docs']
reports = []
for index, name in enumerate(workloads):
    raw = output / ('case-%d.txt' % index)
    with (output / ('case-%d.json' % index)).open('w') as stream:
        child = subprocess.Popen([str(target), name, '7'], cwd=ROOT, stdout=stream)
        try:
            if sys.platform == 'darwin':
                subprocess.run(['sample', str(child.pid), '5', '1', '-file', str(raw)], check=True, capture_output=True, text=True)
            else:
                raise SystemExit('CPU stack collection currently requires the sample command; resource/CPU-time measurement is separate.')
        finally:
            if child.wait() != 0:
                raise RuntimeError('Profiling workload failed')
    text = raw.read_text()
    graph = text.split('Total number in stack')[0]
    samples = re.search(r'^\s+(\d+) Thread_', graph, re.M)
    counts = collections.Counter()
    for line in graph.splitlines():
        match = re.match(r'[ +!:|]*(\d+) (zebra_mermaid::[^ ]+)', line)
        if match:
            symbol = re.sub(r'::h[0-9a-f]+$', '', match[2])
            counts[symbol] += int(match[1])
    reports.append({'case': name, 'threadSamples': int(samples[1]),
                    'inclusiveFrames': [{'symbol': s, 'count': n} for s, n in counts.most_common(30)],
                    'rawSha256': hashlib.sha256(raw.read_bytes()).hexdigest()})
    print(name, flush=True)
(output / 'summary.json').write_text(json.dumps({'method': 'External sample: 5 seconds at 1 ms interval; 7 second render loop. Inclusive frame counts include descendants and recursive occurrences and must not be added as exclusive CPU percentages. Raw captures stay under ignored target/.', 'cases': reports}, indent=2)+'\n')
