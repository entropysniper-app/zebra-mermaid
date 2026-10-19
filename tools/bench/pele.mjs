// Same inputs, calibration, sampling and options as examples/bench.rs.
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
process.env.TZ='UTC';
const root=fileURLToPath(new URL('../..',import.meta.url));
const {render,parse}=await import(pathToFileURL(resolve(root,'.research/pele/dist/pele.js')));
const cases=JSON.parse(readFileSync(resolve(root,'target/bench/cases.json'),'utf8'));
const options={now:1792368000000};
const hash=s=>{let h=0xcbf29ce484222325n;for(const b of Buffer.from(s))h=BigInt.asUintN(64,(h^BigInt(b))*0x100000001b3n);return h.toString(16).padStart(16,'0');};
function time(fn,inputs){
 const run=()=>{let done=0;for(const source of inputs){try{fn(source);done++;}catch{}}return done;};
 const done=run();for(let i=0;i<4;i++)run();let repeats=1;
 for(;;){const start=performance.now();for(let i=0;i<repeats;i++)run();if(performance.now()-start>=10||repeats>=131072)break;repeats*=2;}
 const budget=Number(process.env.BENCH_MS??300),count=Number(process.env.BENCH_SAMPLES??9),samples=[];let elapsed=0;
 while(elapsed<budget||samples.length<count){const start=performance.now();for(let i=0;i<repeats;i++)if(run()!==done)throw Error('success count changed between samples');const ms=performance.now()-start;elapsed+=ms;samples.push(ms/repeats);}
 samples.sort((a,b)=>a-b);const q=p=>samples[Math.floor((samples.length-1)*p)];return{ms:q(.5),p10:q(.1),p90:q(.9),done,samples,repeats};
}
const out=[];
for(const c of cases){
 const verification=c.inputs.map(s=>{try{const r=render(s,options);return{ok:true,svgHash:hash(r.svg),width:r.width,height:r.height};}catch(e){return{ok:false,code:e.code,message:e.message,line:e.line??0,column:e.column??0};}});
 const valid=c.inputs.filter((_,i)=>verification[i].ok),invalid=c.inputs.filter((_,i)=>!verification[i].ok);
 const p=valid.length?time(s=>parse(s),valid):null,r=valid.length?time(s=>render(s,options),valid):null;
 const rejection=invalid.length?time(s=>{let rejected=false;try{render(s,options)}catch{rejected=true}if(!rejected)throw Error('expected rejection')},invalid):null;
 out.push({name:c.name,diagrams:c.inputs.length,verification,renderInputs:valid.length,rejectInputs:invalid.length,parse:p,render:r,rejection});
}
writeFileSync(process.argv[2]??resolve(root,'target/bench/pele.json'),JSON.stringify({engine:'pele',node:process.version,cases:out}));
