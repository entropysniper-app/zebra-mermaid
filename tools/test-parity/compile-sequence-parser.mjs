#!/usr/bin/env node
// Preserve async test order and independent Diagram instances; expectations are never generated from output.
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {format} from 'node:util';
import vm from 'node:vm';
const root=fileURLToPath(new URL('../..',import.meta.url));
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const file='tests/compat/sequence/sequenceDiagram.spec.js',source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json'),'utf8')),originals=inventory.tests.filter(t=>t.file===file),sourceSha256=inventory.files.find(f=>f.path===file).sha256;
if(createHash('sha256').update(source).digest('hex')!==sourceSha256)throw Error('Source hash changed');
class Unsupported extends Error{}
const expressions=new WeakMap(),expressionEpoch=new WeakMap();let actions=[],nextId=0,nextView=0,nextResult=0,viewEpoch=0,config={},siteConfig={};const clone=x=>JSON.parse(JSON.stringify(x));
function literal(v){if(typeof v==='number'&&(!Number.isFinite(v)||Object.is(v,-0)))throw new Unsupported('non-finite or negative-zero literal requires typed encoding');if(v===undefined)return{op:'undefined'};if(v!==null&&(typeof v==='object'||typeof v==='function')&&expressions.has(v)){const e=expressions.get(v);if(e.op==='property'&&expressionEpoch.get(v)!==viewEpoch)throw new Unsupported('scalar observation reused after view mutation');return e;}if(Array.isArray(v))return{op:'array',items:v.map(literal)};if(v!==null&&typeof v==='object')return{op:'object',entries:Object.entries(v).map(([k,v])=>[k,literal(v)])};if(v!==null&&!['number','string','boolean'].includes(typeof v))throw new Unsupported('nonliteral');return{op:'literal',value:v}}
function sym(e){const p=new Proxy(function(){},{get(_,k){if(k==='then')return undefined;if(k===Symbol.iterator||k===Symbol.toPrimitive)return()=>{throw new Unsupported('implicit observation conversion')};if(typeof k==='symbol')throw new Unsupported('symbol property');if(k==='find'||k==='filter')return fn=>sym({op:k,base:e,predicate:predicate(fn)});if(k==='forEach')return fn=>{inspect(fn);const outer=actions;actions=[];try{fn(sym({op:'item'}));const nested=actions;if(!nested.length||nested.some(a=>a.op!=='assert'))throw new Unsupported('forEach must retain assertion-only body');outer.push({op:'forEach',base:e,actions:nested})}finally{actions=outer}};if(k==='keys')return()=>sym({op:'keys',base:e});if(k==='get'||k==='has')return key=>sym({op:k,base:e,key:literal(key)});return sym({op:'property',base:e,key:k})},set(_,key,value){if(e.op!=='get'||e.base.op!=='root'||!e.base.key.startsWith('view:')||key!=='description'||typeof value!=='string'||e.key.op!=='literal'||typeof e.key.value!=='string')throw new Unsupported('unmapped view mutation');actions.push({op:'invoke',method:'setViewDescription',view:Number(e.base.key.slice(5)),actor:e.key,value});viewEpoch++;return true},apply(){throw new Unsupported('unmapped observation method')},ownKeys(){throw new Unsupported('model enumeration')}});expressions.set(p,e);expressionEpoch.set(p,viewEpoch);return p}
function predicate(fn){
 const ast=ts.createSourceFile('predicate.ts',`(${fn.toString()})`,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const fun=ast.statements[0].expression.expression;
 if(!fun.parameters||fun.parameters.length!==1||!ts.isIdentifier(fun.parameters[0].name)||ts.isBlock(fun.body))throw new Unsupported('predicate form');
 const name=fun.parameters[0].name.text;
 const expression=n=>{
  if(ts.isIdentifier(n)&&n.text===name)return{op:'item'};
  if(ts.isPropertyAccessExpression(n))return{op:'property',base:expression(n.expression),key:n.name.text};
  if(ts.isIdentifier(n)&&n.text==='undefined')return{op:'undefined'};
  if(ts.isNumericLiteral(n))return{op:'literal',value:Number(n.text)};
  if(ts.isStringLiteral(n))return{op:'literal',value:n.text};
  if(n.kind===ts.SyntaxKind.TrueKeyword||n.kind===ts.SyntaxKind.FalseKeyword)return{op:'literal',value:n.kind===ts.SyntaxKind.TrueKeyword};
  if(ts.isBinaryExpression(n)){
   const op=n.operatorToken.kind===ts.SyntaxKind.ExclamationEqualsEqualsToken?'notEqual':n.operatorToken.kind===ts.SyntaxKind.EqualsEqualsEqualsToken?'equal':n.operatorToken.kind===ts.SyntaxKind.AmpersandAmpersandToken?'and':n.operatorToken.kind===ts.SyntaxKind.BarBarToken?'or':null;
   if(op)return{op,left:expression(n.left),right:expression(n.right)};
  }
  throw new Unsupported('predicate expression '+ts.SyntaxKind[n.kind]);
 };
 return expression(fun.body);
}

function inspect(fn){const a=ts.createSourceFile('callback.js',`(${fn.toString()})`,ts.ScriptTarget.Latest,true);const forbidden=new Set([ts.SyntaxKind.IfStatement,ts.SyntaxKind.ConditionalExpression,ts.SyntaxKind.TryStatement,ts.SyntaxKind.ThrowStatement,ts.SyntaxKind.SwitchStatement,ts.SyntaxKind.WhileStatement,ts.SyntaxKind.ForStatement,ts.SyntaxKind.ForOfStatement,ts.SyntaxKind.ForInStatement,ts.SyntaxKind.TypeOfExpression,ts.SyntaxKind.PrefixUnaryExpression]);const visit=n=>{if(ts.isCallExpression(n)&&ts.isPropertyAccessExpression(n.expression)&&['find','filter'].includes(n.expression.name.text)){visit(n.expression);return;}if(forbidden.has(n.kind)||ts.isBinaryExpression(n)&&!(n.operatorToken.kind===ts.SyntaxKind.PlusToken||n.operatorToken.kind===ts.SyntaxKind.EqualsToken&&(ts.isIdentifier(n.left)||ts.isPropertyAccessExpression(n.left)&&n.left.name.text==='description')))throw new Unsupported('control flow/operator '+ts.SyntaxKind[n.kind]);ts.forEachChild(n,visit)};visit(a)}
function snapshot(text){const a=ts.createSourceFile('snapshot.js','('+text+')',ts.ScriptTarget.Latest,true);const walk=n=>{if(ts.isParenthesizedExpression(n))return walk(n.expression);if(ts.isStringLiteral(n))return n.text;if(ts.isNumericLiteral(n))return Number(n.text);if(n.kind===ts.SyntaxKind.TrueKeyword)return true;if(n.kind===ts.SyntaxKind.FalseKeyword)return false;if(n.kind===ts.SyntaxKind.NullKeyword)return null;if(ts.isIdentifier(n)&&n.text==='undefined')return undefined;if(ts.isArrayLiteralExpression(n))return n.elements.map(walk);if(ts.isObjectLiteralExpression(n))return Object.fromEntries(n.properties.map(p=>{if(!ts.isPropertyAssignment(p))throw new Unsupported('snapshot property');return[p.name.text,walk(p.initializer)]}));throw new Unsupported('snapshot syntax')};return walk(a.statements[0].expression)}
const dependent=e=>e.op==='root'||e.op==='item'||Object.values(e).some(v=>v&&typeof v==='object'&&(Array.isArray(v)?v.some(x=>x&&typeof x==='object'&&dependent(x)):dependent(v)));
function expect(actual){const mk=negated=>new Proxy({}, {get(_,matcher){if(matcher==='not')return mk(!negated);if(matcher==='resolves'){const expr=literal(actual);if(expr.op!=='root'||!expr.key.startsWith('result:'))throw new Unsupported('resolves requires actual public parser Promise');return mk(negated)}if(!['toBe','toEqual','toContain','toBeDefined','toBeUndefined','toHaveLength','toMatchObject','toBeTruthy','toBeFalsy','toMatchInlineSnapshot'].includes(matcher))throw new Unsupported('matcher '+matcher);return expected=>{const expr=literal(actual);if(!dependent(expr))throw new Unsupported('non-model assertion');if(matcher==='toMatchInlineSnapshot'){expected=snapshot(expected);matcher='toEqual'}actions.push({op:'assert',matcher,negated,actual:expr,expected:literal(expected)})}}});return mk(false)}
const constantsSource=readFileSync(resolve(root,'tests/upstream/source/tests/support/sequence-constants.ts'),'utf8');const constants=vm.runInNewContext(ts.transpileModule(constantsSource.replaceAll('export const ','const ')+'\n({LINETYPE,ARROWTYPE,PLACEMENT})',{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText);
class SequenceDB{
 constructor(){this.id=nextId++;Object.assign(this,constants);actions.push({op:'invoke',method:'create',id:this.id,config:clone(config)});return new Proxy(this,{get(t,k,r){if(Reflect.has(t,k))return Reflect.get(t,k,r);throw new Unsupported('SequenceDB method '+String(k))}})}
 root(key){return sym({op:'root',key:`${this.id}:${key}`})}
 getActors(){const view=nextView++;actions.push({op:'invoke',method:'actorView',id:this.id,view});return sym({op:'root',key:'view:'+view})}getActor(id){return this.getActors().get(id)}getActorKeys(){return this.root('actorKeys')}
 getMessages(){return this.root('messages')}getBoxes(){return this.root('boxes')}
 getCreatedActors(){return this.root('createdActors')}getDestroyedActors(){return this.root('destroyedActors')}
 getDiagramTitle(){return this.root('title')}getAccTitle(){return this.root('accTitle')}getAccDescription(){return this.root('accDescr')}
 getConfig(){return clone(config.sequence)}
 showSequenceNumbers(){return this.root('numbersShown')}
}
const Diagram={
 fromText(source){
  if(typeof source!=='string')throw new Unsupported('nonliteral diagram source');
  const db=new SequenceDB();
  actions.push({op:'invoke',method:'parseDiagram',id:db.id,source});
  return {db,type:'sequence',renderer:{
   draw(_text,_id,_version,diagram){actions.push({op:'invoke',method:'render',id:diagram.db.id,config:clone(config)})},
   bounds:new Proxy({init(){}},{get(t,k){if(k==='init')return t.init;throw new Unsupported('renderer bounds '+String(k)+' requires host adapter')},set(){throw new Unsupported('renderer bounds mutation requires host adapter')}})
  }};
 }
};
function setSiteConfig(next){siteConfig=clone(next);config=clone(next)}
const mermaidAPI={initialize(next){setSiteConfig({...siteConfig,...next})},reset(){config=clone(siteConfig)},getConfig(){return clone(config)},parse(source){if(typeof source!=='string')throw new Unsupported('nonliteral public parse');const result=nextResult++;actions.push({op:'invoke',method:'parsePublic',source,config:clone(config),result});return sym({op:'root',key:'result:'+result})}};
const top={name:'',all:[],each:[],ran:false},stack=[top],registered=[];
function describe(name,fn){const suite={name,all:[],each:[],ran:false};stack.push(suite);fn();stack.pop()}
function register(mode,name,fn,timeout=60000){registered.push({name,timeout,fullName:[...stack.slice(1).map(s=>s.name),name].join(' > '),mode,fn,suites:[...stack]})}
function it(name,fn,timeout){register('run',name,fn,timeout)}it.skip=(name,fn,timeout)=>register('skip',name,fn,timeout);it.each=rows=>(name,fn,timeout)=>{for(const row of rows){const args=Array.isArray(row)?row:[row];register('run',format(name,...args.slice(0,(name.match(/%s/g)??[]).length)),async()=>{inspect(fn);await fn(...args)},timeout)}};
const a=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true);let code=source;for(const s of [...a.statements].reverse())if(ts.isImportDeclaration(s))code=code.slice(0,s.getStart(a))+code.slice(s.end);
const transformed=ts.transform(ts.createSourceFile(file,code,ts.ScriptTarget.Latest,true),[ctx=>{
 const visit=n=>{
  if(ts.isBlock(n)){
   const statements=[];
   for(let i=0;i<n.statements.length;i++){
    const statement=n.statements[i],attempt=n.statements[i+1];
    if(ts.isVariableStatement(statement)&&attempt&&ts.isTryStatement(attempt)){
     const declarations=statement.declarationList.declarations,decl=declarations[0];
     const caught=attempt.catchClause;
     if(declarations.length===1&&ts.isIdentifier(decl.name)&&caught&&!attempt.finallyBlock&&caught.block.statements.length===1){
      const catchStatement=caught.block.statements[0],assign=ts.isExpressionStatement(catchStatement)?catchStatement.expression:null;
      if(assign&&ts.isBinaryExpression(assign)&&assign.operatorToken.kind===ts.SyntaxKind.EqualsToken&&ts.isIdentifier(assign.left)&&assign.left.text===decl.name.text){
       const init=decl.initializer,catchText=assign.right.getText().replace(/\s/g,'');
       const kind=init?.kind===ts.SyntaxKind.FalseKeyword&&assign.right.kind===ts.SyntaxKind.TrueKeyword?'boolean':ts.isStringLiteral(init)&&init.text===''&&caught.variableDeclaration?.name.getText()==='e'&&catchText==='einstanceofError?e.message:String(e)'?'message':null;
       if(kind){
        const body=ts.visitNode(attempt.tryBlock,visit);
        const callback=ts.factory.createArrowFunction([ts.factory.createModifier(ts.SyntaxKind.AsyncKeyword)],undefined,[],undefined,ts.factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),body);
        const call=ts.factory.createAwaitExpression(ts.factory.createCallExpression(ts.factory.createIdentifier('__captureFailure'),undefined,[callback,ts.factory.createStringLiteral(kind)]));
        statements.push(ts.factory.updateVariableStatement(statement,statement.modifiers,ts.factory.updateVariableDeclarationList(statement.declarationList,[ts.factory.updateVariableDeclaration(decl,decl.name,decl.exclamationToken,decl.type,call)])));
        i++;continue;
       }
      }
     }
    }
    statements.push(ts.visitNode(statement,visit));
   }
   return ts.factory.updateBlock(n,statements);
  }
  if(ts.isArrayLiteralExpression(n)&&n.elements.length===1&&ts.isSpreadElement(n.elements[0]))return ts.factory.createCallExpression(ts.factory.createIdentifier('__arrayFrom'),undefined,[ts.visitNode(n.elements[0].expression,visit)]);return ts.visitEachChild(n,visit,ctx)};return sf=>ts.visitNode(sf,visit);
}]);code=ts.createPrinter().printFile(transformed.transformed[0]);transformed.dispose();
async function captureFailure(fn,kind){
 inspect(fn);const outer=actions;actions=[];
 try{await fn();const nested=actions,result=nextResult++;outer.push({op:'invoke',method:'captureFailure',kind,result,actions:nested});return sym({op:'root',key:'result:'+result})}finally{actions=outer}
}
vm.runInNewContext(code,{__captureFailure:captureFailure,__arrayFrom:v=>expressions.has(v)?sym({op:'arrayFrom',base:literal(v)}):Array.from(v),describe,it,expect,SequenceDB,Diagram,mermaidAPI,setSiteConfig,addDiagrams(){},beforeAll:fn=>stack.at(-1).all.push(fn),beforeEach:fn=>stack.at(-1).each.push(fn),vi:{mock(name){if(name!=='d3')throw new Unsupported('module mock')},fn(){throw new Unsupported('mock function')}}},{filename:file});
if(registered.length!==originals.length)throw Error('Registration count changed');const records=[],untranslated=[];
for(let i=0;i<registered.length;i++){
 const t=registered[i],o=originals[i];if(t.fullName!==o.fullName||t.mode!==o.mode||t.timeout!==o.timeoutMs)throw Error('Identity changed at '+i);
 try{for(const s of t.suites)if(!s.ran){s.ran=true;for(const fn of s.all)await fn()}actions=[];nextId=0;nextView=0;nextResult=0;viewEpoch=0;for(const s of t.suites)for(const fn of s.each)await fn();const setupActions=actions;actions=[];inspect(t.fn);await t.fn();if(!actions.some(a=>['assert','forEach'].includes(a.op)||a.op==='invoke'&&a.method==='parseDiagram'))throw new Unsupported('no executed assertion or parse in original test body');records.push({id:o.id,index:i,name:t.fullName,mode:t.mode,timeout:t.timeout,setupActions,actions});}
 catch(e){if(!(e instanceof Unsupported))throw e;untranslated.push({id:o.id,index:i,reason:e.message})}
}
let rust='// Generated from frozen sequence assertions (MIT). One test per original callback.\n';for(const r of records)rust+=`#[test]\n${r.mode==='skip'?'#[ignore = "Original upstream skip; not passed"]\n':''}fn case_${String(r.index).padStart(3,'0')}() { run_case("${r.id}"); }\n`;
const artifacts=[['tests/upstream/ir/sequence-parser.json',JSON.stringify({source:file,sourceSha256,records,untranslated},null,2)+'\n'],['crates/zebra-mermaid/tests/upstream/sequence_parser_generated.rs',rust]];
const path='tests/upstream/ports.json',ports=JSON.parse(readFileSync(resolve(root,path),'utf8')),owned=new Set(originals.map(t=>t.id)),previous=ports.ports.filter(p=>owned.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!owned.has(p.upstreamId));
for(const r of records)ports.ports.push({upstreamId:r.id,sourceSha256,target:{kind:'rust',binary:'lib',file:'crates/zebra-mermaid/tests/upstream/sequence_parser_generated.rs',test:`diagrams::sequence::upstream_parser::case_${String(r.index).padStart(3,'0')}`,features:['sequence']},assertionReview:'Original async callback and parameter order retained. Each Diagram receives a distinct native model identity, including multiple diagrams in one test. Original configuration at construction, entity encoding, trailing newline, model getters, primitive/deep/partial/inline-snapshot assertions are preserved. Snapshot syntax is parsed as literal data only. No renderer output generates expectations; filter/forEach predicates, actual native rendering and numbering, public preprocess/parse Promise results, exact boolean/message try-catch paths, and original test timeouts are retained. Renderer bounds without an upstream counterpart remain unmapped.'});
const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const[p,s]of artifacts)if(readFileSync(resolve(root,p),'utf8')!==s)throw Error('Generated artifact differs');if(JSON.stringify(sort(previous))!==JSON.stringify(sort(ports.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Mapping differs')}
else{for(const[p,s]of artifacts)writeFileSync(resolve(root,p),s);writeFileSync(resolve(root,path),JSON.stringify(ports,null,2)+'\n')}
console.log(JSON.stringify({registered:originals.length,translated:records.length,untranslated}));
