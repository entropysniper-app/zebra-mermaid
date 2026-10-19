#!/usr/bin/env python3
"""Freeze measured shared style storage and ordered membership results without host data."""
import gzip, hashlib, json, math, os, pathlib, re, statistics
root=pathlib.Path(__file__).resolve().parents[2]
target=root/os.environ.get('CARGO_TARGET_DIR','target')
base=root/'target/profiling/shared-style'
out=root/'docs/research/shared-style-optimization-2026-10-19'
assert not out.exists(), 'Published snapshots must not be overwritten'
read=lambda p:json.loads(p.read_text())
reference=read(base/'reference.json');assert len(reference)==3 and all(x['beforeAfterPeleEqual'] for x in reference)
stress=read(root/'target/test-parity/stress-debug-results.json');assert stress['summary']=={'total':172,'passed':172,'failed':0,'timeout':0}
audit=read(root/'target/test-parity/report.json');assert audit['execution']['isolated'] and audit['summary']['failed']==audit['summary']['unverified']==0
log=(root/'.research/shared-style-full-regression.log').read_text();rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
workspace={key:sum(int(row[i]) for row in rows) for key,i in [('passed',1),('failed',2),('ignored',3)]};assert workspace=={'passed':3015,'failed':0,'ignored':5}
comparison=read(base/'paired/comparison.json');assert len(comparison['rows'])==39 and all(r['consistent'] for r in comparison['rows'])
runs={name:read(base/'paired'/(name+'.json'))for name in ['before-1','after-1','after-2','before-2','before-3','after-3']}
for run in runs.values():
 assert len(run['cases'])==39
 for a,b in zip(runs['before-1']['cases'],run['cases']):
  for key in ('name','diagrams','verification','renderInputs','rejectInputs'):assert a[key]==b[key]
families=[r for r in comparison['rows'] if r['case'].startswith('corpus:') and r['case']!='corpus: agentflow-reference'];assert len(families)==33
ratio=math.exp(statistics.mean(math.log(r['render']['afterOverBefore']) for r in families))
files={'reference.json':reference,'benchmark-comparison.json':comparison,'stress-native.json':{**{k:v for k,v in stress.items() if k!='results'},'results':[{k:v for k,v in r.items() if k!='output'}for r in stress['results']]}}
for phase in ('before','after'):
 for name in ['metadata','allocations','cpu','cpu-workload','agentflow-allocations','agentflow-cpu']:
  files[f'{phase}-{name}.json']=read(base/phase/(name+'.json'))
files['before-model-sources.json']=read(base/'before/model-sources.json')
allocations=[]
before=files['before-allocations.json']+[files['before-agentflow-allocations.json']]
after=files['after-allocations.json']+[files['after-agentflow-allocations.json']]
for a,b in zip(before,after):
 assert a['case']==b['case'] and a['retainedBytesAfterBatch']==b['retainedBytesAfterBatch']==0
 for key in ('renderedPerBatch','rejectedPerBatch'):assert a[key]==b[key]
 allocations.append({'case':a['case'],'before':a,'after':b,'reductionPercent':{key:100*(1-b[key]/a[key])for key in ['allocations','requestedBytes','peakAdditionalLiveBytes']}})
files['summary.json']={'baseline':'Pele-style Class render cache already enabled; baseline binaries are frozen. Class baseline allocation/CPU data are reused from the preceding verified snapshot; Agentflow baseline was separately sampled before this change.','strictTestSummary':audit['summary'],'workspace':workspace,'renderRatioVsCacheBaseline':ratio,'renderChangePercent':100*(ratio-1),'allocations':allocations}
files['metadata.json']={'inputSha256':hashlib.sha256((root/'target/bench/cases.json').read_bytes()).hexdigest(),'binarySha256':{'beforeBenchmark':hashlib.sha256((base/'before/bench').read_bytes()).hexdigest(),'afterBenchmark':hashlib.sha256((target/'release/examples/bench').read_bytes()).hexdigest()},'sourceFiles':[{'path':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in sorted((root/'crates/zebra-mermaid/src').rglob('*.rs'))]}
for obj in [*files.values(),*runs.values()]:assert not re.search(r'/Users/|/home/|localhost|127\.0\.0\.1',json.dumps(obj))
out.mkdir()
for name,value in files.items():(out/name).write_text(json.dumps(value,indent=2)+'\n')
for name,value in runs.items():(out/(name+'.json.gz')).write_bytes(gzip.compress(json.dumps(value).encode(),mtime=0))
print(json.dumps({'workspace':workspace,'strict':audit['summary'],'renderChangePercent':100*(ratio-1),'allocations':[{'case':x['case'],**x['reductionPercent']}for x in allocations]},indent=2))
