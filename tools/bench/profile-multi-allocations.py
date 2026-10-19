#!/usr/bin/env python3
"""Record allocations for every existing workload, separately from CPU timing."""
import json, os, pathlib, subprocess
root=pathlib.Path(__file__).resolve().parents[2]
out=root/'target/profiling/multi-diagram'
cases=json.loads((root/'target/bench/cases.json').read_text())
rows=[]
for c in cases:
    row={'case':c['name']}
    for phase in ['before','after']:
        row[phase]=json.loads(subprocess.check_output([str(out/phase/'allocator'),c['name']],cwd=root,
            env={**os.environ,'TZ':'UTC'},text=True))
        assert row[phase]['retainedBytesAfterBatch']==0
    for key in ['renderedPerBatch','rejectedPerBatch']:
        assert row['before'][key]==row['after'][key]
    rows.append(row)
    print(c['name'],flush=True)
(out/'allocations.json').write_text(json.dumps({'method':'Independent counting allocator, five warm-ups then one full batch. Inputs and options excluded; results dropped. Requested bytes include full new realloc sizes, not RSS. These processes are not timed as CPU benchmarks.','rows':rows},indent=2)+'\n')
