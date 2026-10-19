#!/usr/bin/env node
// Serial isolated execution preserves original per-test timeouts and avoids stress-case interference.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../..',import.meta.url)),dir=resolve(root,'target/test-parity');mkdirSync(dir,{recursive:true});
const release=process.argv.includes('--release'),filter=process.argv.slice(2).filter(a=>a!=='--release').join(' ');
const args=['test','-p','zebra-mermaid','--lib','--no-run','--message-format=json'];if(release)args.push('--release');
const build=spawnSync('cargo',args,{cwd:root,encoding:'utf8',maxBuffer:32*1024*1024});
writeFileSync(resolve(dir,'stress-build.log'),build.stdout+build.stderr);if(build.error||build.status!==0)throw Error('Stress build failed; see target/test-parity/stress-build.log');
const artifacts=build.stdout.split('\n').flatMap(s=>{try{return[JSON.parse(s)]}catch{return[]}});
const binary=artifacts.find(a=>a.reason==='compiler-artifact'&&a.executable&&a.profile.test)?.executable;if(!binary)throw Error('No test executable');
const listed=spawnSync(binary,['--list','--format','terse'],{cwd:root,encoding:'utf8'});if(listed.status!==0)throw Error('Cannot list tests');
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/stress.json'))),ports=JSON.parse(readFileSync(resolve(root,'tests/upstream/ports.json'))).ports;
const results=[];
for(const f of data.files)for(const r of f.records){
 if(filter&&!`${f.source} ${r.name}`.includes(filter))continue;
 const target=ports.find(p=>p.upstreamId===r.id).target.test;
 if(!listed.stdout.split('\n').includes(target+': test'))throw Error('Missing compiled test '+target);
 const start=performance.now();const result=spawnSync(binary,[target,'--exact','--nocapture','--test-threads=1'],{cwd:root,encoding:'utf8',timeout:r.timeoutMs,maxBuffer:16*1024*1024});
 const state=result.status===0?'passed':result.error?.code==='ETIMEDOUT'?'timeout':'failed';
 const output=(result.stdout??'')+(result.stderr??'');const elapsed=output.match(/stress_elapsed_ms=([\d.]+)/);
 const record={id:r.id,source:f.source,name:r.name,target,state,budgetMs:r.budgetMs??null,timeoutMs:r.timeoutMs,elapsedMs:elapsed?Number(elapsed[1]):null,processElapsedMs:performance.now()-start,exitCode:result.status,signal:result.signal,output};results.push(record);
 console.log(`${state}: ${f.source} #${r.index} ${r.name}${elapsed?' ('+elapsed[1]+' ms)':''}`);
}
const summary={total:results.length,passed:results.filter(r=>r.state==='passed').length,failed:results.filter(r=>r.state==='failed').length,timeout:results.filter(r=>r.state==='timeout').length};
writeFileSync(resolve(dir,`stress-${release?'release':'debug'}-results.json`),JSON.stringify({mode:release?'release':'debug',binarySha256:createHash('sha256').update(readFileSync(binary)).digest('hex'),summary,results},null,2)+'\n');
console.log(JSON.stringify(summary));if(summary.failed||summary.timeout)process.exitCode=1;
