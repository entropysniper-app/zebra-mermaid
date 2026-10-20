#!/usr/bin/env node
// Run isolated processes serially, alternating engine order across three independent runs.
import {readFileSync,writeFileSync,mkdirSync,statSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../..',import.meta.url));
const referenceCommit=execFileSync('git',['-C',resolve(root,'.research/pele'),'rev-parse','HEAD'],{encoding:'utf8'}).trim();
if(referenceCommit!=='8603bf3313add9143cd417015d4a6bd6c522dada')throw Error('Unexpected Pele revision');
const dir=resolve(root,'target/bench');mkdirSync(dir,{recursive:true});
const target=resolve(root,process.env.CARGO_TARGET_DIR??'target');
const rust=resolve(target,'release/examples/bench'),cli=resolve(target,'release/zebra-mermaid');
const input=readFileSync(resolve(dir,'cases.json'));
const execute=(command,args,options={})=>{const r=spawnSync(command,args,{cwd:root,encoding:'utf8',maxBuffer:32*1024*1024,env:{...process.env,TZ:'UTC'},...options});if(r.error)throw r.error;if(r.status!==0)throw Error(`${command} failed (${r.status}): ${r.stderr}`);return r;};
const runs=[];
for(let run=0;run<3;run++){
 const files={};
 for(const engine of run%2?['rust','pele']:['pele','rust']){
  const file=resolve(dir,`${engine}-${run+1}.json`);console.log(`run ${run+1}/3: ${engine}`);
  execute(engine==='rust'?rust:process.execPath,engine==='rust'?[file]:['tools/bench/pele.mjs',file]);files[engine]=JSON.parse(readFileSync(file,'utf8'));
 }
 runs.push(files);
}
const med=a=>{const s=[...a].sort((a,b)=>a-b);return s[s.length>>1];};
const rows=[];
for(let i=0;i<runs[0].pele.cases.length;i++){
 const ps=runs.map(r=>r.pele.cases[i]),rs=runs.map(r=>r.rust.cases[i]);
 const first=ps[0];
 if([...ps,...rs].some(c=>c.name!==first.name||c.diagrams!==first.diagrams))throw Error('case order/count changed');
 const same=(a,b)=>b&&a.ok===b.ok&&(a.ok?a.svgHash===b.svgHash&&a.width===b.width&&a.height===b.height:a.code===b.code&&a.message===b.message&&a.line===b.line&&a.column===b.column);
 const consistent=ps.every((p,k)=>p.verification.every((v,j)=>same(v,rs[k].verification[j])));
 const eligible=consistent&&ps.every((p,k)=>p.renderInputs===rs[k].renderInputs&&p.rejectInputs===rs[k].rejectInputs&&p.renderInputs>0);
 const stat=key=>ps.every(c=>c[key])&&rs.every(c=>c[key])?({peleMs:med(ps.map(c=>c[key].ms)),rustMs:med(rs.map(c=>c[key].ms)),ratios:ps.map((p,k)=>rs[k][key].ms/p[key].ms)}):null;
 const mismatches=[];for(let k=0;k<ps.length;k++)for(let j=0;j<ps[k].verification.length;j++)if(!same(ps[k].verification[j],rs[k].verification[j]))mismatches.push({run:k+1,input:j,pele:ps[k].verification[j],rust:rs[k].verification[j]});
 const row={name:first.name,diagrams:first.diagrams,consistent,eligible,mismatches,renderInputs:first.renderInputs,rejectInputs:first.rejectInputs,peleDone:ps[0].render?.done??0,rustDone:rs[0].render?.done??0,render:stat('render'),parse:stat('parse')};
 if(eligible){row.render.rustOverPele=row.render.rustMs/row.render.peleMs;row.parse.rustOverPele=row.parse.rustMs/row.parse.peleMs;}
 if(first.rejectInputs>0){row.rejection=stat('rejection');if(consistent)row.rejection.rustOverPele=row.rejection.rustMs/row.rejection.peleMs;}
 rows.push(row);
}
const source='flowchart LR\n  A[Start] --> B{Ready?}\n  B -->|yes| C[Done]\n';
const cold={rust:[],nodeReference:[]};let expected;
for(let i=0;i<31;i++)for(const key of i%2?['rust','nodeReference']:['nodeReference','rust']){
 const start=process.hrtime.bigint();const out=key==='rust'?execute(cli,['render','-','--color-mode','css'],{input:source}).stdout:execute(process.execPath,['tools/bench/pele-cli.mjs'],{input:source}).stdout;
 const ms=Number(process.hrtime.bigint()-start)/1e6;const normalized=out.trimEnd();if(expected===undefined)expected=normalized;else if(normalized!==expected)throw Error('CLI/reference SVG differs');if(i>0)cold[key].push(ms);
}
const coldSummary=Object.fromEntries(Object.entries(cold).map(([k,v])=>[k,{medianMs:med(v),minMs:Math.min(...v),maxMs:Math.max(...v),samples:v}]));
const runtime={node:process.version,rustc:execFileSync('rustc',['--version'],{encoding:'utf8'}).trim()};
const report={createdAt:new Date().toISOString(),commit:'8603bf3313add9143cd417015d4a6bd6c522dada',enabledTypes:runs[0].rust.enabledTypes,inputSha256:createHash('sha256').update(input).digest('hex'),runtime,method:{runs:3,minimumSamples:Number(process.env.BENCH_SAMPLES??9),minimumSamplingMs:Number(process.env.BENCH_MS??300),calibrationMs:10,mode:'css',font:'default Arial metrics',now:1792368000000,timezone:'UTC',correctness:'Every success must match UTF-8 SVG FNV-1a-64 and dimensions; every rejection must match code, message, line and column. Successful render and expected rejection workloads are timed separately.'},rows,cold:coldSummary,size:{rustCliBytes:statSync(cli).size,peleBundleBytes:statSync(resolve(root,'.research/pele/dist/pele.js')).size}};
writeFileSync(resolve(dir,'comparison.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({eligible:rows.filter(r=>r.eligible).length,total:rows.length,cold:coldSummary},null,2));
if(rows.some(row=>!row.consistent))process.exitCode=1;
