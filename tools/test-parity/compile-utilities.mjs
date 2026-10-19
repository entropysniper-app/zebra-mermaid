#!/usr/bin/env node
// Freeze only the original tests' independent reference values, never Pele/Rust outputs.
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import vm from 'node:vm';
const root=fileURLToPath(new URL('../..',import.meta.url));
const require=createRequire(resolve(root,'.research/pele/package.json'));
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const {sanitizeUrl:reference}=require('@braintree/sanitize-url');
const {load,JSON_SCHEMA}=require('js-yaml');
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8'));
const records=[];
const hash=s=>createHash('sha256').update(s).digest('hex');
const sources=['tests/util/url.test.ts','tests/util/yaml.test.ts','tests/util/preprocess.test.ts'];
for(const file of sources){
 const source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
 if(hash(source)!==inventory.files.find(f=>f.path===file).sha256)throw Error('Changed original source');
 const original=inventory.tests.filter(t=>t.file===file),registered=[],suites=[];
 const describe=(name,fn)=>{suites.push(name);fn();suites.pop()};
 const it=(name,fn)=>registered.push({name:[...suites,name].join(' > '),fn});
 let calls=[];
 const actual=(op,source)=>({op,source});
 const expect=value=>({toBe:expected=>record('toBe',value,expected),toEqual:expected=>record('toEqual',value,expected)});
 function record(matcher,value,expected){if(!value||!['url','yaml','entities'].includes(value.op)||typeof value.source!=='string')throw Error('Unsupported actual expression');calls.push({...value,matcher,expected})}
 const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 let executable=source;for(const s of [...ast.statements].reverse())if(ts.isImportDeclaration(s))executable=executable.slice(0,s.getStart(ast))+executable.slice(s.end);
 const corpus=readFileSync(resolve(root,'tests/upstream/source/tests/support/corpus.ts'),'utf8');
 const ca=ts.createSourceFile('corpus.ts',corpus,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const rng=ca.statements.find(s=>ts.isFunctionDeclaration(s)&&s.name.text==='random').getText(ca).replace('export ','');
 executable=ts.transpileModule(rng+'\n'+executable,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
 vm.runInNewContext(executable,{describe,it,expect,reference,load,JSON_SCHEMA,sanitizeUrl:s=>actual('url',s),parseYaml:s=>actual('yaml',s),encodeEntities:s=>actual('entities',s)},{filename:file,timeout:10000});
 if(registered.length!==original.length)throw Error('Different registration count');
 for(let index=0;index<registered.length;index++){
  const test=registered[index],item=original[index];if(test.name!==item.fullName)throw Error('Different identity');
  // The remaining callbacks have explicit manual ports; no callback is silently dropped.
  if(file.includes('yaml')&&index>=25||file.includes('preprocess')&&index>=2)continue;
  calls=[];test.fn();if(!calls.length)throw Error('No assertion');
  records.push({id:item.id,file,index,name:item.fullName,assertions:calls});
 }
}
let unterminatedYamlThrows=false;try{load('{ a: 1',{schema:JSON_SCHEMA})}catch{unterminatedYamlThrows=true}
if(!unterminatedYamlThrows)throw Error('Independent js-yaml rejection assertion failed');
const artifacts=[];
artifacts.push(['tests/upstream/ir/utilities.json',JSON.stringify({referenceChecks:{unterminatedYamlThrows},sources:sources.map(file=>({file,sha256:inventory.files.find(f=>f.path===file).sha256})),reference:'Original @braintree/sanitize-url, js-yaml JSON_SCHEMA and preprocess regex reference; random seed 11, original 20000 iterations and exact LCG call order.',records},null,2)+'\n']);
let rust='// Generated from frozen upstream callbacks by compile-utilities.mjs.\n';
for(const r of records.filter(r=>!r.file.includes('preprocess'))){const kind=r.file.includes('yaml')?'yaml':'url';rust+=`#[test]\nfn ${kind}_${String(r.index).padStart(2,'0')}() { check("${r.id}"); }\n`;}
artifacts.push(['crates/zebra-mermaid/tests/upstream/utilities_generated.rs',rust]);
const pp='tests/upstream/ports.json',ports=JSON.parse(readFileSync(resolve(root,pp),'utf8'));
const owned=new Set(records.map(r=>r.id)),previous=ports.ports.filter(p=>owned.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of records){const preprocess=r.file.includes('preprocess'),kind=r.file.includes('yaml')?'yaml':'url';ports.ports.push({upstreamId:r.id,sourceSha256:inventory.files.find(f=>f.path===r.file).sha256,target:{kind:'rust',binary:preprocess?'lib':'upstream_parity',file:preprocess?'crates/zebra-mermaid/tests/upstream/preprocess.rs':'crates/zebra-mermaid/tests/upstream/utilities_generated.rs',test:preprocess?`preprocess::upstream_tests::entities_${r.index}`:`utilities::${kind}_${String(r.index).padStart(2,'0')}`,features:[]},assertionReview:'Original callback and parameter rows execute during translation with symbolic actual values. Expected values come only from the independent reference explicitly invoked by the original assertion. Every captured assertion runs against the Rust function. Entity random inputs retain seed 11, all 20000 iterations and RNG call order; Rust independently checks the generated sequence. No Pele or Rust outputs become expectations.'})}
const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const[p,s]of artifacts)if(readFileSync(resolve(root,p),'utf8')!==s)throw Error(`Generated artifact changed: ${p}`);if(JSON.stringify(sort(previous))!==JSON.stringify(sort(ports.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Mapping changed')}
else{for(const[p,s]of artifacts)writeFileSync(resolve(root,p),s);writeFileSync(resolve(root,pp),JSON.stringify(ports,null,2)+'\n')}
console.log(JSON.stringify({translated:records.length,assertions:records.reduce((n,r)=>n+r.assertions.length,0)}));
