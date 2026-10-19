#!/usr/bin/env node
// Original date callbacks emit symbolic calls; all expectations remain original literals/Date constructors.
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {format as titleFormat} from 'node:util';
import vm from 'node:vm';
process.env.TZ='UTC';
const root=fileURLToPath(new URL('../..',import.meta.url));
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const file='tests/gantt/dates.test.ts',source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
const inv=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8'));
const originals=inv.tests.filter(t=>t.file===file),sourceSha256=inv.files.find(f=>f.path===file).sha256;
if(createHash('sha256').update(source).digest('hex')!==sourceSha256)throw Error('Source hash changed');
const expressions=new WeakMap(),registered=[],suites=[];let assertions=[];
function value(v){
 if(v===undefined)return{op:'undefined'};
 if(v!==null&&(typeof v==='object'||typeof v==='function')&&expressions.has(v))return expressions.get(v);
 if(Array.isArray(v))return{op:'array',items:v.map(value)};
 if(Object.prototype.toString.call(v)==='[object Date]')return{op:'date',time:value(v.getTime())};
 if(typeof v==='number'&&!Number.isFinite(v))return{op:'number',value:String(v)};
 if(v===null||['string','number','boolean'].includes(typeof v))return{op:'literal',value:v};
 throw Error('Unsupported value');
}
function sym(expr){const p=new Proxy({}, {get(_,k){
 if(k===Symbol.toPrimitive)throw Error('Numeric coercion needs manual port');
 if(k==='length')return sym({op:'length',base:expr});
 if(k==='getTime')return()=>sym({op:'getTime',base:expr});
 throw Error('Unsupported property '+String(k));
 }});expressions.set(p,expr);return p}
const functions=Object.fromEntries(['parseStrict','format','add','parseDuration','timeFormat','autoTicks','intervalTicks'].map(op=>[op,(...args)=>sym({op,args:args.map(value)})]));
const expect=actual=>new Proxy({}, {get(_,matcher){if(!['toEqual','toBe','toBeUndefined','toBeLessThanOrEqual'].includes(matcher))throw Error('Unsupported matcher');return expected=>assertions.push({matcher,actual:value(actual),expected:value(expected)})}});
const describe=(name,fn)=>{suites.push(name);fn();suites.pop()};
const it=(name,fn)=>registered.push({name:[...suites,name].join(' > '),fn});
it.each=rows=>(name,fn)=>{for(const row of rows){const args=Array.isArray(row)?row:[row];it(titleFormat(name,...args.slice(0,(name.match(/%s/g)??[]).length)),()=>fn(...args))}};
const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let code=source;
for(const s of [...ast.statements].reverse())if(ts.isImportDeclaration(s))code=code.slice(0,s.getStart(ast))+code.slice(s.end);
code=ts.transpileModule(code,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
vm.runInNewContext(code,{...functions,describe,it,expect,Number:{isNaN:n=>expressions.has(n)?sym({op:'isNaN',base:value(n)}):Number.isNaN(n)}},{filename:file});
if(originals.length!==registered.length)throw Error('Count differs');
const records=[],manual=[];
for(let index=0;index<registered.length;index++){
 const t=registered[index],o=originals[index];// Four titles interpolate local Date.getTime(); preserve the inventory identity while running in UTC.
 const canonical=n=>[55,56,83,84].includes(index)?n.replace(/ as \d+$/, ' as <date>'):n;
 if(canonical(t.name)!==canonical(o.fullName))throw Error(`Identity differs ${index}: ${t.name}`);
 if(t.name==='date arithmetic > adds exact hours, minutes, seconds, and milliseconds'){manual.push({id:o.id,index,name:t.name});continue}
 assertions=[];t.fn();if(!assertions.length)throw Error('No assertions');records.push({id:o.id,index,name:o.fullName,generatedName:t.name,assertions});
}
if(manual.length!==1)throw Error('Manual adapter count changed');
let rust='// Generated from unchanged Pele Gantt date assertions. One function per original test.\n';
for(const r of records)rust+=`#[test]\nfn case_${String(r.index).padStart(2,'0')}() { check("${r.id}"); }\n`;
const artifacts=[['tests/upstream/ir/gantt-dates.json',JSON.stringify({source:file,sourceSha256,timezone:'UTC',records,manual},null,2)+'\n'],['crates/zebra-mermaid/tests/upstream/gantt_dates_generated.rs',rust]];
const path='tests/upstream/ports.json',data=JSON.parse(readFileSync(resolve(root,path),'utf8')),owned=new Set(originals.map(t=>t.id));const previous=data.ports.filter(p=>owned.has(p.upstreamId));data.ports=data.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of [...records,...manual]){const isManual=manual.includes(r);data.ports.push({upstreamId:r.id,sourceSha256,target:{kind:'rust',binary:'lib',file:`crates/zebra-mermaid/tests/upstream/gantt_dates${isManual?'':'_generated'}.rs`,test:`diagrams::gantt::upstream_dates::${isManual?'adds_exact_time_units':'case_'+String(r.index).padStart(2,'0')}`,features:['gantt']},assertionReview:'Original callbacks and every parameter row preserved. Calls remain symbolic until execution against Rust date/axis functions; expected Date constructors, arrays, NaN and undefined remain typed, with UTC as the deterministic test time zone. Actual-result numeric coercion is handled by the explicit manual port retaining all four subtraction assertions. No renderer/reference output is used as an expectation.'})}
const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const[p,s]of artifacts)if(readFileSync(resolve(root,p),'utf8')!==s)throw Error('Generated artifact changed: '+p);if(JSON.stringify(sort(previous))!==JSON.stringify(sort(data.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Mappings changed')}
else{for(const[p,s]of artifacts)writeFileSync(resolve(root,p),s);writeFileSync(resolve(root,path),JSON.stringify(data,null,2)+'\n')}
console.log(JSON.stringify({registered:originals.length,translated:records.length,manual:manual.length,assertions:records.reduce((n,r)=>n+r.assertions.length,0)+4}));
