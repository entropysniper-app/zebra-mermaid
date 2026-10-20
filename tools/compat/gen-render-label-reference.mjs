// Independent pinned-reference output and measurement traces for both label storage paths.
import {readFileSync, writeFileSync, mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {gzipSync} from 'node:zlib';
import {dirname} from 'node:path';
import {createHash} from 'node:crypto';
import {build} from '../../.research/pele/node_modules/esbuild/lib/main.js';
const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], {encoding:'utf8'}).trim() !== upstreamCommit) throw Error('Unexpected reference revision');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
mkdirSync('target/compat', {recursive:true});
const outfile='target/compat/render-label-reference.mjs';
await build({stdin:{contents:`
export {layoutLabel} from './.research/pele/src/text/label.ts';
export {metricsMeasurer} from './.research/pele/src/text/measurer.ts';
export {withIcons} from './.research/pele/src/text/icons.ts';
export {restyle} from './.research/pele/src/diagrams/requirement/render.ts';
export {labelSvg} from './.research/pele/src/svg/builder.ts';
`,resolveDir:process.cwd()},bundle:true,format:'esm',platform:'node',outfile,plugins:[{name:'expose-pinned-internal-restyle',setup(builder){builder.onLoad({filter:/[\\/]requirement[\\/]render\.ts$/},args=>({contents:readFileSync(args.path,'utf8')+'\nexport {restyle};\n',loader:'ts',resolveDir:dirname(args.path)}));}}]});
const o=await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const view=new DataView(new ArrayBuffer(8));
const bits=value=>{view.setFloat64(0,value);return view.getBigUint64(0).toString(16).padStart(16,'0');};
const fromBits=value=>{view.setBigUint64(0,BigInt(`0x${value}`));return view.getFloat64(0);};
const freeze=value=>{
 if(typeof value==='number')return {$bits:bits(value)};
 if(Array.isArray(value))return value.map(freeze);
 if(value&&typeof value==='object')return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,freeze(v)]));
 return value;
};
const sources=[undefined,'','   ','plain','AV AV AV','a\tb\u3000c','中文（测试）。好','😀e\u0301AV😀','one\ntwo\n','a<br/>b\\nc','<b>bold</b> and <i>it</i>','**bold** _it_ snake_case','<b></b>','&lt;&amp;#169;','\u0000<&"\u007f','fa:fa-car drive fa:fa-missing','<b>AV</b> fa:fa-car **中😀**','AV '.repeat(32)];
const resolver=name=>name==='fa:fa-car'?'<path d="M1 2"/>':null;
const cases=[];
for(const source of sources)for(const markdown of [false,true])for(const size of [0,-0,0.1,16,23.5,-0.1,-1.25])for(const max of [0,40,Infinity])for(const base of [0,1,4,255])for(const mode of ['metrics','custom']){
 const trace=[];
 const measurer={width(text,fontSize,style){trace.push([text,bits(fontSize),style]);const w=o.metricsMeasurer.width(text,fontSize,style);return mode==='custom'?w-(text.includes('AV')?fontSize*0.75:0):w;}};
 o.withIcons(resolver,()=>{
  const label=o.layoutLabel(source,markdown,measurer,size,max,base);
  const expect=freeze(label),layoutTrace=trace.slice(),svg=o.labelSvg(label,-12.345,0.1,' class="test"');
  const restyled=[];
  for(const styleBits of [0,1,3]){
   trace.length=0;
   const styled=o.restyle(label,styleBits,measurer);
   restyled.push({bits:styleBits,expect:freeze(styled),trace:trace.slice(),svg:o.labelSvg(styled,-12.345,0.1,' class="test"'),svgWithIcons:o.labelSvg(styled,-12.345,0.1,' class="test"',resolver)});
  }
  cases.push({source,markdown,size:bits(size),max:bits(max),base,mode,expect,trace:layoutTrace,svg,svgWithIcons:o.labelSvg(label,-12.345,0.1,' class="test"',resolver),restyled});
 });
}
const roundValues=[0,-0,0.1,-0.1,0.5,-0.5,0.49999999999999994,-0.49999999999999994,Number.MIN_VALUE,-Number.MIN_VALUE,Infinity,-Infinity,NaN];
let seed=0x18c0de;
for(let i=0;i<4096;i++){
 const next=()=>seed=(Math.imul(seed,1664525)+1013904223)>>>0;
 const hex=next().toString(16).padStart(8,'0')+next().toString(16).padStart(8,'0');
 const value=fromBits(hex);if(Number.isFinite(value))roundValues.push(value);
}
const round=roundValues.map(value=>[bits(value),bits(Math.round(value))]);
writeFileSync('tests/fixtures/unit/render-label-reference.json.gz',gzipSync(JSON.stringify({upstreamCommit,seed:'0x18c0de',absentSource:'JavaScript undefined corresponds to Rust None',restyleSourceSha256:createHash('sha256').update(readFileSync('.research/pele/src/diagrams/requirement/render.ts')).digest('hex'),cases,round}),{level:9}));
console.log(JSON.stringify({labels:cases.length,restyle:cases.reduce((n,c)=>n+c.restyled.length,0),round:round.length}));
