// Mermaid.js API timing in an already-running browser. IPC, load and launch stay outside timing.
import {createRequire} from 'node:module';
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {resolve,sep} from 'node:path';
import {createServer} from 'node:http';
const require=createRequire(import.meta.url),{chromium}=require(process.env.PLAYWRIGHT_MODULE??'playwright');
const [mode,input,output]=process.argv.slice(2),cases=JSON.parse(readFileSync(input,'utf8'));
const dist=resolve('.research/pele/website/node_modules/mermaid/dist');
const html=`<!doctype html><meta charset="utf-8"><style>body{font:16px Arial;background:#1c1b1a}#host{width:1440px}</style><div id="host"></div><script type="module">
import mermaid from '/mermaid/mermaid.esm.min.mjs';
mermaid.initialize({startOnLoad:false,securityLevel:'strict',suppressErrorRendering:true,theme:'dark',fontFamily:'Arial',themeVariables:{fontFamily:'Arial',fontSize:'16px'}});
const host=document.getElementById('host');let sequence=0;
async function draw(source){host.replaceChildren();try{return(await mermaid.render('bench'+sequence++,source,host)).svg}finally{host.replaceChildren()}}
window.verify=async inputs=>{const rows=[];for(const i of inputs){try{
 const rawSvg=await draw(i.source),container=document.createElement('div');container.innerHTML=rawSvg;
 const svg=container.querySelector('svg');if(!svg)throw Error('No SVG returned');
 host.append(container);const rect=svg.getBoundingClientRect();
 rows.push({id:i.id,ok:true,svg:new XMLSerializer().serializeToString(svg),viewportWidth:rect.width,viewportHeight:rect.height});
 host.replaceChildren();
}catch(e){rows.push({id:i.id,ok:false,error:String(e.message??e)})}}return rows};
window.measure=async inputs=>{
 const run=async()=>{let bytes=0;for(const i of inputs)bytes+=(await draw(i.source)).length;return bytes};
 for(let i=0;i<5;i++)await run();let repeats=1;
 for(;;){const start=performance.now();for(let i=0;i<repeats;i++)await run();if(performance.now()-start>=10||repeats>=131072)break;repeats*=2}
 const samples=[];let elapsed=0;
 while(elapsed<300||samples.length<9){const start=performance.now();for(let i=0;i<repeats;i++)await run();const ms=performance.now()-start;elapsed+=ms;samples.push(ms/repeats)}
 samples.sort((a,b)=>a-b);return{ms:samples[Math.floor((samples.length-1)/2)],samples,repeats,inputs:inputs.length};
};
await document.fonts.ready;window.ready=true;
</script>`;
const server=createServer((req,res)=>{
 const pathname=new URL(req.url,'http://local.invalid').pathname;
 if(pathname==='/'){res.setHeader('content-type','text/html');return res.end(html)}
 if(pathname.startsWith('/mermaid/')){
  const path=resolve(dist,decodeURIComponent(pathname.slice(9)));
  if(path.startsWith(dist+sep)&&existsSync(path)){res.setHeader('content-type','text/javascript');return res.end(readFileSync(path))}
 }
 res.writeHead(404).end();
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const origin='http://127.0.0.1:'+server.address().port,browser=await chromium.launch({headless:true});
const context=await browser.newContext({viewport:{width:1440,height:1000},timezoneId:'UTC'});
await context.addInitScript(()=>{const NativeDate=Date;window.Date=class extends NativeDate{constructor(...args){super(...(args.length?args:[1792368000000]))}static now(){return 1792368000000}}});
await context.route('**/*',route=>route.request().url().startsWith(origin+'/')||route.request().url().startsWith('data:')?route.continue():route.abort());
const page=await context.newPage(),rows=[];
try{
 await page.goto(origin);await page.waitForFunction(()=>window.ready);
 for(const c of cases){
  const value=await page.evaluate(async({mode,inputs})=>mode==='verify'?await window.verify(inputs):await window.measure(inputs),{mode,inputs:c.inputs});
  rows.push(mode==='verify'?{name:c.name,results:value}:{name:c.name,timing:value});
  console.log('mermaid',mode,c.name);
 }
 writeFileSync(output,JSON.stringify({engine:'mermaid',browser:browser.version(),cases:rows}));
}finally{await browser.close();await new Promise(r=>server.close(r))}
