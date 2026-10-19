// Validate the translated action stream against Pele's raw parser/DB before running Rust.
import {readFileSync,mkdirSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('../..',import.meta.url)),up=resolve(root,'.research/pele');
const require=createRequire(resolve(up,'package.json')),esbuild=require('esbuild');
const dir=resolve(root,'target/test-parity');mkdirSync(dir,{recursive:true});const out=resolve(dir,'flow-parser-reference.mjs');
await esbuild.build({stdin:{contents:`export {FlowDb} from './src/diagrams/flowchart/db.ts';export {parseFlowchart} from './src/diagrams/flowchart/parser.ts'; export {cleanupComments} from './src/preprocess.ts'; export {buildFlowGraph} from './src/diagrams/flowchart/graph.ts';`,resolveDir:up,sourcefile:'flow-parser-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:out,logLevel:'silent'});
const {FlowDb,parseFlowchart,cleanupComments,buildFlowGraph}=await import(pathToFileURL(out));
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/flow-parser.json'),'utf8'));
function evaluate(e,ctx,item){switch(e.op){
 case'item':return item;case'equal':return evaluate(e.left,ctx,item)===evaluate(e.right,ctx,item);case'and':return evaluate(e.left,ctx,item)&&evaluate(e.right,ctx,item);case'or':return evaluate(e.left,ctx,item)||evaluate(e.right,ctx,item);case'find':return evaluate(e.base,ctx,item).find(v=>evaluate(e.predicate,ctx,v));
 case'undefined':return undefined;case'literal':return e.value;
 case'cleanupComments':return cleanupComments(evaluate(e.source,ctx,item));
 case'root':if(e.key.startsWith('calls:'))return ctx.spies[e.key.slice(6)].calls;if(e.key==='data')return {...buildFlowGraph(ctx.db),other:{},config:{securityLevel:'strict'}};return e.key==='accTitle'?ctx.db.accTitle??'':e.key==='accDescr'?ctx.db.accDescr??'':ctx.db[e.key];
 case'array':return e.items.map(x=>evaluate(x,ctx,item));case'object':return Object.fromEntries(e.entries.map(([k,v])=>[k,evaluate(v,ctx,item)]));
 case'map':return new Map(e.entries.map(([k,v])=>[evaluate(k,ctx,item),evaluate(v,ctx,item)]));
 case'property':return e.key==='displayDetails'?evaluate(e.base,ctx,item).getDisplayDetails():evaluate(e.base,ctx,item)[e.key];case'get':return evaluate(e.base,ctx,item).get(evaluate(e.key,ctx,item));case'has':return evaluate(e.base,ctx,item).has(evaluate(e.key,ctx,item));
 default:throw new Error(`Unsupported reference expression ${e.op}`);
}}
function execute(actions,ctx){for(const a of actions){switch(a.op){
 case'reset':ctx.spies={};ctx.db=new FlowDb({maxEdges:500,warn:(...args)=>{if(ctx.spies.warn?.enabled)ctx.spies.warn.calls.push(args)}});break;
 case'observe':{const name=a.method;if(a.enabled){const spy=ctx.spies[name]={calls:[],enabled:true};if(name!=='warn'){const prior=ctx.db[name];assert.equal(typeof prior,'function');spy.restore=()=>ctx.db[name]=prior;ctx.db[name]=function(...args){spy.calls.push(args);return Reflect.apply(prior,this,args)}}}else{ctx.spies[name].enabled=false;ctx.spies[name].restore?.()}break;}
 case'callAssert':{const calls=evaluate({op:'root',key:a.method},ctx);const ok=calls.some(call=>call.length===a.args.length&&call.every((v,i)=>a.args[i].op==='stringContaining'?typeof v==='string'&&v.includes(a.args[i].value):Object.is(v,evaluate(a.args[i],ctx))));assert.notEqual(ok,a.negated,'observed call arguments');break;}case'parse':parseFlowchart(a.sourceExpr?evaluate(a.sourceExpr,ctx):a.source,ctx.db);break;case'raise':throw new ReferenceError(a.message);
 case'throws':{const pattern=a.pattern?new RegExp(a.pattern.source,a.pattern.flags):a.message?e=>e.message.includes(a.message):undefined;if(a.negated)assert.doesNotThrow(()=>execute(a.actions,ctx),pattern);else assert.throws(()=>execute(a.actions,ctx),pattern);break;}
 case'assert':{const actual=evaluate(a.actual,ctx),expected=evaluate(a.expected,ctx);let passed;
  switch(a.matcher){case'toBe':passed=Object.is(actual,expected);break;case'toEqual':try{assert.deepStrictEqual(actual,expected);passed=true;}catch{passed=false;}break;case'toContain':passed=actual.includes(expected);break;case'toHaveLength':passed=actual.length===expected;break;case'toBeUndefined':passed=actual===undefined;break;case'toBeDefined':passed=actual!==undefined;break;case'toBeGreaterThan':passed=actual>expected;break;case'toBeLessThan':passed=actual<expected;break;case'toBeGreaterThanOrEqual':passed=actual>=expected;break;case'toBeLessThanOrEqual':passed=actual<=expected;break;default:throw Error(`Unsupported matcher ${a.matcher}`);}
  assert.notEqual(passed,a.negated,`${a.matcher}: ${JSON.stringify(actual)} vs ${JSON.stringify(expected)}`);break;}
 default:throw Error(`Unsupported action ${a.op}`);
}}}
let count=0;const skippedFailures=[];
for(const record of data.records){try{execute(record.actions,{db:new FlowDb({maxEdges:500}),spies:{}});count++;}catch(e){
 if(record.mode==='skip')skippedFailures.push({id:record.id,name:record.name,error:e.message});
 else throw Error(`Translated reference test failed: ${record.name}: ${e.message}`);
}}
console.log(JSON.stringify({passed:count,originalSkippedFailures:skippedFailures},null,2));
