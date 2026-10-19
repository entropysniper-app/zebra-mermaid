// Check every generated assertion against the original date/axis implementation.
import {readFileSync,mkdirSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
process.env.TZ='UTC';
const root=fileURLToPath(new URL('../..',import.meta.url)),up=resolve(root,'.research/pele');
const require=createRequire(resolve(up,'package.json'));
const out=resolve(root,'target/test-parity/gantt-dates-reference.mjs');mkdirSync(resolve(root,'target/test-parity'),{recursive:true});
await require('esbuild').build({stdin:{contents:"export {autoTicks,intervalTicks,timeFormat} from './src/diagrams/gantt/axis.ts'; export {add,format,parseStrict} from './src/diagrams/gantt/dates.ts'; export {parseDuration} from './src/diagrams/gantt/db.ts';",resolveDir:up,sourcefile:'gantt-dates-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:out,logLevel:'silent'});
const functions=await import(pathToFileURL(out));
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/gantt-dates.json'),'utf8'));
function evaluate(e){switch(e.op){
 case'undefined':return undefined;case'literal':return e.value;case'number':assert.equal(e.value,'NaN');return NaN;
 case'date':return new Date(evaluate(e.time));case'array':return e.items.map(evaluate);
 case'length':return evaluate(e.base).length;case'getTime':return evaluate(e.base).getTime();case'isNaN':return Number.isNaN(evaluate(e.base));
 default:assert.equal(typeof functions[e.op],'function');return functions[e.op](...e.args.map(evaluate));
}}
for(const r of data.records)for(const a of r.assertions){const actual=evaluate(a.actual),expected=evaluate(a.expected);switch(a.matcher){
 case'toEqual':assert.deepStrictEqual(actual,expected,r.name);break;case'toBe':assert.ok(Object.is(actual,expected),r.name);break;
 case'toBeUndefined':assert.equal(actual,undefined,r.name);break;case'toBeLessThanOrEqual':assert.ok(actual<=expected,r.name);break;
 default:throw Error('Unsupported matcher');
}}
const start=new Date(2013,0,1);
for(const[n,unit,expected]of[[2,'h',7200000],[2,'m',120000],[.005,'s',5],[20,'ms',20]])assert.equal(+functions.add(start,n,unit)-+start,expected);
console.log(`${data.records.length+1} Gantt date tests retain all assertions against the original implementation`);
