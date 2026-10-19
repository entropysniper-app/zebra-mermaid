// Validate the translated action stream against Pele's raw parser/DB before running Rust.
import {readFileSync,mkdirSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('../..',import.meta.url)),up=resolve(root,'.research/pele');
const require=createRequire(resolve(up,'package.json')),esbuild=require('esbuild');
const dir=resolve(root,'target/test-parity');mkdirSync(dir,{recursive:true});const out=resolve(dir,'class-parser-reference.mjs');
await esbuild.build({stdin:{contents:`export {ClassDb} from './src/diagrams/class/db.ts';export {parseClassDiagram} from './src/diagrams/class/parser.ts';`,resolveDir:up,sourcefile:'class-parser-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:out,logLevel:'silent'});
const {ClassDb,parseClassDiagram}=await import(pathToFileURL(out));
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/class-parser.json'),'utf8'));
function evaluate(e,ctx){switch(e.op){
 case'undefined':return undefined;case'literal':return e.value;
 case'root':if(e.key.startsWith('calls:'))return ctx.calls[e.key.slice(6)];return e.key==='accTitle'?ctx.db.accTitle??'':e.key==='accDescr'?ctx.db.accDescr??'':ctx.db[e.key];
 case'mapOwnKeys':return Object.keys(evaluate(e.base,ctx));
 case'array':return e.items.map(x=>evaluate(x,ctx));case'object':return Object.fromEntries(e.entries.map(([k,v])=>[k,evaluate(v,ctx)]));
 case'map':return new Map(e.entries.map(([k,v])=>[evaluate(k,ctx),evaluate(v,ctx)]));
 case'property':return e.key==='displayDetails'?evaluate(e.base,ctx).getDisplayDetails():evaluate(e.base,ctx)[e.key];case'get':return evaluate(e.base,ctx).get(evaluate(e.key,ctx));case'has':return evaluate(e.base,ctx).has(evaluate(e.key,ctx));
 default:throw new Error(`Unsupported reference expression ${e.op}`);
}}
function execute(actions,ctx){for(const a of actions){switch(a.op){
 case'reset':ctx.db=new ClassDb();ctx.calls={};break;
 case'observe':{assert.ok(a.enabled);const original=ctx.db[a.method];assert.equal(typeof original,'function');ctx.calls[a.method]=[];ctx.db[a.method]=function(...args){ctx.calls[a.method].push(args);return original.apply(this,args)};break;}
 case'callAssert':{const expected=a.args.map(e=>evaluate(e,ctx));const matched=ctx.calls[a.method.slice(6)].some(args=>{try{assert.deepStrictEqual(args,expected);return true}catch{return false}});assert.notEqual(matched,a.negated);break;}case'parse':parseClassDiagram(a.source,ctx.db);break;case'raise':throw new ReferenceError(a.message);
 case'throws':{const pattern=a.pattern?new RegExp(a.pattern.source,a.pattern.flags):undefined;if(a.negated)assert.doesNotThrow(()=>execute(a.actions,ctx),pattern);else assert.throws(()=>execute(a.actions,ctx),pattern);break;}
 case'assert':{const actual=evaluate(a.actual,ctx),expected=evaluate(a.expected,ctx);let passed;
  switch(a.matcher){case'toBe':passed=Object.is(actual,expected);break;case'toEqual':try{assert.deepStrictEqual(actual,expected);passed=true;}catch{passed=false;}break;case'toContain':passed=actual.includes(expected);break;case'toHaveLength':passed=actual.length===expected;break;case'toBeUndefined':passed=actual===undefined;break;case'toBeDefined':passed=actual!==undefined;break;case'toBeGreaterThan':passed=actual>expected;break;case'toBeLessThan':passed=actual<expected;break;case'toBeGreaterThanOrEqual':passed=actual>=expected;break;case'toBeLessThanOrEqual':passed=actual<=expected;break;default:throw Error(`Unsupported matcher ${a.matcher}`);}
  assert.notEqual(passed,a.negated,`${a.matcher}: ${JSON.stringify(actual)} vs ${JSON.stringify(expected)}`);break;}
 default:throw Error(`Unsupported action ${a.op}`);
}}}
let count=0;const skippedFailures=[];
for(const record of data.records){try{execute(record.actions,{db:new ClassDb()});count++;}catch(e){
 if(record.mode==='skip')skippedFailures.push({id:record.id,name:record.name,error:e.message});
 else throw Error(`Translated reference test failed: ${record.name}: ${e.message}`);
}}
console.log(JSON.stringify({passed:count,originalSkippedFailures:skippedFailures},null,2));
