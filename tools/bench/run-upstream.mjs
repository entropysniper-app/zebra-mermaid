#!/usr/bin/env node
// Execute Pele's ORIGINAL benchmark file unmodified; capture only its final console.table.
import {readFileSync,writeFileSync,mkdirSync,statSync} from 'node:fs';
import {gzipSync} from 'node:zlib';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../..',import.meta.url)),upstream=resolve(root,'.research/pele');
const referenceCommit=execFileSync('git',['-C',upstream,'rev-parse','HEAD'],{encoding:'utf8'}).trim();
if(referenceCommit!=='e7f092a4de147cc32210913c20e36eaa5fcd0ef1')throw Error('Unexpected Pele revision');
const dir=resolve(root,'target/bench-upstream');mkdirSync(dir,{recursive:true});
const source=readFileSync(resolve(upstream,'scripts/bench.mjs'),'utf8');
const target=resolve(root,process.env.CARGO_TARGET_DIR??'target');
const execute=(cmd,args,cwd=root)=>{const r=spawnSync(cmd,args,{cwd,encoding:'utf8',maxBuffer:32*1024*1024});if(r.error||r.status!==0)throw Error(`${cmd}: ${r.error??r.stderr}`);return r;};
// Evaluate the upstream declarations verbatim to obtain its exact five workloads.
const prefix=source.slice(0,source.indexOf('function time(')).replace("'../dist/pele.js'",JSON.stringify(pathToFileURL(resolve(upstream,'dist/pele.js')).href));
process.chdir(upstream);
const {cases}=await import('data:text/javascript;base64,'+Buffer.from(prefix+'\nexport { cases };').toString('base64'));
process.chdir(root);
const input=resolve(dir,'cases.json');writeFileSync(input,JSON.stringify(cases.map(([name,inputs])=>({name,inputs}))));
const preload=resolve(dir,'capture.mjs');
writeFileSync(preload,`import{writeFileSync}from'node:fs';\nconst table=console.table;console.table=function(rows,...rest){writeFileSync(${JSON.stringify(resolve(dir,'pele.json'))},JSON.stringify(rows));return table.call(this,rows,...rest)};\n`);
console.log('Running original Pele scripts/bench.mjs');
const reference=execute(process.execPath,['--import',pathToFileURL(preload).href,'scripts/bench.mjs'],upstream);
writeFileSync(resolve(dir,'pele-console.txt'),reference.stdout);process.stdout.write(reference.stdout);
console.log('Running Rust with the same five workloads and sampling protocol');
execute(resolve(target,'release/examples/bench_upstream'),[input,resolve(dir,'rust.json')]);
const p=JSON.parse(readFileSync(resolve(dir,'pele.json'),'utf8')),r=JSON.parse(readFileSync(resolve(dir,'rust.json'),'utf8'));
const {render}=await import(pathToFileURL(resolve(upstream,'dist/pele.js')));
for(let i=0;i<cases.length;i++)for(let j=0;j<cases[i][1].length;j++){
 const actual=r.verification[i][j],expected=render(cases[i][1][j]);
 if(actual.svg!==expected.svg||actual.width!==expected.width||actual.height!==expected.height)throw Error(`SVG differs at ${cases[i][0]} input ${j}`);
}
const fmt=ms=>(ms<1?ms.toFixed(3):ms.toFixed(1))+' ms';
const rows=p.map((row,i)=>{
 if(row.case!==r.cases[i].name||row.diagrams!==r.cases[i].diagrams)throw Error('Workloads differ');
 let value=fmt(r.cases[i].ms);if(r.cases[i].done!==row.diagrams)value+=` (${r.cases[i].done} rendered)`;
 return{case:row.case,diagrams:row.diagrams,pele:row.pele,'zebra-mermaid':value};
});console.table(rows);
const code=readFileSync(resolve(upstream,'dist/pele.js')),cli=readFileSync(resolve(target,'release/zebra-mermaid'));
const result={createdAt:new Date().toISOString(),enabledTypes:r.enabledTypes,commit:'e7f092a4de147cc32210913c20e36eaa5fcd0ef1',scriptSha256:createHash('sha256').update(source).digest('hex'),inputSha256:createHash('sha256').update(readFileSync(input)).digest('hex'),runtime:{node:process.version},protocol:'Original scripts/bench.mjs unmodified: render(src), 1 warm-up batch, >=300 ms, >=5 samples, maximum 2000 batches, upper median; ms per full batch',rows,validation:{success:true,diagrams:cases.reduce((n,c)=>n+c[1].length,0),method:'exact SVG strings and dimensions outside timing'},size:{peleMinifiedBytes:code.length,peleGzipBytes:gzipSync(code).length,rustCliBytes:cli.length,rustCliGzipBytes:gzipSync(cli).length}};
writeFileSync(resolve(dir,'comparison.json'),JSON.stringify(result,null,2)+'\n');
console.log('Saved target/bench-upstream/comparison.json');
