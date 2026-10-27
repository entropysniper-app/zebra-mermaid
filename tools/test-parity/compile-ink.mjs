#!/usr/bin/env node
// Translate original ink callbacks symbolically. Every call executes once in Rust; no renderer output is an expectation.
import {readFileSync,writeFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import vm from 'node:vm';
import {formatRust} from './format-rust.mjs';
const root=resolve(import.meta.dirname,'../..');
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json')));
const file='tests/render/ink.test.ts',source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
const sourceSha256=createHash('sha256').update(source).digest('hex');
if(sourceSha256!==inventory.files.find(f=>f.path===file)?.sha256)throw Error('Frozen ink source differs');
const originals=inventory.tests.filter(t=>t.file===file),registered=[],ancestry=[];
const describe=(name,fn)=>{ancestry.push(name);fn();ancestry.pop()};
const it=(name,fn)=>registered.push({name:[...ancestry,name].join(' > '),fn});
let actions=[],next=0;
const resolveStyle=declarations=>({shape:{declarations},text:{declarations}});
const ink=(shape,text)=>{if(shape.declarations!==text.declarations)throw Error('Unreviewed style use');const slot=next++;actions.push({op:'ink',slot,declarations:shape.declarations});return{$ink:slot}};
const expect=actual=>({toBe:expected=>actions.push({op:'equal',actual,expected}),toBeUndefined:()=>actions.push({op:'equal',actual,expected:{$undefined:true}}),toHaveLength:expected=>{if(!Array.isArray(actual))throw Error('Unreviewed length expression');actions.push({op:'length',actual,expected})}});
const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true);
let code=source;for(const s of [...ast.statements].reverse())if(ts.isImportDeclaration(s))code=code.slice(0,s.getStart(ast))+code.slice(s.end);
code=ts.transpileModule(code,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
vm.runInNewContext(code,{describe,it,expect,resolveStyle,ink},{filename:file,timeout:10000});
if(registered.length!==originals.length)throw Error('Ink registration count differs');
const records=[];
for(let index=0;index<registered.length;index++){
 const test=registered[index],item=originals[index];if(test.name!==item.fullName)throw Error('Ink test identity differs');
 actions=[];next=0;test.fn();if(!actions.some(a=>a.op==='ink')||!actions.some(a=>a.op==='equal'))throw Error('Empty ink callback');
 records.push({id:item.id,index,name:item.fullName,actions});
}
const artifacts=[['tests/upstream/ir/ink.json',JSON.stringify({file,sourceSha256,method:'Original callbacks and loops evaluated with symbolic actual values. Slots retain call order and reuse, including one expected hex call and two named calls per each of 148 rows. Literal expectations and assertion relations are retained; no Pele or Rust engine outputs become expectations.',records},null,2)+'\n']];
let rust='// Generated from frozen original callbacks by compile-ink.mjs.\n';
for(const r of records)rust+=`#[test]\nfn case_${String(r.index).padStart(2,'0')}(){check("${r.id}");}\n`;
artifacts.push(['crates/zebra-mermaid/tests/upstream/ink_generated.rs',formatRust(rust)]);
const portsPath=resolve(root,'tests/upstream/ports.json'),ports=JSON.parse(readFileSync(portsPath));
const ids=new Set(records.map(r=>r.id));const before=ports.ports.filter(p=>ids.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!ids.has(p.upstreamId));
for(const r of records)ports.ports.push({upstreamId:r.id,sourceSha256,target:{kind:'rust',file:'crates/zebra-mermaid/tests/upstream/ink_generated.rs',test:`ink::case_${String(r.index).padStart(2,'0')}`,features:[]},assertionReview:'Complete original callback translated symbolically. Every resolveStyle/ink invocation is retained once in original order; result slots preserve reuse of the hex expectation and both named-case assertions for each of all 148 literal rows. Original expected strings, undefined outcomes and table length remain assertions. No Rust outputs or sampled-input aggregation are used.'});
const sorted=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const[p,s]of artifacts)if(readFileSync(resolve(root,p),'utf8')!==s)throw Error('Changed ink translation: '+p);if(JSON.stringify(sorted(before))!==JSON.stringify(sorted(ports.ports.filter(p=>ids.has(p.upstreamId)))))throw Error('Changed ink mapping');}
else{for(const[p,s]of artifacts)writeFileSync(resolve(root,p),s);writeFileSync(portsPath,JSON.stringify(ports,null,2)+'\n');}
console.log(JSON.stringify({translated:records.length,calls:records.reduce((n,r)=>n+r.actions.filter(a=>a.op==='ink').length,0),assertions:records.reduce((n,r)=>n+r.actions.filter(a=>a.op!=='ink').length,0)}));
