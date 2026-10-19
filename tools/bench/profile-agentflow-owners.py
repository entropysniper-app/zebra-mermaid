#!/usr/bin/env python3
"""Compare the frozen pre-change driver with owner-aware global membership cleanup."""
import collections, hashlib, json, os, pathlib, re, shutil, statistics, subprocess

root = pathlib.Path(__file__).resolve().parents[2]
out = root / 'target/profiling/agentflow-owners'
target = root / os.environ.get('CARGO_TARGET_DIR', 'target')
assert not (out/'timing.json').exists(), 'Do not overwrite measured results'
after = out / 'after'; after.mkdir(exist_ok=True)
metadata = {}
for name, path in [('profile', target/'release/examples/profile'), ('bench', target/'release/examples/bench'),
                   ('cli', target/'release/zebra-mermaid'), ('allocator', target/'release/zebra-mermaid-profiler')]:
    metadata[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    if (after/name).exists():
        assert hashlib.sha256((after/name).read_bytes()).hexdigest() == metadata[name]
    else:
        shutil.copy2(path, after/name)
(after/'metadata.json').write_text(json.dumps(metadata, indent=2)+'\n')
case_path = root/'target/profiling/stress/cases.json'
case = next(c for c in json.loads(case_path.read_text()) if c['name'].startswith('stress: agentflow/44 '))
name = case['name']; env = {**os.environ, 'TZ':'UTC', 'ZM_PROFILE_CASES':str(case_path)}

outputs = []
for phase in ('before','after'):
    svg = subprocess.check_output([str(out/phase/'cli'), 'render','-', '--color-mode','css', '--limit',str(2**63-1)],
                                  input=case['inputs'][0].encode(), cwd=root, env=env)
    outputs.append(svg)
    allocations = json.loads(subprocess.check_output([str(out/phase/'allocator'),name],cwd=root,env=env))
    (out/phase/'allocations.json').write_text(json.dumps(allocations,indent=2)+'\n')
pele = subprocess.check_output(['node','--input-type=module','-e',
    "import{readFileSync}from'node:fs';import{render}from'./.research/pele/dist/pele.js';process.stdout.write(render(readFileSync(0,'utf8'),{limit:Infinity,now:1792368000000}).svg)"],
    input=case['inputs'][0].encode(),cwd=root,env=env)
assert outputs[0] == outputs[1] == pele
(out/'reference.json').write_text(json.dumps({'case':name,'beforeAfterPeleEqual':True,'bytes':len(pele),'sha256':hashlib.sha256(pele).hexdigest()},indent=2)+'\n')

runs = []
for trial in range(3):
    row = {}
    for phase in (['before','after'] if trial%2==0 else ['after','before']):
        r = json.loads(subprocess.check_output([str(out/phase/'profile'),name,'3'],cwd=root,env=env))
        r['msPerBatch'] = r['elapsedMs']/r['batches']; row[phase] = r
    assert row['before']['renderedPerBatch']==row['after']['renderedPerBatch']==1
    assert row['before']['rejectedPerBatch']==row['after']['rejectedPerBatch']==0
    runs.append(row)
    print('Agentflow isolated loop trial',trial+1,flush=True)
before = statistics.median(r['before']['msPerBatch'] for r in runs)
current = statistics.median(r['after']['msPerBatch'] for r in runs)
(out/'timing.json').write_text(json.dumps({'method':'Three independent processes per version, alternating order, five warm-ups then a >=3 second full-render loop. Report median of three loop means; this stress-specific metric is separate from the original Pele benchmark table. No sampler or allocation counters in timing.',
    'case':name,'beforeMs':before,'afterMs':current,'afterOverBefore':current/before,'runs':runs},indent=2)+'\n')
for phase in ('before','after'):
    raw = out/phase/'cpu-raw.txt'
    with (out/phase/'cpu-workload.json').open('w') as stream:
        child = subprocess.Popen([str(out/phase/'profile'),name,'7'],cwd=root,env=env,stdout=stream)
        try: subprocess.run(['sample',str(child.pid),'5','1','-file',str(raw)],check=True,capture_output=True)
        finally: assert child.wait()==0
    graph = raw.read_text().split('Total number in stack')[0]
    samples = int(re.search(r'^\s+(\d+) Thread_',graph,re.M)[1]); counts = collections.Counter()
    for line in graph.splitlines():
        match = re.match(r'[ +!:|]*(\d+) (zebra_mermaid::[^ ]+)',line)
        if match: counts[re.sub(r'::h[0-9a-f]+$','',match[2])] += int(match[1])
    (out/phase/'cpu.json').write_text(json.dumps({'threadSamples':samples,'inclusiveFrames':[{'symbol':s,'count':n}for s,n in counts.most_common(40)],
        'rawSha256':hashlib.sha256(raw.read_bytes()).hexdigest()},indent=2)+'\n')
print(json.dumps({'beforeMs':before,'afterMs':current,'speedup':before/current}),flush=True)
