#!/usr/bin/env python3
"""Publish portable CPU, ownership, RSS and leak-check evidence after regression succeeds."""
import gzip, hashlib, json, math, pathlib, re, statistics

root = pathlib.Path(__file__).resolve().parents[2]
base = root/'target/profiling/agentflow-owners'
out = root/'docs/research/agentflow-memory-2026-10-19'
assert not out.exists(), 'Published evidence must not be overwritten'
read = lambda p: json.loads(p.read_text())
comparison = read(base/'paired/comparison.json')
assert len(comparison['rows']) == 39 and all(r['consistent'] for r in comparison['rows'])
families = [r for r in comparison['rows'] if r['case'].startswith('corpus:') and r['case'] != 'corpus: agentflow-reference']
assert len(families) == 33
ratio = math.exp(statistics.mean(math.log(r['render']['afterOverBefore']) for r in families))
memory = read(root/'target/profiling/memory/summary.json')
soak = read(root/'target/profiling/memory-soak/summary.json')
assert memory['binarySha256'] == soak['binarySha256']
for run in memory['runs'] + soak['runs']:
    r = run['result']
    assert r['retainedBytesPerRound'] == r['retainedBlocksPerRound'] == r['retainedBytesAfterRepeatedCacheDrop'] == 0
assert len(memory['systemLeaks']) == 5
for r in memory['systemLeaks']:
    assert (r['leaks'] >= 1 and r['leakedBytes'] >= 4096) if r['mode']=='negative-control' else (r['leaks']==r['leakedBytes']==0)
audit = read(root/'target/test-parity/report.json')
assert (root/'target/test-parity/report.json').stat().st_mtime > (root/'crates/zebra-mermaid/src/diagrams/agentflow/db.rs').stat().st_mtime
assert audit['execution']['isolated'] and audit['summary']['failed']==audit['summary']['unverified']==0
stress = read(root/'target/test-parity/stress-debug-results.json')
assert stress['summary'] == {'total':172,'passed':172,'failed':0,'timeout':0}
log = (root/'.research/owners-full-regression.log').read_text()
rows = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
workspace = {k:sum(int(r[i])for r in rows)for k,i in [('passed',1),('failed',2),('ignored',3)]}
assert workspace == {'passed':3016,'failed':0,'ignored':5}, workspace
reference = read(base/'reference.json'); assert reference['beforeAfterPeleEqual']
files = {name:read(base/name) for name in ['reference.json','model-reference.json','timing.json']}
for phase in ('before','after'):
    for name in ['metadata','allocations','cpu','cpu-workload']:
        files[f'{phase}-{name}.json'] = read(base/phase/(name+'.json'))
files.update({'memory.json':memory,'memory-soak.json':soak,'benchmark-comparison.json':comparison,
              'stress-native.json':{**{k:v for k,v in stress.items() if k!='results'},
                'results':[{k:v for k,v in r.items() if k!='output'}for r in stress['results']]}})
files['summary.json'] = {'baseline':'Previous shared Class style / ordered membership implementation; binaries frozen before owner indexing.',
    'workspace':workspace,'originalTests':audit['summary'],'stress':stress['summary'],
    'renderRatioVsPreviousRust':ratio,'renderChangePercent':100*(ratio-1),'targetTiming':files['timing.json'],
    'memorySourceCycles':sum(r['result']['rounds']*r['result']['inputsPerRound']for r in memory['runs']+soak['runs']),
    'conclusion':'No retained allocator bytes or blocks after any measured round. Independent system leaks reports zero in four tested paths and detects a separate deliberate leak. RSS includes allocator/page retention and is reported separately; this is evidence for measured scenarios, not a proof for all possible programs.'}
paths = sorted((root/'crates/zebra-mermaid/src').rglob('*.rs')) + sorted((root/'tools/profiler/src').rglob('*.rs'))
paths += [root/p for p in ['Cargo.toml','Cargo.lock','tools/profiler/Cargo.toml','crates/zebra-mermaid/tests/model_serialization.rs']]
files['metadata.json'] = {'productionSourceAndProbeSha256':[{'path':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in paths],
    'inputSha256':hashlib.sha256((root/'target/bench/cases.json').read_bytes()).hexdigest(),
    'baselineAgentflowSourceSha256':hashlib.sha256((base/'before/db.rs').read_bytes()).hexdigest(),
    'cpuMethod':'External sample: 5 seconds at 1 ms interval during a 7 second loop. Inclusive frame counts include descendants, not exclusive CPU percentages. Captures may include warm-up. Instrumented loop timings are not benchmark numbers.'}
runs = {p.name:read(p)for p in sorted((base/'paired').glob('*.json')) if p.name!='comparison.json'}
assert len(runs)==6
for obj in list(files.values())+list(runs.values()):
    assert not re.search(r'/Users/|/home/|localhost|127\.0\.0\.1',json.dumps(obj))
out.mkdir()
for name,obj in files.items(): (out/name).write_text(json.dumps(obj,indent=2)+'\n')
for name,obj in runs.items(): (out/(name+'.gz')).write_bytes(gzip.compress(json.dumps(obj).encode(),mtime=0))
print(json.dumps(files['summary.json'],indent=2))
