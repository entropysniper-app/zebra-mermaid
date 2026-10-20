// Freeze author-style edge cases against the pinned upstream implementation.
import {readFileSync,writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {loadOracle,plain} from './oracle.mjs';
const commit=JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if(execFileSync('git',['-C','.research/pele','rev-parse','HEAD'],{encoding:'utf8'}).trim()!==commit)throw Error('Unexpected reference revision');
execFileSync('git',['-C','.research/pele','diff','--exit-code','HEAD','--','src']);
const oracle=await loadOracle();
const props=['fill','fill-opacity','stroke','stroke-width','stroke-dasharray','stroke-dashoffset','stroke-linecap','stroke-linejoin','stroke-opacity','opacity','rx','ry','color','font-size','font-family','font-weight','font-style','text-decoration','letter-spacing','word-spacing','FILL','STROKE','stroKe','unknown',''];
const values=['red','#ABC','1','0.5','bold','bolder','BOLD','600','italic','20px','20PX','200.1px','.5px','2em','', ' ',
 'red!important',' 700 !IMPORTANT !important ','u!importantrl(#x)','url(#x)','uRL\u3000(#x)','eXpReSsIoN\t(1)',
 'javascript:foo','@IMPORT x','image-set(x)','/* x */','red;fill:blue','<red>','red\\x','"A&B"','\u0000red','var(--x)','😀','NaN'];
const inputs=[[],['fill:red','stroke:blue','fill:green','color:black','opacity:.5','color:white'],
 ['stroke:red','stroke:url(x)','stroke:blue'],['font-size:20px','font-size:invalid','font-size:1.5px']];
for(const p of props)for(const value of values)inputs.push([`${p}:${value}`]);
let seed=0x57a1e;
const random=()=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed};
for(let i=0;i<2048;i++){
 const declarations=[];const length=1+random()%24;
 for(let j=0;j<length;j++)declarations.push(`${props[random()%props.length]}:${values[random()%values.length]}`);
 inputs.push(declarations);
}
const cases=inputs.map(declarations=>({declarations,expected:plain(oracle.resolveStyle(declarations))}));
writeFileSync('tests/fixtures/style-regression.json.gz',gzipSync(JSON.stringify({commit,seed:'0x57a1e',randomCases:2048,cases}),{level:9}));
console.log(JSON.stringify({cases:cases.length,commit}));
