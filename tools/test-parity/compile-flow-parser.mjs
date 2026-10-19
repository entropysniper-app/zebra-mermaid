#!/usr/bin/env node
// Mechanical translation of flowchart parser calls AND assertions. No Pele/Rust outputs are harvested.
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
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8'));
const files=['flow-text.spec.js','flow-edges.spec.js','flow-singlenode.spec.js','flow-arrows.spec.js','flow-comments.spec.js','flow-direction.spec.js','flow-interactions.spec.js','flow-lines.spec.js','flow-md-string.spec.js','flow-node-data.spec.js','flow-style.spec.js','flow-vertice-chaining.spec.js','flow.spec.js','subgraph.spec.js'].map(f=>'tests/compat/flowchart/'+f);
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
   if(expr.op==='root'&&expr.key==='edges'&&['defaultStyle','defaultInterpolate'].includes(key))return sym({op:'root',key:key==='defaultStyle'?'defaultEdgeStyle':'defaultInterpolate'});
   if(key==='mockRestore'&&expr.op==='root'&&expr.key.startsWith('calls:'))return()=>actions.push({op:'observe',method:expr.key.slice(6),enabled:false});
   if(key==='find')return fn=>sym({op:'find',base:expr,predicate:predicate(fn)});
   if(key==='get'||key==='has')return arg=>sym({op:key,base:expr,key:literal(arg)});
   return sym({op:'property',base:expr,key});
  },
  apply(){throw new Unsupported('unsupported model method')},
  ownKeys(){throw new Unsupported('model enumeration requires an explicit adapter')},
 });
 expressions.set(proxy,expr);return proxy;
}
function predicate(fn){
 const ast=ts.createSourceFile('predicate.ts',`(${fn.toString()})`,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const fun=ast.statements[0].expression.expression;
 if(!fun.parameters||fun.parameters.length!==1||!ts.isIdentifier(fun.parameters[0].name)||ts.isBlock(fun.body))throw new Unsupported('predicate form');
 const name=fun.parameters[0].name.text;
 const expression=n=>{
  if(ts.isIdentifier(n)&&n.text===name)return{op:'item'};
  if(ts.isPropertyAccessExpression(n))return{op:'property',base:expression(n.expression),key:n.name.text};
  if(ts.isStringLiteral(n))return{op:'literal',value:n.text};
  if(n.kind===ts.SyntaxKind.TrueKeyword||n.kind===ts.SyntaxKind.FalseKeyword)return{op:'literal',value:n.kind===ts.SyntaxKind.TrueKeyword};
  if(ts.isBinaryExpression(n)){
   const op=n.operatorToken.kind===ts.SyntaxKind.EqualsEqualsEqualsToken?'equal':n.operatorToken.kind===ts.SyntaxKind.AmpersandAmpersandToken?'and':n.operatorToken.kind===ts.SyntaxKind.BarBarToken?'or':null;
   if(op)return{op,left:expression(n.left),right:expression(n.right)};
  }
  throw new Unsupported('predicate expression '+ts.SyntaxKind[n.kind]);
 };
 return expression(fun.body);
}
function inspect(fn){
 const ast=ts.createSourceFile('test.js',`(${fn.toString()})`,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const forbidden=new Set([ts.SyntaxKind.AsyncKeyword,ts.SyntaxKind.AwaitExpression,ts.SyntaxKind.IfStatement,ts.SyntaxKind.ConditionalExpression,ts.SyntaxKind.SwitchStatement,ts.SyntaxKind.WhileStatement,ts.SyntaxKind.ForStatement,ts.SyntaxKind.ForInStatement,ts.SyntaxKind.ForOfStatement,ts.SyntaxKind.TypeOfExpression,ts.SyntaxKind.BinaryExpression,ts.SyntaxKind.PrefixUnaryExpression]);
 // Branches cannot be evaluated by JS while observations are symbolic. Fail closed.
 const visit=n=>{if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='find'){visit(n.expression);return;}if(forbidden.has(n.kind)&&!(ts.isBinaryExpression(n)&&(n.operatorToken.kind===ts.SyntaxKind.PlusToken||n.operatorToken.kind===ts.SyntaxKind.EqualsToken&&ts.isIdentifier(n.left))))throw new Unsupported(`control flow/operator needs manual translation: ${ts.SyntaxKind[n.kind]}`);ts.forEachChild(n,visit);};visit(ast);
}
function nested(fn){inspect(fn);const parent=actions;actions=[];const result=fn();if(result&&typeof result.then==='function')throw new Unsupported('async test body');const out=actions;actions=parent;return out;}
const dependent=e=>e.op==='root'||Object.values(e).some(v=>v&&typeof v==='object'&&(Array.isArray(v)?v.some(x=>x&&typeof x==='object'&&dependent(x)):dependent(v)));
function expect(actual){
 const make=negated=>new Proxy({}, {get(_,name){if(name==='not')return make(!negated);return (...args)=>{const expected=args[0];
  if(name==='toHaveBeenCalledWith'){const e=literal(actual);if(e.op!=='root'||!e.key.startsWith('calls:'))throw new Unsupported('not an observed method');actions.push({op:'callAssert',method:e.key,args:args.map(literal),negated});return;}
  if(name==='toThrow'||name==='toThrowError'){
   if(typeof actual!=='function'||expressions.has(actual))throw new Unsupported('unsupported throw matcher');
   const pattern=expected===undefined?null:Object.prototype.toString.call(expected)==='[object RegExp]'?{source:expected.source,flags:expected.flags}:null;
   if(expected!==undefined&&typeof expected!=='string'&&(!pattern||pattern.source!=="(Expecting\\s'STR'|Unrecognized\\stext)"||pattern.flags!==''))throw new Unsupported('unmapped throw pattern');
   actions.push({op:'throws',negated,actions:nested(actual),...(pattern?{pattern}:typeof expected==='string'?{message:expected}:{})});return;
  }
  if(!['toBe','toEqual','toContain','toHaveLength','toBeUndefined','toBeDefined','toBeGreaterThan','toBeLessThan','toBeGreaterThanOrEqual','toBeLessThanOrEqual'].includes(name))throw new Unsupported(`matcher ${String(name)}`);
  const expr=literal(actual);if(!dependent(expr))throw new Unsupported('assertion has no symbolic model observation');
  actions.push({op:'assert',matcher:name,negated,actual:expr,expected:literal(expected)});
 }} });return make(false);
}
expect.stringContaining=value=>sym({op:'stringContaining',value});
expect.objectContaining=()=>{throw new Unsupported('asymmetric object matcher requires explicit translation')};
expect.any=()=>{throw new Unsupported('asymmetric type matcher requires explicit translation')};
class FlowDB{
 constructor(){actions.push({op:'reset'});this.spies=new Set();return new Proxy(this,{get(target,key,receiver){if(target.spies.has(key))return sym({op:'root',key:'calls:'+key});if(Reflect.has(target,key))return Reflect.get(target,key,receiver);throw new Unsupported('unmapped FlowDB operation '+String(key))}})}
 clear(){actions.push({op:'reset'})}
 setGen(){}
 getData(){return sym({op:'root',key:'data'})}
 getVertices(){return sym({op:'root',key:'nodes'})}
 getEdges(){return sym({op:'root',key:'edges'})}
 getClasses(){return sym({op:'root',key:'classes'})}
 getSubGraphs(){return sym({op:'root',key:'subgraphs'})}
 getDirection(){return sym({op:'root',key:'direction'})}
 getTooltip(id){return sym({op:'get',base:{op:'root',key:'tooltips'},key:literal(id)})}
 getAccTitle(){return sym({op:'root',key:'accTitle'})}
 getAccDescription(){return sym({op:'root',key:'accDescr'})}
}
const log={warn(){throw new Unsupported('warn direct call requires adapter')}};
function spyOn(target,key){if(target===log&&key==='warn'){}else if(target instanceof FlowDB&&['setLink','setTooltip','setClickEvent'].includes(key)){target.spies.add(key)}else throw new Unsupported('unmapped spy');actions.push({op:'observe',method:key,enabled:true});return sym({op:'root',key:'calls:'+key})}
const cleanupComments=source=>sym({op:'cleanupComments',source:literal(source)});
function setConfig(config){if(Object.keys(config).some(k=>k!=='securityLevel'))throw new Unsupported('nondefault FlowDB configuration requires explicit adapter');}
const records=[],untranslated=[];const sources=[];
for(const file of files){
 const source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
 const originals=inventory.tests.filter(t=>t.file===file),fileHash=inventory.files.find(f=>f.path===file).sha256;
 if(createHash('sha256').update(source).digest('hex')!==fileHash)throw Error('Source hash changed');
 sources.push({file,sha256:fileHash});
const parser={yy:null,parse(source){if(typeof source==='string')actions.push({op:'parse',source});else if(expressions.has(source))actions.push({op:'parse',sourceExpr:literal(source)});else throw new Unsupported('non-string source');}};
parser.parser=parser;const flow=parser;const suites=[];const hooks=[];const registered=[];
function describe(name,fn){suites.push(name);hooks.push([]);fn();hooks.pop();suites.pop();}
function register(mode,name,fn){registered.push({name,fullName:[...suites,name].join(' > '),fn,hooks:hooks.flat(),mode});}
function it(name,fn){register('run',name,fn)}it.skip=(name,fn)=>register('skip',name,fn);it.todo=name=>register('todo',name,()=>{});
it.each=rows=>(name,fn)=>{for(const row of rows){const args=Array.isArray(row)?row:[row];register('run',format(name,...args.slice(0,(name.match(/%s/g)??[]).length)),()=>{inspect(fn);return fn(...args)});}};
// Imports are removed by their parsed source spans, not a regex that might alter a string.
const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
let executable=source;for(const statement of [...ast.statements].reverse())if(ts.isImportDeclaration(statement))executable=executable.slice(0,statement.getStart(ast))+executable.slice(statement.end);
executable=ts.transpileModule(executable,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
vm.runInNewContext(executable,{describe,it,beforeEach:fn=>hooks.at(-1).push(fn),expect,FlowDB,flow,setConfig,cleanupComments,log,vi:{spyOn}},{filename:file,timeout:10000});
if(registered.length!==originals.length)throw Error(`case count changed: ${registered.length}/${originals.length}`);
for(let i=0;i<registered.length;i++){
 const test=registered[i],original=originals[i];if(test.fullName!==original.fullName||test.mode!==original.mode)throw Error(`case identity changed at ${i}: ${test.fullName}`);
 try{actions=[];for(const hook of test.hooks)hook();actions.push(...nested(test.fn));if(!actions.some(a=>a.op==='parse'||a.op==='throws'||a.op==='assert'))throw new Unsupported('no parser call or model assertion');records.push({file,id:original.id,index:i,name:test.fullName,mode:test.mode,actions});}
 catch(e){if(!(e instanceof Unsupported))throw e;untranslated.push({file,id:original.id,index:i,reason:e.message});}
}
}
const slug=file=>file.split('/').at(-1).replace('.spec.js','').replaceAll('-','_');
const artifacts=[];
artifacts.push([resolve(root,'tests/upstream/ir/flow-parser.json'),JSON.stringify({sources,records,untranslated},null,2)+'\n']);
let rust='// Generated from original Mermaid/Pele flowchart callbacks; one function per case (MIT).\n';
for(const r of records)rust+=`#[test]\n${r.mode==='skip'?'#[ignore = "Original upstream skip; not counted as passed"]\n':''}fn ${slug(r.file)}_${String(r.index).padStart(3,'0')}() { run_case("${r.id}"); }\n`;
artifacts.push([resolve(root,'crates/zebra-mermaid/tests/upstream/flow_parser_generated.rs'),rust]);
const portPath=resolve(root,'tests/upstream/ports.json'),ports=JSON.parse(readFileSync(portPath,'utf8'));
const owned=new Set(inventory.tests.filter(t=>files.includes(t.file)).map(t=>t.id));
const previous=ports.ports.filter(p=>owned.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of records)ports.ports.push({upstreamId:r.id,sourceSha256:sources.find(s=>s.file===r.file).sha256,target:{kind:'rust',binary:'lib',file:'crates/zebra-mermaid/tests/upstream/flow_parser_generated.rs',test:`diagrams::flowchart::upstream_compat::${slug(r.file)}_${String(r.index).padStart(3,'0')}`,features:['flowchart']},assertionReview:'Original callback, every it.each row, parser call, model field observation, equality and throw matcher preserved symbolically. Registration loops and parameter rows execute unchanged; unsupported test-body control flow fails closed. The raw Rust FlowchartModel uses the adapter default maxEdges=500. No reference/target outputs generate expectations.'});
if(process.argv.includes('--check')){
 for(const[path,text]of artifacts)if(readFileSync(path,'utf8')!==text)throw Error('Generated artifact differs');
 const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
 if(JSON.stringify(sort(previous))!==JSON.stringify(sort(ports.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Mappings differ');
}else{for(const[path,text]of artifacts)writeFileSync(path,text);writeFileSync(portPath,JSON.stringify(ports,null,2)+'\n')}
console.log(JSON.stringify({registered:inventory.tests.filter(t=>files.includes(t.file)).length,translated:records.length,untranslated}));
