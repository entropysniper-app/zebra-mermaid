#!/usr/bin/env python3
"""Recheck noisy/regressing cases with closer before/after pairing, without changing the driver."""
import json, os, pathlib, statistics, subprocess
root=pathlib.Path(__file__).resolve().parents[2]
base=root/'target/profiling/multi-diagram'
out=base/'focused';out.mkdir(exist_ok=False)
cases=json.loads((root/'target/bench/cases.json').read_text())
initial=json.loads((base/'paired/comparison.json').read_text())
selected={r['case']for r in initial['rows']if r.get('render',{}).get('afterOverBefore',0)>=1.03}
selected.update(['docs: all flowchart examples','corpus: er-docs','corpus: sequence-docs'])
work=out/'work';(work/'target/bench').mkdir(parents=True)
rows=[]
for index,c in enumerate(c for c in cases if c['name']in selected):
    (work/'target/bench/cases.json').write_text(json.dumps([c]))
    runs=[];expected=None
    for trial in range(5):
        results={}
        for phase in (['before','after']if trial%2==0 else ['after','before']):
            file=out/f'case-{index}-{phase}-{trial+1}.json'
            subprocess.run([str(base/phase/'bench'),str(file)],cwd=work,env={**os.environ,'TZ':'UTC','BENCH_MS':'300','BENCH_SAMPLES':'9'},check=True)
            result=json.loads(file.read_text())['cases'][0]
            observed={k:result[k]for k in ['name','diagrams','verification','renderInputs','rejectInputs']}
            if expected is None:expected=observed
            assert observed==expected
            results[phase]=result
        runs.append(results)
    row={'case':c['name'],'diagrams':len(c['inputs']),'consistent':True}
    for op in ['render','parse','rejection']:
        if not runs[0]['before'].get(op):continue
        b=[r['before'][op]['ms']for r in runs];a=[r['after'][op]['ms']for r in runs]
        row[op]={'beforeMs':statistics.median(b),'afterMs':statistics.median(a),'afterOverBefore':statistics.median(a)/statistics.median(b),'beforeRuns':b,'afterRuns':a,'pairedRatios':[x/y for x,y in zip(a,b)]}
    rows.append(row)
    print(c['name'],row['render']['afterOverBefore'],flush=True)
(out/'comparison.json').write_text(json.dumps({'method':'Selected all initial render regressions >=3%, plus flowchart/ER/sequence controls. Unmodified frozen bench executable; one original workload per process, original input order/options/5 warm-ups/10ms calibration/>=9 samples and >=300ms preserved. Five before/after process pairs per case, alternating order. Temporary cwd supplies the same selected case at the original relative input path; canonical corpus unchanged. Both initial and focused records retained, not merged into a new global geometric mean.','rows':rows},indent=2)+'\n')
