// Same full-render workload loop as examples/profile.rs, for CPU sampling and process memory.
import {readFileSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
process.env.TZ='UTC';
const root=fileURLToPath(new URL('../..',import.meta.url));
const {render}=await import(pathToFileURL(resolve(root,'.research/pele/dist/pele.js')));
const cases=JSON.parse(readFileSync(resolve(root,process.env.ZM_PROFILE_CASES??'target/bench/cases.json'),'utf8'));
const name=process.argv[2],seconds=Number(process.argv[3]??2);
const c=cases.find(c=>c.name===name);if(!c)throw Error('Unknown workload');
const options={now:1792368000000,...(c.unlimitedInput?{limit:Infinity}:{})};
function batch(){let rendered=0,rejected=0,bytes=0;for(const source of c.inputs){try{bytes+=Buffer.byteLength(render(source,options).svg);rendered++}catch{rejected++}}return[rendered,rejected,bytes]}
const expected=batch();for(let i=0;i<4;i++)if(String(batch())!==String(expected))throw Error('unstable result');
const started=performance.now();let batches=0;
while(performance.now()-started<seconds*1000){if(String(batch())!==String(expected))throw Error('unstable result');batches++}
console.log(JSON.stringify({engine:'pele',case:name,batches,elapsedMs:performance.now()-started,
 renderedPerBatch:expected[0],rejectedPerBatch:expected[1],svgBytesPerBatch:expected[2]}));
