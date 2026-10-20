import {readFileSync,writeFileSync} from 'node:fs';
import {render} from '../../.research/pele/dist/pele.js';
const [mode,input,output,consume='api']=process.argv.slice(2),cases=JSON.parse(readFileSync(input,'utf8'));
if(!['api','utf8'].includes(consume))throw Error('Output consumption must be api or utf8');
const options={now:1792368000000,fontSize:16,fontFamily:'Arial'};
function measure(inputs){
 const run=consume==='api'
  ?()=>{let bytes=0;for(const i of inputs)bytes+=render(i.source,options).svg.length;return bytes}
  :()=>{let bytes=0;for(const i of inputs)bytes+=Buffer.from(render(i.source,options).svg,'utf8').length;return bytes};
 for(let i=0;i<5;i++)run();let repeats=1;
 for(;;){const start=performance.now();for(let i=0;i<repeats;i++)run();if(performance.now()-start>=10||repeats>=131072)break;repeats*=2}
 const samples=[];let elapsed=0;
 while(elapsed<300||samples.length<9){const start=performance.now();for(let i=0;i<repeats;i++)run();const ms=performance.now()-start;elapsed+=ms;samples.push(ms/repeats)}
 samples.sort((a,b)=>a-b);return{ms:samples[Math.floor((samples.length-1)/2)],samples,repeats,inputs:inputs.length};
}
const rows=cases.map(c=>mode==='verify'?{name:c.name,results:c.inputs.map(i=>{try{return{id:i.id,ok:true,svg:render(i.source,options).svg}}catch(e){return{id:i.id,ok:false,error:String(e.message)}}})}:{name:c.name,timing:measure(c.inputs)});
writeFileSync(output,JSON.stringify({engine:'pele',consume,cases:rows}));
