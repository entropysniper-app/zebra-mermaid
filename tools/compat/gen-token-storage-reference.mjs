import {mkdirSync as ensureReleaseDir} from 'node:fs';
ensureReleaseDir('tests/fixtures/pele-1.9.0/unit',{recursive:true});
// Independent pinned-Pele token streams and errors for the internal/public storage paths.
import {readFileSync, writeFileSync, mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {gzipSync, gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {build} from '../../.research/pele/node_modules/esbuild/lib/main.js';

const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], {encoding:'utf8'}).trim() !== upstreamCommit) throw Error('Unexpected reference revision');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
const specs = [
  ['info', 'info/parser', 'INFO'], ['architecture', 'architecture/parser', 'ARCHITECTURE'],
  ['cynefin', 'cynefin/parser', 'CYNEFIN'], ['eventmodeling', 'eventmodeling/parser', 'EVENTMODELING'],
  ['gitGraph', 'git/parser', 'GIT'], ['packet', 'packet/parser', 'PACKET'],
  ['pie', 'pie/parser', 'PIE'], ['radar', 'radar/parser', 'RADAR'],
  ['treemap', 'treemap/parser', 'TREEMAP'], ['treeView', 'treeview/parser', 'TREEVIEW'],
  ['wardley', 'wardley/parser', 'WARDLEY'], ['railroad-ir', 'railroad/ir', 'IR'],
  ['railroad-ebnf', 'railroad/ebnf', 'EBNF'], ['railroad-abnf', 'railroad/abnf', 'ABNF'],
  ['railroad-peg', 'railroad/peg', 'PEG'],
];
mkdirSync('target/compat', {recursive:true});
const outfile='target/compat/token-storage-reference.mjs';
const imports=specs.map(([, module, name]) => `export {${name}_TOKENS} from './.research/pele/src/diagrams/${module}.ts';`).join('\n');
await build({stdin:{contents:`${imports}
export {tokenize, Reader} from './.research/pele/src/diagrams/common/tokens.ts';
export {lexRadar} from './.research/pele/src/diagrams/radar/parser.ts';
`,resolveDir:process.cwd()},bundle:true,format:'esm',platform:'node',outfile});
const oracle=await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const fixturePath='tests/fixtures/pele-1.9.0.jsonl.gz';
const docs=gunzipSync(readFileSync(fixturePath)).toString().trim().split('\n').map(JSON.parse);
const common=['', ' ', '\t\r\n', '\u2028\u2029', '\u00a0\ufeff', '\u0000', '😀', '中文', 'é e\u0301',
  'title AV 😀\r\n', 'accTitle: AV\n', 'accDescr: 中😀 %% comment\n', 'accDescr {a\n b}\n',
  'accDescr { a\naccDescr { b\n', '---\ntitle: x\n---\n', '---\nunclosed\n', '%%{init: {}}%%\n',
  '%%{unclosed\n', '%% line\r\n', '"😀中\\nAV"', "'a\\t b'", '"unclosed\\',
  '0'.repeat(4096), '0'.repeat(4096)+'.1.2', '12.34.56 -1.2.3 05 1. 0. -0.\n',
  ('accDescr {\n').repeat(128), ('title\t😀\n').repeat(32)];
let state=0x70c3a9;
const next=()=>state=(Math.imul(state,1664525)+1013904223)>>>0;
const insert=['\n','\r\n','\t','\u00a0','😀','中文','"','\\','%%','accDescr {','0.0','@','[',']','{','}'];
const failure=error=>({ok:false,code:error.code,type:error.type,line:error.line,column:error.column,snippet:error.snippet,message:error.message});
const tables=[];
for(const [name,module,prefix] of specs){
  const type=name.startsWith('railroad-')?'railroad':name;
  const table=oracle[`${prefix}_TOKENS`];
  const docType=type==='gitGraph'?'git':type.toLowerCase();
  const sources=docs.filter(row=>row.id.split('-docs.json')[0]===docType).map(row=>row.source);
  if(sources.length===0)throw Error(`No documentation sources for ${name}`);
  // Mixed notation is intentional: this is a lexer test, including rejection at its own boundary.
  const base=[...sources,...common];
  const cases=base.map((source,index)=>({id:`base-${index}`,source}));
  for(let i=0;i<96;i++){
    const chars=Array.from(base[next()%base.length]);
    const at=next()%(chars.length+1),operation=next()%3;
    if(operation===0)chars.splice(at,0,insert[next()%insert.length]);
    else if(operation===1)chars.splice(at,1);
    else chars.splice(at,1,insert[next()%insert.length]);
    cases.push({id:`mutation-${i}`,source:chars.join('')});
  }
  for(const row of cases){
    try{
      const result=name==='radar'?oracle.lexRadar(row.source):oracle.tokenize(row.source,table,type);
      row.expect={ok:true,tokens:result.kinds.map((kind,i)=>[kind,result.starts[i],result.ends[i],row.source.slice(result.starts[i],result.ends[i])])};
      row.expect.readerErrors=[];
      for(const i of new Set([0,Math.floor(result.kinds.length/2),result.kinds.length-1])){
        const reader=new oracle.Reader(result,table,type);reader.i=i;
        try{reader.fail('token probe');throw Error('Reader.fail did not throw');}
        catch(error){row.expect.readerErrors.push({index:i,error:failure(error)});}
      }
    }catch(error){
      row.expect=failure(error);
    }
  }
  tables.push({name,type,documentationSources:sources.length,tokenNames:table.map(t=>t.name),cases});
}
const count=tables.reduce((n,t)=>n+t.cases.length,0);
const value={upstreamCommit,seed:'0x70c3a9',mutationsPerTable:96,referenceFixtureSha256:createHash('sha256').update(readFileSync(fixturePath)).digest('hex'),tables,count};
writeFileSync('tests/fixtures/pele-1.9.0/unit/token-storage-reference.json.gz',gzipSync(JSON.stringify(value),{level:9}));
console.log(JSON.stringify({tables:tables.length,cases:count,perTable:tables.map(t=>[t.name,t.cases.length])}));
