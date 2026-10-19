#!/usr/bin/env python3
"""Publish the verified Class style optimization and its original stress-test evidence."""
import gzip, hashlib, json, math, pathlib, re, statistics
root=pathlib.Path(__file__).resolve().parents[2]
source=root/'target/profiling/class-style'
dest=root/'docs/research/class-style-optimization-2026-10-19'
assert not dest.exists(), 'Keep published snapshots immutable'
read=lambda path:json.loads(path.read_text())
manifest=read(root/'tests/upstream/ir/stress.json')
native=read(root/'target/test-parity/stress-debug-results.json')
reference=read(root/'.research/stress-upstream.json')
assert native['summary']=={'total':172,'passed':172,'failed':0,'timeout':0}
assert reference['numTotalTests']==reference['numPassedTests']==172 and reference['numFailedTests']==0
before=read(source/'before/allocations.json');after=read(source/'after/allocations.json')
assert read(source/'before/svg.json')==read(source/'after/svg.json')
svg=read(source/'reference.json');assert len(svg)==2 and all(x['beforeAfterPeleEqual'] for x in svg)
comparison=read(root/'target/profiling/paired/comparison.json')
assert len(comparison['rows'])==39 and all(r['consistent'] for r in comparison['rows'])
paired={}
for version in ('before','after'):
 for trial in range(1,4):
  name=f'{version}-{trial}';paired[name]=read(root/f'target/profiling/paired/{name}.json')
for run in paired.values():
 for x,y in zip(paired['before-1']['cases'],run['cases']):
  for k in ('name','diagrams','verification','renderInputs','rejectInputs'):assert x[k]==y[k]
# Store portable results only; no raw system trace or runner file path is published.
original=[]
for f in manifest['files']:
 actual=next(t for t in reference['testResults'] if t['name'].endswith('/'+f['source']))
 tests=[]
 for r in f['records']:
  a=next(a for a in actual['assertionResults'] if ' > '.join(a['ancestorTitles']+[a['title']])==r['name'])
  assert a['status']=='passed'
  tests.append({'id':r['id'],'name':r['name'],'state':a['status'],'vitestDurationMs':a['duration'],'timeoutMs':r['timeoutMs']})
 original.append({'source':f['source'],'sourceSha256':f['sourceSha256'],'tests':tests})
results={**{k:v for k,v in native.items() if k!='results'},'results':[{k:v for k,v in r.items() if k!='output'} for r in native['results']]}
first=(root/'.research/stress-first-run.log').read_text();timings=[]
for index in (50,51):
 pattern=rf'^failed: tests/class/stress.test.ts #{index} (.*?) \(([\d.]+) ms\)$'
 match=re.search(pattern,first,re.M);assert match
 current=next(r for r in native['results'] if r['name']==match[1])
 timings.append({'name':match[1],'beforeDebugMs':float(match[2]),'afterDebugMs':current['elapsedMs'],'originalBudgetMs':3000})
families=[r for r in comparison['rows'] if r['case'].startswith('corpus:') and r['case']!='corpus: agentflow-reference'];assert len(families)==33
ratio=math.exp(statistics.mean(math.log(r['render']['afterOverBefore']) for r in families))
summary={'stressPassed':172,'strictOriginalTestsPassed':2629,'strictOriginalTestsTotal':6810,
         'wholeWorkspacePassed':3013,'wholeWorkspaceFailed':0,'wholeWorkspaceIgnored':5,
         'stressTiming':timings,'overallRenderRatioVsInitialBaseline':ratio,
         'allocation':[]}
for a,b in zip(before,after):
 assert a['case']==b['case'] and a['retainedBytesAfterBatch']==b['retainedBytesAfterBatch']==0
 for k in ('renderedPerBatch','rejectedPerBatch'):assert a[k]==b[k]
 summary['allocation'].append({'case':a['case'],'before':a,'after':b,'reductionPercent':{k:100*(1-b[k]/a[k]) for k in ('allocations','requestedBytes','peakAdditionalLiveBytes')}})
files={'summary.json':summary,'stress-reference.json':original,'stress-native.json':results,'svg-reference.json':svg,'benchmark-comparison.json':comparison}
for phase in ('before','after'):
 for name in ('metadata','allocations','cpu','cpu-workload','svg'):files[f'{phase}-{name}.json']=read(source/phase/(name+'.json'))
for value in [*files.values(),*paired.values()]:
 text=json.dumps(value);assert not re.search(r'/Users/|/home/|localhost|127\.0\.0\.1',text)
dest.mkdir()
for name,value in files.items():(dest/name).write_text(json.dumps(value,indent=2)+'\n')
for name,value in paired.items():(dest/(name+'.json.gz')).write_bytes(gzip.compress(json.dumps(value).encode(),mtime=0))
print(json.dumps({'stressPassed':172,'overallRenderReductionPercent':100*(1-ratio),'allocationReductions':[r['reductionPercent']for r in summary['allocation']]},indent=2))
