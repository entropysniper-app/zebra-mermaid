// The unchanged 17 original outline regression inputs, replayed only by pinned Pele.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {loadOracle,answer} from './oracle.mjs';
const {commit}=JSON.parse(readFileSync('tests/upstream/inventory.json'));
if(execFileSync('git',['-C','.research/pele','rev-parse','HEAD'],{encoding:'utf8'}).trim()!==commit)throw Error('Unexpected reference revision');
execFileSync('git',['-C','.research/pele','diff','--exit-code','HEAD','--','src']);
const {version}=JSON.parse(readFileSync('.research/pele/package.json'));
const sources=[
 'mindmap\n','mindmap a\n','mindmap\n a\n  :::big\r\n','kanban\n',
 'mindmap\n  a\n b\n','mindmap\n a\n  b[x\n','mindmap\n a\n  b{x}\n',
 'mindmap\n a\n  b(x) c\n','mindmap\n a\n  ::icon(x\n','mindmap\n 中文😀\n  b{x}\n',
 'mindmap\n a\n  b["`x\n','kanban\n a\n  b@{ shape: Rect }\n','kanban\n a\n  b@{ label: [ }\n',
 'kanban\n  a\n b\n c\n','kanban\n a\n  b@{ x^ }\n','kanban\n a\n  b@{ label: x } y\n',
 'kanban\n a\n  b@{ label: "x\n\n  y" }\n  c@{ a: 1\n',
];
const oracle=await loadOracle();
const rows=sources.map(source=>{const result=answer(oracle,{op:'render',source});return{source,expect:result.ok?null:result.error}});
const directory=`tests/fixtures/pele-${version}/unit`;mkdirSync(directory,{recursive:true});
writeFileSync(`${directory}/outline-errors.jsonl`,rows.map(row=>JSON.stringify(row)).join('\n')+'\n');
console.log(JSON.stringify({version,cases:rows.length,errors:rows.filter(r=>r.expect!==null).length}));
