// Independent fixed-reference semantic maps and error precedence for ownership changes.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {gzipSync,gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {build} from '../../.research/pele/node_modules/esbuild/lib/main.js';
const upstreamCommit=JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if(execFileSync('git',['-C','.research/pele','rev-parse','HEAD'],{encoding:'utf8'}).trim()!==upstreamCommit)throw Error('Reference revision differs');
execFileSync('git',['-C','.research/pele','diff','--exit-code','HEAD','--','src']);
mkdirSync('target/compat',{recursive:true});
const outfile='target/compat/usecase-resolution-reference.mjs';
await build({stdin:{contents:"export {parse} from './.research/pele/src/index.ts';",resolveDir:process.cwd()},bundle:true,format:'esm',platform:'node',outfile});
const {parse}=await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const nullable=x=>x??null;
const element=(e,actor)=>({id:e.id,label:e.label,labelType:e.labelType,kind:actor?e.type:'',shape:actor?'':e.shape,icon:nullable(e.icon),business:e.business,stereotype:nullable(e.stereotype),parent:nullable(e.parentId),classes:e.classes,styles:e.styles});
const canonical=m=>({
 actors:[...m.actors].map(([id,e])=>[id,element(e,true)]),
 useCases:[...m.useCases].map(([id,e])=>[id,element(e,false)]),
 boundaries:[...m.systemBoundaries].map(([id,b])=>[id,{id:b.id,label:b.label,labelType:b.labelType,kind:b.type,members:b.members,classes:b.classes,styles:b.styles}]),
 relationships:m.relationships.map(e=>({id:e.id,explicitId:e.explicitId,source:e.source,target:e.target,kind:e.type,arrowType:e.arrowType,label:nullable(e.label),labelType:nullable(e.labelType),minlen:e.minlen,classes:e.classes,styles:e.styles,animate:e.animate,animation:nullable(e.animation)})),
 notes:[...m.notes].map(([id,n])=>[id,{id:n.id,target:n.target,label:n.label,labelType:n.labelType}]),
 jsonNodes:[...m.jsonNodes].map(([id,n])=>[id,{id:n.id,classes:n.classes,styles:n.styles,propertyOrder:[...n.propertyOrder]}]),
 classDefs:[...m.classDefs].map(([id,c])=>[id,c.styles]),
 direction:m.direction,title:nullable(m.title),accTitle:m.accTitle,accDescr:m.accDescr,accDescription:m.accDescription,
});
const docs=gunzipSync(readFileSync('tests/fixtures/pele-1.7.0.jsonl.gz')).toString().trim().split('\n').map(JSON.parse).filter(r=>r.id.includes('usecase-docs')).map(r=>r.source);
const sources=[...docs,
 'usecase-beta\nA1 --> U0\nactor A0@{ type: awesome, business: true }\nactor A1@{ type: awesome, business: true }\nU0',
 'usecase-beta\nactor A0@{ type: awesome, business: true }\nactor A1\nA0 ..> : include A1',
 'usecase-beta\nactor A0@{ type: awesome, business: true }\nMissing@{ type: normal }',
 'usecase-beta\nactor A0@{ type: awesome, business: true }\nstyle Missing fill:red',
 'usecase-beta\nactor A0("First")\nactor A0("Second")',
 'usecase-beta\nactor A0\nA0[Case]',
 'usecase-beta\nsystemBoundary B\n U0("Same")\nend\nsystemBoundary C\n U0("Same")\nend',
 'usecase-beta\nactor A\nU("One")\nA --> U\nA@{ type: hollow }\nclassDef hot stroke:red\nclass A,U hot\nstyle A fill:blue',
 'usecase-beta\nactor A("中😀")\nA@{ type: awesome, business: true }',
];
let state=0x4a0e14;
const next=()=>state=(Math.imul(state,1664525)+1013904223)>>>0;
const shuffle=a=>{for(let i=a.length-1;i>0;i--){const j=next()%(i+1);[a[i],a[j]]=[a[j],a[i]];}return a;};
for(let i=0;i<128;i++){
 const n=2+next()%9;
 const actors=Array.from({length:2},(_,j)=>`actor A${j}("Actor ${j}")`);
 const cases=Array.from({length:n},(_,j)=>`U${j}("Case ${j}")`);
 const links=shuffle(Array.from({length:n},(_,j)=>`A${j%2} --> U${j}`));
 const decl=shuffle([...actors,...cases]);
 const metadata=['A0@{ type: hollow, business: true }','U0@{ business: true }'];
 if(i%4===0)metadata.push('A1@{ icon: "fa:user" }');
 if(i%11===0)metadata.push('A1@{ type: awesome, business: true }');
 if(i%13===0)decl.push('U0("Conflicting label")');
 sources.push(['usecase-beta',...links.slice(0,1+next()%n),...decl,...links.slice(1),...metadata,'classDef hot stroke:red','class A0,U0 hot','class A0 hot','style U0 fill:blue'].join('\n'));
}
const cases=sources.map((source,index)=>{
 let model;
 try{model=parse(source);}
 catch(error){
  if(!error.code)throw error;
  return {index,source,expect:{ok:false,error:{code:nullable(error.code),type:nullable(error.type),mermaidType:nullable(error.mermaidType),line:error.line??0,column:error.column??0,snippet:error.snippet??'',message:error.message}}};
 }
 if(model.type!=='usecase')throw Error('Unexpected model type');
 return {index,source,expect:{ok:true,model:canonical(model)}};
});
if(cases.filter(c=>c.expect.ok).length===0)throw Error('No successful reference model');
const sha=x=>createHash('sha256').update(x).digest('hex');
const value={upstreamCommit,seed:'0x4a0e14',generatedCases:128,documentationCases:docs.length,scope:'Public ordered element/class/relationship maps, metadata and first semantic error; unchanged ordered-JSON payload arenas and statement/source-position trees are not projected here. This is supplementary coverage, not an original-test port.',sourceHashes:{db:sha(readFileSync('.research/pele/src/diagrams/usecase/db.ts')),parser:sha(readFileSync('.research/pele/src/diagrams/usecase/parser.ts'))},cases};
writeFileSync('tests/fixtures/unit/usecase-resolution-reference.json.gz',gzipSync(JSON.stringify(value),{level:9}));
console.log(JSON.stringify({cases:cases.length,successes:cases.filter(c=>c.expect.ok).length,rejections:cases.filter(c=>!c.expect.ok).length}));
