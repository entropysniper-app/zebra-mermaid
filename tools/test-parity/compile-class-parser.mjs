#!/usr/bin/env node
import {formatRust} from './format-rust.mjs';
// Mechanical translation of class parser calls AND assertions. No Pele/Rust outputs are harvested.
// Model observations are symbolic until the generated Rust test executes its real parser.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {format} from 'node:util';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../..',import.meta.url));
const require=createRequire(resolve(root,'.research/pele/website/package.json'));
const ts=require('typescript');
const file='tests/compat/class/classDiagram.spec.ts';
const source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8'));
const originals=inventory.tests.filter(t=>t.file===file);
const fileHash=inventory.files.find(f=>f.path===file).sha256;
if(createHash('sha256').update(source).digest('hex')!==fileHash)throw Error('Source hash changed');
class Unsupported extends Error{}
const expressions=new WeakMap();let actions=[];
const literal=value=>{
 if(value===undefined)return{op:'undefined'};
 if(typeof value==='number'&&(!Number.isFinite(value)||Object.is(value,-0)))throw new Unsupported('non-finite or negative-zero literal requires typed encoding');
 if(value!==null&&(typeof value==='object'||typeof value==='function')&&expressions.has(value))return expressions.get(value);
 if(Array.isArray(value))return{op:'array',items:value.map(literal)};
 if(Object.prototype.toString.call(value)==='[object Map]')return{op:'map',entries:[...value].map(([k,v])=>[literal(k),literal(v)])};
 if(value!==null&&typeof value==='object')return{op:'object',entries:Object.entries(value).map(([k,v])=>[k,literal(v)])};
 if(!['string','number','boolean'].includes(typeof value)&&value!==null)throw new Unsupported('nonliteral expression');
 return{op:'literal',value};
};
function sym(expr){
 const proxy=new Proxy(function(){},{
  get(_,key){
   if(key===Symbol.toPrimitive||key===Symbol.iterator)return()=>{throw new Unsupported('implicit conversion/iteration of model value')};
   if(typeof key==='symbol')throw new Unsupported('symbol property');
   if(key==='getDisplayDetails')return()=>sym({op:'property',base:expr,key:'displayDetails'});
   if(key==='get'||key==='has')return arg=>sym({op:key,base:expr,key:literal(arg)});
   return sym({op:'property',base:expr,key});
  },
  apply(){throw new Unsupported('unsupported model method')},
  ownKeys(){throw new Unsupported('model enumeration requires an explicit adapter')},
 });
 expressions.set(proxy,expr);return proxy;
}
function inspect(fn){
 const ast=ts.createSourceFile('test.js',`(${fn.toString()})`,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const forbidden=new Set([ts.SyntaxKind.AsyncKeyword,ts.SyntaxKind.AwaitExpression,ts.SyntaxKind.IfStatement,ts.SyntaxKind.ConditionalExpression,ts.SyntaxKind.SwitchStatement,ts.SyntaxKind.WhileStatement,ts.SyntaxKind.ForStatement,ts.SyntaxKind.ForInStatement,ts.SyntaxKind.ForOfStatement,ts.SyntaxKind.TypeOfExpression,ts.SyntaxKind.BinaryExpression,ts.SyntaxKind.PrefixUnaryExpression]);
 // Branches cannot be evaluated by JS while observations are symbolic. Fail closed.
 const visit=n=>{if(forbidden.has(n.kind)&&!(ts.isBinaryExpression(n)&&n.operatorToken.kind===ts.SyntaxKind.PlusToken))throw new Unsupported(`control flow/operator needs manual translation: ${ts.SyntaxKind[n.kind]}`);ts.forEachChild(n,visit);};visit(ast);
}
function nested(fn){inspect(fn);const parent=actions;actions=[];const result=fn();if(result&&typeof result.then==='function')throw new Unsupported('async test body');const out=actions;actions=parent;return out;}
const dependent=e=>e.op==='root'||Object.values(e).some(v=>v&&typeof v==='object'&&(Array.isArray(v)?v.some(x=>x&&typeof x==='object'&&dependent(x)):dependent(v)));
function expect(actual){
 const make=negated=>new Proxy({}, {get(_,name){if(name==='not')return make(!negated);if(name==='toHaveBeenCalledWith')return(...args)=>{const expr=literal(actual);if(expr.op!=='root'||!expr.key.startsWith('calls:'))throw new Unsupported('call assertion on non-spy');actions.push({op:'callAssert',method:expr.key,negated,args:args.map(literal)})};return expected=>{
  if(name==='toThrow'||name==='toThrowError'){
   if(typeof actual!=='function'||expressions.has(actual))throw new Unsupported('unsupported throw matcher');
   const pattern=expected===undefined?null:Object.prototype.toString.call(expected)==='[object RegExp]'?{source:expected.source,flags:expected.flags}:null;
   if(expected!==undefined&&(!pattern||pattern.source!=="(Expecting\\s'STR'|Unrecognized\\stext)"||pattern.flags!==''))throw new Unsupported('unmapped throw pattern');
   actions.push({op:'throws',negated,actions:nested(actual),...(pattern?{pattern}:{})});return;
  }
  if(!['toBe','toEqual','toContain','toHaveLength','toBeUndefined','toBeDefined','toBeGreaterThan','toBeLessThan','toBeGreaterThanOrEqual','toBeLessThanOrEqual'].includes(name))throw new Unsupported(`matcher ${String(name)}`);
  const expr=literal(actual);if(!dependent(expr))throw new Unsupported('assertion has no symbolic model observation');
  actions.push({op:'assert',matcher:name,negated,actual:expr,expected:literal(expected)});
 }} });return make(false);
}
expect.objectContaining=()=>{throw new Unsupported('asymmetric object matcher requires explicit translation')};
expect.any=()=>{throw new Unsupported('asymmetric type matcher requires explicit translation')};
class ClassDB{
 constructor(){actions.push({op:'reset'});this.lineType={LINE:0,DOTTED_LINE:1};this.relationType={AGGREGATION:0,EXTENSION:1,COMPOSITION:2,DEPENDENCY:3,LOLLIPOP:4};return new Proxy(this,{get(target,key,receiver){if(Reflect.has(target,key))return Reflect.get(target,key,receiver);if(['setLink','setTooltip','setClickEvent'].includes(key))return sym({op:'root',key:'calls:'+key});throw new Unsupported('unmapped ClassDB operation '+String(key))}})}
 clear(){actions.push({op:'reset'})}
 getClasses(){return sym({op:'root',key:'classes'})}
 getClass(id){return this.getClasses().get(id)}
 getRelations(){return sym({op:'root',key:'relations'})}
 getNotes(){return sym({op:'root',key:'notes'})}
 getNote(id){return this.getNotes().get(typeof id==='number'?`note${id}`:id)}
 getNamespaces(){return sym({op:'root',key:'namespaces'})}
 getNamespace(id){return this.getNamespaces().get(id)}
 getDirection(){return sym({op:'root',key:'direction'})}
 getAccTitle(){return sym({op:'root',key:'accTitle'})}
 getAccDescription(){return sym({op:'root',key:'accDescr'})}
 getData(){return sym({op:'root',key:'data'})}
}
const parser={yy:null,parse(source){if(typeof source!=='string')throw new Unsupported('non-string source');actions.push({op:'parse',source});}};
const suites=[];const hooks=[];const registered=[];
function describe(name,fn){suites.push(name);hooks.push([]);fn();hooks.pop();suites.pop();}
function register(mode,name,fn){registered.push({name,fullName:[...suites,name].join(' > '),fn,hooks:hooks.flat(),mode});}
function it(name,fn){register('run',name,fn)}it.skip=(name,fn)=>register('skip',name,fn);it.todo=name=>register('todo',name,()=>{});
it.each=rows=>(name,fn)=>{for(const row of rows){const args=Array.isArray(row)?row:[row];register('run',format(name,...args),()=>{inspect(fn);return fn(...args)});}};
// Imports are removed by their parsed source spans, not a regex that might alter a string.
const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
let executable=source;for(const statement of [...ast.statements].reverse())if(ts.isImportDeclaration(statement))executable=executable.slice(0,statement.getStart(ast))+executable.slice(statement.end);
const transformed=ts.transform(ts.createSourceFile(file,executable,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS),[ctx=>{
 const visit=n=>{if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&ts.isIdentifier(n.expression.expression)&&n.expression.expression.text==='Object'&&n.expression.name.text==='keys')return ts.factory.updateCallExpression(n,ts.factory.createIdentifier('__objectKeys'),n.typeArguments,n.arguments);return ts.visitEachChild(n,visit,ctx)};
 return sf=>ts.visitNode(sf,visit);
}]);executable=ts.createPrinter().printFile(transformed.transformed[0]);transformed.dispose();
executable=ts.transpileModule(executable,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
vm.runInNewContext(executable,{__objectKeys:value=>sym({op:'mapOwnKeys',base:literal(value)}),describe,it,beforeEach:fn=>hooks.at(-1).push(fn),expect,ClassDB,parser,vi:{spyOn(target,method){if(!(target instanceof ClassDB)||!['setLink','setTooltip','setClickEvent'].includes(method))throw new Unsupported('unsupported ClassDB spy');actions.push({op:'observe',method,enabled:true});return target[method]}}},{filename:file,timeout:10000});
if(registered.length!==originals.length)throw Error(`case count changed: ${registered.length}/${originals.length}`);
const records=[],untranslated=[];
for(let i=0;i<registered.length;i++){
 const test=registered[i],original=originals[i];if(test.fullName!==original.fullName||test.mode!==original.mode)throw Error(`case identity changed at ${i}: ${test.fullName}`);
 try{actions=[];for(const hook of test.hooks)hook();actions.push(...nested(test.fn));if(!actions.some(a=>a.op==='parse'||a.op==='throws'||a.op==='assert'))throw new Unsupported('no parser call or model assertion');records.push({id:original.id,index:i,name:test.fullName,mode:test.mode,actions});}
 catch(e){if(!(e instanceof Unsupported))throw e;untranslated.push({id:original.id,index:i,reason:e.message});}
}
mkdirSync(resolve(root,'tests/upstream/ir'),{recursive:true});
const artifacts=[];
artifacts.push([resolve(root,'tests/upstream/ir/class-parser.json'),JSON.stringify({source:file,sourceSha256:fileHash,records,untranslated},null,2)+'\n']);
let rust='// Generated by tools/test-parity/compile-class-parser.mjs from unchanged upstream assertions.\n// Mermaid (c) 2014-2022 Knut Sveidqvist, MIT. Each parameter expansion is a separate test.\n';
for(const record of records){rust+=`${record.mode==='skip'?'// Upstream skip retained in inventory; execute the unchanged assertion in Rust.\n':''}#[test]\n${record.mode==='todo'?'#[ignore = "Upstream todo has no assertion; not a pass"]\n':''}fn case_${record.index.toString().padStart(3,'0')}() { run_case("${record.id}"); }\n`;}
artifacts.push([resolve(root,'crates/zebra-mermaid/tests/upstream/class_parser_generated.rs'),formatRust(rust)]);
const portPath=resolve(root,'tests/upstream/ports.json'),ports=JSON.parse(readFileSync(portPath,'utf8'));
// These three async configuration cases have individually reviewed native Rust ports.
const manualIndices=new Set([4,5,6]);
const owned=new Set(originals.filter(t=>!manualIndices.has(t.index)).map(t=>t.id));const previousOwned=ports.ports.filter(p=>owned.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of records)ports.ports.push({upstreamId:r.id,sourceSha256:fileHash,target:{kind:'rust',binary:'lib',file:'crates/zebra-mermaid/tests/upstream/class_parser_generated.rs',test:`diagrams::class::upstream_parser::case_${r.index.toString().padStart(3,'0')}`,features:['class']},assertionReview:'Mechanical translation of the original callback: parser calls, data reads, all matchers, callback-local assertions inside toThrow and resets are retained in IR. Model observations remain symbolic during generation; no reference/target model output is used as an expected value. Unsupported control flow is rejected. Rust executes the real raw class parser and projects field names through the same adapter contract. Spies record actual native model method calls, retaining omitted/present arguments and original call assertions; Map own keys are projected as JavaScript enumerable keys.'});
if(process.argv.includes('--check')){
 for(const [path,text]of artifacts)if(readFileSync(path,'utf8')!==text)throw Error(`Generated artifact differs from original assertions: ${path}`);
 const sort=list=>[...list].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
 if(JSON.stringify(sort(previousOwned))!==JSON.stringify(sort(ports.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Generated test mappings differ');
}else{
 for(const [path,text]of artifacts)writeFileSync(path,text);
 writeFileSync(portPath,JSON.stringify(ports,null,2)+'\n');
}
console.log(JSON.stringify({registered:registered.length,translated:records.length,untranslated}));
