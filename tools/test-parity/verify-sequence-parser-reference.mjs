import {readFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('../..',import.meta.url)),up=resolve(root,'.research/pele'),require=createRequire(resolve(up,'package.json'));
const output=resolve(root,'target/test-parity/sequence-reference.mjs');mkdirSync(resolve(root,'target/test-parity'),{recursive:true});
await require('esbuild').build({stdin:{contents:"export {SequenceDB,Diagram,mermaidAPI,setSiteConfig} from './tests/compat/sequence/adapter.ts';",resolveDir:up,sourcefile:'sequence-reference.ts'},bundle:true,format:'esm',platform:'node',outfile:output,logLevel:'silent'});
const {SequenceDB,Diagram,mermaidAPI,setSiteConfig}=await import(pathToFileURL(output));
const data=JSON.parse(readFileSync(resolve(root,'tests/upstream/ir/sequence-parser.json'),'utf8'));
function evaluate(e,models,item){switch(e.op){
 case'item':return item;
 case'equal':return evaluate(e.left,models,item)===evaluate(e.right,models,item);
 case'notEqual':return evaluate(e.left,models,item)!==evaluate(e.right,models,item);
 case'and':return evaluate(e.left,models,item)&&evaluate(e.right,models,item);
 case'or':return evaluate(e.left,models,item)||evaluate(e.right,models,item);
 case'find':case'filter':return evaluate(e.base,models,item)[e.op](v=>evaluate(e.predicate,models,v));
 case'keys':return evaluate(e.base,models,item).keys();case'arrayFrom':return [...evaluate(e.base,models,item)];
 case'undefined':return undefined;case'literal':return e.value;case'array':return e.items.map(x=>evaluate(x,models,item));case'object':return Object.fromEntries(e.entries.map(([k,v])=>[k,evaluate(v,models,item)]));
 case'root':{
  if(e.key.startsWith('view:'))return models.views.get(Number(e.key.slice(5)));
  if(e.key.startsWith('result:'))return models.results.get(Number(e.key.slice(7)));
  const[id,key]=e.key.split(':'),db=models.get(Number(id)).db;
  if(key==='actors')return db.getActors();if(key==='actorKeys')return db.getActorKeys();if(key==='numbersShown')return db.showSequenceNumbers();
  if(['title','accTitle','accDescr'].includes(key))return db[key]??'';return db[key];
 }
 case'property':return evaluate(e.base,models,item)[e.key];case'get':return evaluate(e.base,models,item).get(evaluate(e.key,models,item));case'has':return evaluate(e.base,models,item).has(evaluate(e.key,models,item));
 default:throw Error('Unsupported sequence expression '+e.op);
}}
function partial(a,b){if(Array.isArray(b))return Array.isArray(a)&&a.length===b.length&&b.every((v,i)=>partial(a[i],v));if(b!==null&&typeof b==='object')return a!==null&&typeof a==='object'&&Object.keys(b).every(k=>k in a&&partial(a[k],b[k]));return Object.is(a,b)}
async function execute(actions,models,item){
 for(const a of actions){
  if(a.op==='forEach'){for(const value of evaluate(a.base,models,item))await execute(a.actions,models,value);continue;}
  if(a.op==='invoke'){
   switch(a.method){
    case'create':setSiteConfig(a.config);models.set(a.id,{db:new SequenceDB(),config:a.config});break;
    case'actorView':models.views.set(a.view,models.get(a.id).db.getActors());break;
    case'setViewDescription':models.views.get(a.view).get(a.actor.value).description=a.value;break;
    case'parseDiagram':{
     const model=models.get(a.id);setSiteConfig(model.config);
     const diagram=await Diagram.fromText(a.source);model.db=diagram.db;model.renderer=diagram.renderer;break;
    }
    case'render':setSiteConfig(a.config);await models.get(a.id).renderer.draw('', '', '', models.get(a.id));break;
    case'parsePublic':setSiteConfig(a.config);models.results.set(a.result,await mermaidAPI.parse(a.source));break;
    case'captureFailure':{
     let error;
     try{await execute(a.actions,models)}catch(e){error=e;}
     assert.ok(a.kind==='boolean'||a.kind==='message');
     models.results.set(a.result,a.kind==='boolean'?error!==undefined:error===undefined?'':error instanceof Error?error.message:String(error));break;
    }
    default:throw Error('Unknown sequence operation '+a.method);
   }
  }else if(a.op==='assert'){
   const actual=evaluate(a.actual,models,item),expected=evaluate(a.expected,models,item);let passed;
   switch(a.matcher){
    case'toBe':passed=Object.is(actual,expected);break;
    case'toEqual':try{assert.deepStrictEqual(actual,expected);passed=true}catch{passed=false}break;
    case'toMatchObject':passed=partial(actual,expected);break;case'toHaveLength':passed=actual.length===expected;break;case'toContain':passed=actual.includes(expected);break;
    case'toBeTruthy':passed=!!actual;break;case'toBeFalsy':passed=!actual;break;case'toBeDefined':passed=actual!==undefined;break;case'toBeUndefined':passed=actual===undefined;break;
    default:throw Error('Unsupported sequence matcher '+a.matcher);
   }
   assert.notEqual(passed,a.negated,`${a.matcher}: ${JSON.stringify(actual)} vs ${JSON.stringify(expected)}`);
  }else throw Error('Unsupported action '+a.op);
 }
}
let count=0;
for(const r of data.records){
 const models=new Map();models.views=new Map();models.results=new Map();
 try{await execute(r.setupActions,models);await execute(r.actions,models);count++;}
 catch(e){throw Error(r.name+': '+e.message)}
}
console.log(JSON.stringify({passed:count}));
