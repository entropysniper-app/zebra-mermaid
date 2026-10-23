import {mkdirSync as ensureReleaseDir} from 'node:fs';
ensureReleaseDir('tests/fixtures/pele-1.9.0',{recursive:true});
// Independent regression expectations from the pinned upstream SVG root, never Rust output.
import {build} from '../../.research/pele/node_modules/esbuild/lib/main.js';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {gzipSync} from 'node:zlib';

const upstream=resolve('.research/pele');
const commit=JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if(execFileSync('git',['-C',upstream,'rev-parse','HEAD'],{encoding:'utf8'}).trim()!==commit)throw Error('Unexpected reference revision');
execFileSync('git',['-C',upstream,'diff','--exit-code','HEAD','--','src']);
mkdirSync('target/compat',{recursive:true});
const output=resolve('target/compat/svg-root-reference.mjs');
await build({entryPoints:[resolve(upstream,'src/svg/root.ts')],bundle:true,format:'esm',platform:'node',outfile:output,logLevel:'silent'});
const {svgDocument}=await import(pathToFileURL(output));
const fragments=['','<rect/>','<rect x="1" rx="var(--pele-radius,4px)"/>','<rect rx="var(--pele-radius,4px)" style="fill:red">',
 '<rectx rx="var(--pele-radius,4px)"/>','<rect rx="var(--pele-radius,4px)" rx="var(--pele-radius,4px)"/>',
 '<rect rx="var(--pele-radius,4px)" style="rx:17px"/>','<rect\nrx="var(--pele-radius,4px)"/>',
 '<text font-weight="var(--_tw)">标题 &amp; text</text>','<g style="rx:var(--_r)"/>','var(--_hw)','var(--_w)','var(--_x)',
 '<rect data-x="中文😀"/>','<rect:custom rx="var(--pele-radius,4px)">','<rect rx="var(--pele-radius,4px)"',
 '<rect data-inner="<rect/>" rx="var(--pele-radius,4px)"/>','<path d="M0,0"/>'];
const fonts=[undefined,'Arial','"A&B", serif',"'A > B', sans-serif",'bad; font', '中文, monospace'];
let seed=0x51a7;
const random=()=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed};
const cases=[];
for(let i=0;i<512;i++){
 const count=i<fragments.length?1:1+random()%24;
 let content='';for(let j=0;j<count;j++)content+=fragments[i<fragments.length?i:random()%fragments.length];
 const options={fontFamily:fonts[random()%fonts.length],fontFamilyMono:fonts[random()%fonts.length],responsive:random()%2===0,idPrefix:'root<&"'};
 const acc={accTitle:i%3===0?'Title <&"':undefined,accDescr:i%5===0?'Description #9829;':undefined};
 const size=[0,12,16,18.25][random()%4];
 cases.push({content,options,acc,size,expected:svgDocument('flowchart',641.25,480.5,size,options,acc,content)});
}
writeFileSync('tests/fixtures/pele-1.9.0/svg-root.json.gz',gzipSync(JSON.stringify({commit,seed:'0x51a7',cases}),{level:9}));
console.log(JSON.stringify({cases:cases.length,commit}));
