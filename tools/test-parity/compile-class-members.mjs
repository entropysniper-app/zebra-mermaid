#!/usr/bin/env node
// Mechanical port of every original classTypes callback; input/output observations stay symbolic.
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import vm from 'node:vm';
const root=fileURLToPath(new URL('../..',import.meta.url));
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const file='tests/compat/class/classTypes.spec.ts',source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
const inv=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8')),originals=inv.tests.filter(t=>t.file===file);
const sourceSha256=inv.files.find(f=>f.path===file).sha256;if(createHash('sha256').update(source).digest('hex')!==sourceSha256)throw Error('Source hash changed');
let assertions=[];const registered=[],suites=[],expressions=new WeakSet();
class ClassMember{
 constructor(input,kind){if(typeof input!=='string'||!['method','attribute'].includes(kind))throw Error('Unsupported member constructor');this.input=input;this.kind=kind}
 getDisplayDetails(){return new Proxy({}, {get:(_,key)=>{if(!['displayText','cssStyle'].includes(key))throw Error('Unsupported detail');const e={input:this.input,kind:this.kind,key};expressions.add(e);return e}})}
}
const expect=actual=>({toBe(expected){if(!expressions.has(actual)||typeof expected!=='string')throw Error('Unsupported assertion');assertions.push({...actual,expected})}});
const describe=(name,fn)=>{suites.push(name);fn();suites.pop()};const it=(name,fn)=>registered.push({name:[...suites,name].join(' > '),fn});
const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let code=source;for(const s of [...ast.statements].reverse())if(ts.isImportDeclaration(s))code=code.slice(0,s.getStart(ast))+code.slice(s.end);
vm.runInNewContext(ts.transpileModule(code,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText,{describe,it,expect,ClassMember,vi:{spyOn(){throw Error('Spy needs explicit port')}}},{filename:file});
if(registered.length!==originals.length)throw Error('Count differs');const records=[];
for(let i=0;i<registered.length;i++){const t=registered[i],o=originals[i];if(t.name!==o.fullName)throw Error('Identity differs');assertions=[];t.fn();if(!assertions.length)throw Error('No assertion');records.push({id:o.id,index:i,name:t.name,assertions})}
let rust='// Generated from unchanged Mermaid/Pele ClassMember assertions (MIT).\n';for(const r of records)rust+=`#[test]\nfn case_${String(r.index).padStart(2,'0')}() { check("${r.id}"); }\n`;
const artifacts=[['tests/upstream/ir/class-members.json',JSON.stringify({source:file,sourceSha256,records},null,2)+'\n'],['crates/zebra-mermaid/tests/upstream/class_members_generated.rs',rust]];
const path='tests/upstream/ports.json',data=JSON.parse(readFileSync(resolve(root,path),'utf8')),owned=new Set(originals.map(t=>t.id));const previous=data.ports.filter(p=>owned.has(p.upstreamId));data.ports=data.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of records)data.ports.push({upstreamId:r.id,sourceSha256,target:{kind:'rust',binary:'lib',file:'crates/zebra-mermaid/tests/upstream/class_members_generated.rs',test:`diagrams::class::upstream_members::case_${String(r.index).padStart(2,'0')}`,features:['class']},assertionReview:'Unchanged original callback, constructor input, method/attribute kind and both getDisplayDetails properties are retained. Rust Member::display() maps displayText; the parsed classifier maps cssStyle through the same three-value representation as the reference adapter. Every original string equality is executed; no output is used to generate expectations.'});
const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const[p,s]of artifacts)if(readFileSync(resolve(root,p),'utf8')!==s)throw Error('Generated artifact changed');if(JSON.stringify(sort(previous))!==JSON.stringify(sort(data.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Mappings changed')}
else{for(const[p,s]of artifacts)writeFileSync(resolve(root,p),s);writeFileSync(resolve(root,path),JSON.stringify(data,null,2)+'\n')}
console.log(JSON.stringify({translated:records.length,assertions:records.reduce((n,r)=>n+r.assertions.length,0)}));
