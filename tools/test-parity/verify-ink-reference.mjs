// Re-run the translated assertion stream against the independent pinned TypeScript code.
import {readFileSync,mkdirSync} from 'node:fs';
import {createRequire} from 'node:module';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const root=resolve(import.meta.dirname,'../..'),up=resolve(root,'.research/pele');
const require=createRequire(resolve(up,'package.json'));
const output=resolve(root,'target/test-parity/ink-reference.mjs');
mkdirSync(resolve(root,'target/test-parity'),{recursive:true});
await require('esbuild').build({stdin:{contents:"export {resolveStyle} from './src/svg/theme.ts'; export {ink} from './src/svg/ink.ts';",resolveDir:up,sourcefile:'ink-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:output,logLevel:'silent'});
const {resolveStyle,ink}=await import(pathToFileURL(output));
const {records}=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/ink.json')));
let calls=0,assertions=0;
for(const record of records){
 const slots=new Map();
 const value=x=>x&&typeof x==='object'&&!Array.isArray(x)?('$ink' in x?slots.get(x.$ink):x.$undefined?undefined:x):x;
 for(const action of record.actions){
  if(action.op==='ink'){
   const style=resolveStyle(action.declarations);slots.set(action.slot,ink(style.shape,style.text));calls++;
  }else if(action.op==='equal'){
   assert.equal(value(action.actual),value(action.expected),record.name);assertions++;
  }else if(action.op==='length'){
   assert.equal(value(action.actual).length,value(action.expected),record.name);assertions++;
  }else throw Error('Unreviewed ink action '+action.op);
 }
}
console.log(JSON.stringify({passed:records.length,calls,assertions}));
