// Replay unchanged archived requests against the current pinned Node oracle. Historical
// fixtures stay immutable; every new expected value is generated solely by upstream Pele.
import {readFileSync,readdirSync,writeFileSync,mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {gzipSync,gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {loadOracle,answer} from './oracle.mjs';
const inventory=JSON.parse(readFileSync('tests/upstream/inventory.json'));
if(execFileSync('git',['-C','.research/pele','rev-parse','HEAD'],{encoding:'utf8'}).trim()!==inventory.commit)throw Error('Unexpected reference revision');
execFileSync('git',['-C','.research/pele','diff','--exit-code','HEAD','--','src']);
const version=JSON.parse(readFileSync('.research/pele/package.json')).version;
const base='tests/fixtures',out=`${base}/pele-${version}`;
const oracle=await loadOracle();
const sha=b=>createHash('sha256').update(b).digest('hex');
const files=[],unrepresentable=[];
const wellFormed=v=>typeof v==='string'?v.isWellFormed():Array.isArray(v)?v.every(wellFormed):v&&typeof v==='object'?Object.entries(v).every(([k,x])=>k.isWellFormed()&&wellFormed(x)):true;
for(const dir of ['corpus','harvest','unit']){
 mkdirSync(`${out}/${dir}`,{recursive:true});
 for(const name of readdirSync(`${base}/${dir}`).filter(n=>/\.jsonl(?:\.gz)?$/.test(n)).sort()){
  // JS number formatting/transcendentals do not depend on a Pele release.
  if(dir==='unit'&&!['label.jsonl','entities.jsonl','preprocess.jsonl','yaml.jsonl','url.jsonl','style.jsonl'].includes(name))continue;
  const path=`${dir}/${name}`,bytes=readFileSync(`${base}/${path}`);
  const text=(name.endsWith('.gz')?gunzipSync(bytes):bytes).toString();
  const lines=text.trimEnd().split('\n');
  const rows=lines.map(line=>{
   const row=JSON.parse(line),request={...row,op:dir==='unit'?name.replace('.jsonl',''):'render'};
   if(row.icons)request.options={...row.options,icons:row.icons};
   delete request.expect;
   const actual=answer(oracle,request);delete actual.id;
   if(request.op==='label')row.expect=actual.label;
   else if(['entities','style'].includes(request.op))row.expect=actual.value;
   else if(dir==='unit'&&!actual.ok)row.expect={ok:false,message:actual.error.message,...('code' in row.expect?{code:actual.error.code}:{} )};
   else row.expect=actual;
   if(!wellFormed(row))unrepresentable.push({input:`${base}/${path}`,record:JSON.stringify(row),reason:'An unchanged archived input or its Node reference contains isolated UTF-16 surrogates; not representable by the UTF-8 Rust API, not counted as passed.'});
   return row;
  });
  const data=Buffer.from(rows.map(r=>JSON.stringify(r)).join('\n')+'\n');
  writeFileSync(`${out}/${path}`,name.endsWith('.gz')?gzipSync(data,{level:9}):data);
  files.push({input:`${base}/${path}`,inputSha256:sha(bytes),output:`${out}/${path}`,cases:rows.length});
 }
}
writeFileSync(`${out}/unrepresentable-inputs.json`,JSON.stringify({commit:inventory.commit,entries:unrepresentable},null,2)+'\n');
const oldKnown=JSON.parse(readFileSync(`${base}/known-differences.json`));
const retained=oldKnown.entries.filter(e=>!answer(oracle,{op:e.suite,source:e.input}).ok);
writeFileSync(`${out}/known-differences.json`,JSON.stringify({note:'Historical entries are retained only when the pinned upstream still exhibits that difference; exclusions are not passes.',entries:retained},null,2)+'\n');
writeFileSync(`${out}/metadata.json`,JSON.stringify({version,commit:inventory.commit,runtime:{node:process.version},method:'Unchanged archived requests, options and order; expected values regenerated only by the fixed upstream Node source. No Rust output is used. These supplemental fixtures are separate from original-test mappings.',files},null,2)+'\n');
console.log(JSON.stringify({version,files:files.length,cases:files.reduce((n,f)=>n+f.cases,0)}));
