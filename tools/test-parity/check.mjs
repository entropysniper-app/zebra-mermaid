#!/usr/bin/env node
// A fail-closed, one-test-to-one-test audit. Golden render fixtures are deliberately not mappings.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const read = p => readFileSync(join(root,p),'utf8');
const hash = s => createHash('sha256').update(s).digest('hex');
const inventory = JSON.parse(read('tests/upstream/inventory.json'));
const { ports } = JSON.parse(read('tests/upstream/ports.json'));
if(ports.some(p=>p.target.file.endsWith('/er_generated.rs'))){
  const verify=spawnSync(process.execPath,['tools/test-parity/compile-er.mjs','--check'],{cwd:root,encoding:'utf8'});
  if(verify.error||verify.status!==0)throw new Error(`ER assertion translation verification failed: ${verify.stderr}`);
}
const full = process.argv.includes('--require-complete');
const run = full || process.argv.includes('--run');
const known = new Map(inventory.tests.map(t=>[t.id,t]));
if (known.size !== inventory.total) throw new Error('Duplicate or missing upstream test identity');
const files = new Map(inventory.files.map(f=>[f.path,f]));
for (const file of inventory.sourceFiles) {
  if(hash(read(`tests/upstream/source/${file.path}`))!==file.sha256) throw new Error(`Frozen source changed: ${file.path}`);
}
for (const file of inventory.files) {
  if (inventory.tests.filter(t=>t.file===file.path).length!==file.tests) throw new Error(`File test count differs: ${file.path}`);
}
const byId=new Map(),targets=new Set(),features=new Set(),sourceHashes={};
const binaryOf=port=>port.target.binary??'upstream_parity';
for(const port of ports){
  const test=known.get(port.upstreamId);
  if(!test||byId.has(test.id)) throw new Error(`Unknown/duplicate mapping: ${port.upstreamId}`);
  if(port.sourceSha256!==files.get(test.file).sha256) throw new Error(`Mapping needs re-review: ${test.file}`);
  if(port.target.kind!=='rust'||!port.assertionReview?.trim()) throw new Error(`Unreviewed or unsupported test adapter: ${test.id}`);
  if(targets.has(port.target.test)) throw new Error(`Multiple upstream cases map to one test: ${port.target.test}`);
  if(!['lib','upstream_parity'].includes(binaryOf(port)))throw new Error('Unsupported Rust test binary');
  if(!port.target.file.startsWith('crates/zebra-mermaid/tests/upstream/')||port.target.file.includes('..')) throw new Error('Invalid test target path');
  sourceHashes[port.target.file]=hash(read(port.target.file));
  byId.set(test.id,port);targets.add(port.target.test);
  for(const feature of port.target.features) features.add(feature);
}
const observed=new Map();let processFailed=false;
const output=join(root,'target/test-parity');mkdirSync(output,{recursive:true});
if(run&&ports.length){
  const args=['test','-p','zebra-mermaid','--test','upstream_parity','--no-run','--message-format=json'];
  if(ports.some(p=>binaryOf(p)==='lib'))args.push('--lib');
  if(features.size)args.push('--features',[...features].sort().join(','));
  const build=spawnSync('cargo',args,{cwd:root,encoding:'utf8',maxBuffer:32*1024*1024});
  writeFileSync(join(output,'build.log'),build.stdout+build.stderr);
  if(build.error)throw build.error;
  if(build.status!==0)throw new Error('Rust test build failed; see target/test-parity/build.log');
  const artifacts=build.stdout.trim().split('\n').flatMap(line=>{try{return[JSON.parse(line)]}catch{return[]}});
  const bins=artifacts.filter(a=>a.reason==='compiler-artifact'&&a.executable&&a.profile?.test);
  const expectedBinaries=new Set(ports.map(binaryOf));
  if(bins.length!==expectedBinaries.size)throw new Error('Expected one executable for each mapped test binary');
  for(const artifact of bins){
  const binary=artifact.target.name==='upstream_parity'?'upstream_parity':'lib';
  const wanted=new Set(ports.filter(p=>binaryOf(p)===binary).map(p=>p.target.test));
  const listed=spawnSync(artifact.executable,['--list','--format','terse'],{cwd:root,encoding:'utf8'});
  if(listed.error||listed.status!==0)throw new Error('Could not list actual Rust test cases');
  const runnable=new Set(listed.stdout.split('\n').filter(l=>l.endsWith(': test')).map(l=>l.slice(0,-6)));
  for(const target of wanted)if(!runnable.has(target))throw new Error(`Mapped test was not built: ${target}`);
  if(binary==='upstream_parity')for(const target of runnable)if(!targets.has(target))throw new Error(`Rust test lacks an upstream mapping: ${target}`);
  const result=spawnSync(artifact.executable,['--test-threads=1','--format','pretty'],{cwd:root,encoding:'utf8',maxBuffer:32*1024*1024});
  if(result.error)throw result.error;
  writeFileSync(join(output,`rust-${binary}-tests.log`),result.stdout+result.stderr);
  processFailed ||= result.status!==0;
  for(const match of result.stdout.matchAll(/^test (.*?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$/gm)){
    if(wanted.has(match[1]))observed.set(match[1],match[2]==='ok'?'passed':match[2]==='FAILED'?'failed':'skipped');
  }
  for(const target of wanted)if(!observed.has(target))processFailed=true;
  }
}
const summary={total:inventory.total,mapped:ports.length,unmapped:inventory.total-ports.length,passed:0,failed:0,skipped:0,unverified:0};
const cases=inventory.tests.map(test=>{
  const port=byId.get(test.id);const state=port?(observed.get(port.target.test)??'unverified'):'unmapped';
  if(state!=='unmapped')summary[state]++;
  return {id:test.id,file:test.file,name:test.fullName,upstreamMode:test.mode,target:port?.target.test??null,state};
});
// A skipped/todo reference case never becomes a passing target merely by being preserved as ignored.
const complete=!processFailed&&summary.passed===inventory.total;
const report={commit:inventory.commit,upstreamBaseline:inventory.baseline,summary,complete,sourceHashes,cases};
writeFileSync(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({upstream:inventory.baseline,...summary,complete},null,2));
if(processFailed||summary.failed||(full&&!complete))process.exitCode=1;
