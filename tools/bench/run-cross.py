#!/usr/bin/env python3
"""Verify all four engines, then measure exactly their common successful Sample inputs."""
import datetime, hashlib, json, math, os, pathlib, statistics, subprocess, xml.etree.ElementTree as ET

root=pathlib.Path(__file__).resolve().parents[2]
out=root/'target/cross-bench'; out.mkdir(exist_ok=True)
revision='3726ccbffe0e8032361eb9668694b24f77858060'
actual=subprocess.check_output(['git','-C','.research/mermaid-rs-renderer','rev-parse','HEAD'],cwd=root,text=True).strip()
assert actual==revision, 'Unexpected mermaid-rs-renderer revision'
pele_revision=subprocess.check_output(['git','-C','.research/pele','rev-parse','HEAD'],cwd=root,text=True).strip()
assert pele_revision=='e7f092a4de147cc32210913c20e36eaa5fcd0ef1', 'Unexpected Pele revision'
native=root/os.environ.get('ZM_CROSS_BINARY','target/cross-bench-build/release/zebra-mermaid-cross-bench')
env={**os.environ,'TZ':'UTC','XDG_CACHE_HOME':str(out/'font-cache')}
engines=['mermaid','pele','zebra','mmdr']
data=json.loads((root/'docs/render-comparison/results.json').read_text())
cases=[]
for r in data['results']:
    if not cases or cases[-1]['type']!=r['type']:cases.append({'name':'samples: '+r['type'],'type':r['type'],'inputs':[]})
    cases[-1]['inputs'].append({'id':r['id'],'source':r['source']})
assert len(cases)==33 and sum(len(c['inputs']) for c in cases)==167
case_path=out/'cases.json';case_path.write_text(json.dumps(cases,ensure_ascii=False)+'\n')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def execute(engine, mode, inputs, output, log_name):
    command=[str(native),engine,mode,str(inputs),str(output)] if engine in ('zebra','mmdr') else [
        'node','tools/bench/cross-'+engine+'.mjs',mode,str(inputs),str(output)]
    with (out/(log_name+'.log')).open('w') as log:
        r=subprocess.run(command,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=1200)
    assert r.returncode==0,(engine,mode,'see ignored log',log_name)
    print(engine,mode,log_name,flush=True)
    return json.loads(output.read_text())

verification={}
expected_mmdr={'flowchart':'Flowchart','class':'Class','state':'State','sequence':'Sequence','er':'Er','pie':'Pie',
    'mindmap':'Mindmap','journey':'Journey','timeline':'Timeline','gantt':'Gantt','requirement':'Requirement','git':'GitGraph',
    'c4':'C4','sankey':'Sankey','quadrant':'Quadrant','block':'Block','packet':'Packet','kanban':'Kanban',
    'architecture':'Architecture','radar':'Radar','treemap':'Treemap','xychart':'XYChart'}
source_type={i['id']:c['type']for c in cases for i in c['inputs']}
svgdir=out/'svg';svgdir.mkdir(exist_ok=True)
for engine in engines:
    report=execute(engine,'verify',case_path,out/(engine+'-verify.json'),engine+'-verify')
    results={}
    for c in report['cases']:
        for row in c['results']:
            if row['ok']:
                row['returnedSvg']=True
                svg=row.pop('svg')
                try:
                    element=ET.fromstring(svg)
                    assert element.tag.rsplit('}',1)[-1]=='svg'
                    box=[float(n)for n in element.attrib.get('viewBox','').replace(',',' ').split()]
                    if not box and engine=='mermaid':
                        box=[0,0,row['viewportWidth'],row['viewportHeight']]
                        row['dimensionsFrom']='browser viewport (SVG has no viewBox)'
                    assert len(box)==4 and all(math.isfinite(n)for n in box) and box[2]>0 and box[3]>0
                    path=svgdir/(row['id']+'.'+engine+'.svg');path.write_text(svg)
                    row.update({'svg':path.relative_to(out).as_posix(),'bytes':len(svg.encode()),'sha256':sha(path),'width':box[2],'height':box[3]})
                    if engine=='mmdr' and row.get('actualType')!=expected_mmdr.get(source_type[row['id']]):
                        row.update({'ok':False,'error':'Diagram type mismatch: '+source_type[row['id']]+' parsed as '+row['actualType']})
                except (ET.ParseError,ValueError,AssertionError) as error:
                    row.update({'ok':False,'error':'Returned SVG failed XML/viewBox validation: '+str(error)})
            results[row['id']]=row
    verification[engine]=results
(out/'verification.json').write_text(json.dumps(verification,ensure_ascii=False,indent=2)+'\n')
paired=[]
for id, value in verification['pele'].items():
    other=verification['zebra'][id]
    equal=value['ok']==other['ok'] and (not value['ok'] or all(value[k]==other[k] for k in ['sha256','width','height']))
    assert equal, ('Pele/zebra output mismatch',id)
    paired.append({'id':id,'equal':equal})
(out/'pele-zebra-equality.json').write_text(json.dumps(paired,indent=2)+'\n')

common=[];coverage=[]
for c in cases:
    shared=[i for i in c['inputs']if all(verification[e][i['id']]['ok']for e in engines)]
    coverage.append({'name':c['name'],'type':c['type'],'total':len(c['inputs']),'common':len(shared),
        'success':{e:sum(verification[e][i['id']]['ok']for i in c['inputs'])for e in engines},'commonIds':[i['id']for i in shared]})
    if shared:common.append({**c,'inputs':shared})
common_path=out/'common.json';common_path.write_text(json.dumps(common,ensure_ascii=False)+'\n')
(out/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n')
if os.environ.get('ZM_CROSS_VERIFY_ONLY')=='1':
    print('Verification only; common inputs:',sum(c['common']for c in coverage),flush=True)
    raise SystemExit(0)

runs=[]
orders=[['mermaid','pele','zebra','mmdr'],['mmdr','zebra','pele','mermaid'],['pele','mmdr','mermaid','zebra']]
for trial,order in enumerate(orders,1):
    run={}
    for engine in order:
        run[engine]=execute(engine,'bench',common_path,out/(engine+'-'+str(trial)+'.json'),engine+'-'+str(trial))
    runs.append(run)
rows=[]
for index,c in enumerate(common):
    row={'name':c['name'],'type':c['type'],'inputs':len(c['inputs']),'timings':{}}
    for e in engines:
        timings=[r[e]['cases'][index]for r in runs]
        assert all(t['name']==c['name']and t['timing']['inputs']==len(c['inputs'])for t in timings)
        values=[t['timing']['ms']for t in timings]
        row['timings'][e]={'ms':statistics.median(values),'runsMs':values}
    rows.append(row)
versions={'mermaid':json.loads((root/'.research/pele/website/node_modules/mermaid/package.json').read_text())['version'],
    'pele':'1.9.0','zebra':'0.1.0','mmdr':'0.3.1','mmdrCommit':revision,
    'peleCommit':'e7f092a4de147cc32210913c20e36eaa5fcd0ef1','browser':runs[0]['mermaid']['browser'],
    'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'node':subprocess.check_output(['node','--version'],text=True).strip()}
report={'createdAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'versions':versions,'inputSha256':sha(case_path),'commonSha256':sha(common_path),'nativeBinarySha256':sha(native),
    'nativeCargoLockSha256':sha(root/'tools/cross-bench/Cargo.lock'),'coverage':coverage,'rows':rows,
    'method':{'warmups':5,'calibrationMs':10,'minimumSamples':9,'minimumSamplingMs':300,'processRuns':3,'orders':orders,
        'unit':'ms per full batch of identical common successful Sample inputs','timed':'Full parse, layout and SVG generation; no output cache. Mermaid DOM cleanup and native owned-options clone are included. Browser/driver launch, imports, verification, IPC, disk output and warm-up excluded.',
        'settings':'Arial 16 requested; Pele/zebra CSS output uses Pele variables; Mermaid.js/mmdr dark theme. Source directives retained. Pele/zebra use Arial table metrics; Mermaid browser DOM metrics; mmdr default native font metrics, fastText=false.',
        'correctness':'All 167 original inputs attempted by every engine. Eligibility requires valid SVG XML, positive finite dimensions (viewBox or Mermaid browser viewport), and mmdr parsed DiagramKind matching the input chart family. A returned SVG of a wrongly inferred Flowchart is recorded but excluded. Mermaid supported HTML output normalized through DOM only outside timing. This does not prove full cross-engine semantic or pixel equivalence. Pele/zebra SVG equality checked separately.',
        'clock':'Pele/zebra and browser Date fixed to 1792368000000; UTC. mmdr uses its native handling of authored dates.'}}
(out/'comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'groups':len(rows),'commonInputs':sum(r['inputs']for r in rows),'successes':{e:sum(v['ok']for v in verification[e].values())for e in engines}}),flush=True)
