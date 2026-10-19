#!/usr/bin/env python3
"""Freeze binaries and profile the two unchanged Class stress workloads, before or after a fix."""
import collections, hashlib, json, os, pathlib, re, shutil, subprocess, sys
root=pathlib.Path(__file__).resolve().parents[2]
phase=sys.argv[1]
assert phase in ('before','after')
out=root/os.environ.get('ZM_STYLE_PROFILE_DIR','target/profiling/class-style')/phase
out.mkdir(parents=True,exist_ok=False)
target=root/os.environ.get('CARGO_TARGET_DIR','target')
cases_path=root/'target/profiling/stress/cases.json'
cases=[c for c in json.loads(cases_path.read_text()) if c['name'].startswith(('stress: class/50 ', 'stress: class/51 '))]
assert len(cases)==2
metadata={'phase':phase,'inputSha256':hashlib.sha256(cases_path.read_bytes()).hexdigest(),'renderSourceSha256':hashlib.sha256((root/'crates/zebra-mermaid/src/diagrams/class/render.rs').read_bytes()).hexdigest(),'binarySha256':{}}
for name,path in [('profile',target/'release/examples/profile'),('allocator',target/'release/zebra-mermaid-profiler'),('cli',target/'release/zebra-mermaid')]:
 shutil.copy2(path,out/name);metadata['binarySha256'][name]=hashlib.sha256(path.read_bytes()).hexdigest()
metadata['modelSources']=[{'path':path,'sha256':hashlib.sha256((root/path).read_bytes()).hexdigest()} for path in ['crates/zebra-mermaid/src/diagrams/class/mod.rs','crates/zebra-mermaid/src/diagrams/class/parser.rs','crates/zebra-mermaid/src/diagrams/agentflow/db.rs','crates/zebra-mermaid/Cargo.toml']]
(out/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
env={**os.environ,'TZ':'UTC','ZM_PROFILE_CASES':str(cases_path)}
allocations=[];svg=[]
for c in cases:
 value=json.loads(subprocess.check_output([str(out/'allocator'),c['name']],cwd=root,env=env,text=True))
 allocations.append(value)
 raw=subprocess.check_output([str(out/'cli'),'render','-','--color-mode','css','--limit',str(2**63-1)],input=c['inputs'][0],cwd=root,env=env,text=True)
 svg.append({'case':c['name'],'bytes':len(raw.encode()),'sha256':hashlib.sha256(raw.encode()).hexdigest()})
 (out/('class-'+c['name'].split('/')[1].split(' ')[0]+'.svg')).write_text(raw)
 print(c['name'],flush=True)
(out/'allocations.json').write_text(json.dumps(allocations,indent=2)+'\n')
(out/'svg.json').write_text(json.dumps(svg,indent=2)+'\n')
raw=out/'cpu-raw.txt'
with (out/'cpu-workload.json').open('w') as stream:
 child=subprocess.Popen([str(out/'profile'),cases[0]['name'],'7'],cwd=root,env=env,stdout=stream)
 try:subprocess.run(['sample',str(child.pid),'5','1','-file',str(raw)],check=True,capture_output=True,text=True)
 finally:
  if child.wait()!=0:raise RuntimeError('CPU workload failed')
text=raw.read_text();graph=text.split('Total number in stack')[0];samples=re.search(r'^\s+(\d+) Thread_',graph,re.M);counts=collections.Counter()
for line in graph.splitlines():
 match=re.match(r'[ +!:|]*(\d+) (zebra_mermaid::[^ ]+)',line)
 if match:counts[re.sub(r'::h[0-9a-f]+$','',match[2])]+=int(match[1])
(out/'cpu.json').write_text(json.dumps({'case':cases[0]['name'],'threadSamples':int(samples[1]),'inclusiveFrames':[{'symbol':s,'count':n}for s,n in counts.most_common(40)],'rawSha256':hashlib.sha256(raw.read_bytes()).hexdigest()},indent=2)+'\n')
print('Saved '+phase+' Class style profiling',flush=True)
