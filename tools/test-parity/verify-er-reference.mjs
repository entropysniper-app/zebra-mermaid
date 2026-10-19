// Validate the translated action stream against Pele's raw parser/DB before running Rust.
import {readFileSync,mkdirSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('../..',import.meta.url)),up=resolve(root,'.research/pele');
const require=createRequire(resolve(up,'package.json')),esbuild=require('esbuild');
const dir=resolve(root,'target/test-parity');mkdirSync(dir,{recursive:true});const out=resolve(dir,'er-reference.mjs');
await esbuild.build({stdin:{contents:`export {ErDb} from './src/diagrams/er/db.ts';export {parseEr} from './src/diagrams/er/parser.ts';`,resolveDir:up,sourcefile:'er-parity-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:out,logLevel:'silent'});
const {ErDb,parseEr}=await import(pathToFileURL(out));
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/er-parser.json'),'utf8'));
function evaluate(e,ctx){switch(e.op){
 case'undefined':return undefined;case'literal':return e.value;
 case'root':return e.key==='accTitle'?ctx.db.accTitle??'':e.key==='accDescr'?ctx.db.accDescr??'':ctx.db[e.key];
 case'array':return e.items.map(x=>evaluate(x,ctx));case'object':return Object.fromEntries(e.entries.map(([k,v])=>[k,evaluate(v,ctx)]));
 case'map':return new Map(e.entries.map(([k,v])=>[evaluate(k,ctx),evaluate(v,ctx)]));
 case'property':return evaluate(e.base,ctx)[e.key];case'get':return evaluate(e.base,ctx).get(evaluate(e.key,ctx));case'has':return evaluate(e.base,ctx).has(evaluate(e.key,ctx));
 default:throw new Error(`Unsupported reference expression ${e.op}`);
}}
function execute(actions,ctx){for(const a of actions){switch(a.op){
 case'reset':ctx.db=new ErDb();break;case'parse':parseEr(a.source,ctx.db);break;case'raise':throw new ReferenceError(a.message);
 case'throws':if(a.negated)assert.doesNotThrow(()=>execute(a.actions,ctx));else assert.throws(()=>execute(a.actions,ctx));break;
 case'assert':{const actual=evaluate(a.actual,ctx),expected=evaluate(a.expected,ctx);let passed;
  switch(a.matcher){case'toBe':passed=Object.is(actual,expected);break;case'toEqual':try{assert.deepStrictEqual(actual,expected);passed=true;}catch{passed=false;}break;case'toContain':passed=actual.includes(expected);break;case'toHaveLength':passed=actual.length===expected;break;case'toBeUndefined':passed=actual===undefined;break;case'toBeDefined':passed=actual!==undefined;break;case'toBeGreaterThan':passed=actual>expected;break;case'toBeLessThan':passed=actual<expected;break;case'toBeGreaterThanOrEqual':passed=actual>=expected;break;case'toBeLessThanOrEqual':passed=actual<=expected;break;default:throw Error(`Unsupported matcher ${a.matcher}`);}
  assert.notEqual(passed,a.negated,`${a.matcher}: ${JSON.stringify(actual)} vs ${JSON.stringify(expected)}`);break;}
 default:throw Error(`Unsupported action ${a.op}`);
}}}
let count=0;for(const record of data.records){try{execute(record.actions,{db:new ErDb()});count++;}catch(e){throw Error(`Translated reference test failed: ${record.name}: ${e.message}`);}}
console.log(`${count} ER action streams preserve their assertions against the original Pele parser`);
