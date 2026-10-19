#!/usr/bin/env node
// Exact Pele Playground samples, with its default dark CSS and browser font measurements.
import {readFileSync,writeFileSync,readdirSync,mkdirSync,existsSync} from 'node:fs';
import {resolve,join,extname,sep,dirname} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {spawnSync,execFileSync} from 'node:child_process';
import {createServer} from 'node:http';
const root=fileURLToPath(new URL('../..',import.meta.url));
const up=resolve(root,'.research/pele'),scratch=resolve(root,'target/sample-comparison'),output=resolve(root,'docs/render-comparison');
for(const d of [scratch,output,join(output,'svg'),join(output,'sources')])mkdirSync(d,{recursive:true});
const require=createRequire(import.meta.url),upRequire=createRequire(join(up,'package.json')),siteRequire=createRequire(join(up,'website/package.json'));
const {chromium}=require(process.env.PLAYWRIGHT_MODULE??'playwright');
const commit=execFileSync('git',['rev-parse','HEAD'],{cwd:up,encoding:'utf8'}).trim();
if(commit!=='44da152bbb79b91f03e68016f42499bb25bf6e82')throw Error('Unexpected Pele revision');
const mermaidDist=resolve(up,'website/node_modules/mermaid/dist'),mermaidVersion=JSON.parse(readFileSync(join(mermaidDist,'../package.json'),'utf8')).version;
if(mermaidVersion!=='12.1.0')throw Error('Expected the pinned Mermaid 12.1.0');
const fontRoot=dirname(siteRequire.resolve('@fontsource-variable/inter/wght.css'));
const sourceFiles=['website/src/styles/base.scss','website/src/styles/pele-theme.scss','website/src/styles/playground.scss','website/src/scripts/playground-mermaid.ts','website/src/scripts/playground.ts','website/src/lib/icons.ts'];
const css=siteRequire('sass').compileString('@use "base"; @use "pele-theme"; @use "playground";',{loadPaths:[join(up,'website/src/styles')],style:'expanded'}).css;
writeFileSync(join(output,'playground.css'),css);
const esbuild=upRequire('esbuild');
await esbuild.build({stdin:{contents:"export {sampleGroups} from './website/src/lib/examples.ts';",resolveDir:up,sourcefile:'sample-selection.ts'},bundle:true,format:'esm',platform:'node',outfile:join(scratch,'selection.mjs'),logLevel:'silent'});
await esbuild.build({stdin:{contents:"export {render} from './src/index.ts'; export {defaultMeasurer} from './src/text/measurer.ts'; export {siteIcon} from './website/src/lib/icons.ts';",resolveDir:up,sourcefile:'playground-host.ts'},bundle:true,format:'esm',platform:'browser',target:'es2021',outfile:join(scratch,'pele-host.mjs'),logLevel:'silent'});
const {sampleGroups}=await import(pathToFileURL(join(scratch,'selection.mjs')));
const corpora=Object.fromEntries(readdirSync(join(up,'tests/corpus')).filter(f=>f.endsWith('-docs.json')).sort().map(f=>['../../../tests/corpus/'+f,JSON.parse(readFileSync(join(up,'tests/corpus',f),'utf8'))]));
const groups=sampleGroups(corpora),samples=groups.flatMap(g=>g.samples.map((s,i)=>({id:`${g.type}-${String(i+1).padStart(2,'0')}`,type:g.type,beta:g.beta,label:s.label,source:s.source})));
const now=1792368000000,sha=x=>createHash('sha256').update(x).digest('hex');
const binary=resolve(root,process.env.CARGO_TARGET_DIR??'target','release/examples/playground_sample');
if(!existsSync(binary))throw Error('Build the playground_sample release example first');
const cleanError=e=>String(e?.message??e).replaceAll(root,'<repository>');
const driver=`<!doctype html><html><meta charset="utf-8"><link rel="stylesheet" href="/playground.css"><link rel="stylesheet" href="/fonts/wght.css"><link rel="stylesheet" href="/fonts/wght-italic.css"><style>body{margin:0;width:760px}.playground-preview{display:block;width:760px}.playground-diagram-canvas{width:760px;min-height:150px}</style><div class="playground-preview" data-pele-theme="dark"><div class="playground-diagram-canvas" id="canvas"></div></div><script type="module">
import mermaid from '/mermaid/mermaid.esm.min.mjs';
import {render,defaultMeasurer,siteIcon} from '/pele-host.mjs';
const canvas=document.getElementById('canvas');await document.fonts.ready;
const style=getComputedStyle(canvas),vars={};for(const key of style)if(key.startsWith('--pele-'))vars[key]=style.getPropertyValue(key).trim();
const fontFamily=vars['--pele-font']||style.fontFamily,fontFamilyMono=vars['--pele-font-mono'];
const maxWidth=canvas.clientWidth-parseFloat(style.paddingLeft)-parseFloat(style.paddingRight);
const measurer=defaultMeasurer(fontFamily,fontFamilyMono);
window.host={vars,fontFamily,fontFamilyMono,maxWidth,canvasWidth:canvas.clientWidth,background:vars['--pele-bg'],textRendering:style.textRendering,fontSmoothing:style.webkitFontSmoothing};
window.measureRequests=requests=>requests.map(r=>({...r,width:measurer.width(r.text,r.size,r.style)}));
window.peleSample=({source,idPrefix,now})=>{const widths=new Map(),icons={};try{const result=render(source,{idPrefix,now,fontFamily,fontFamilyMono,maxWidth,measurer:{width(text,size,style){const width=measurer.width(text,size,style);widths.set(JSON.stringify([text,size,style]),{text,size,style,width});return width}},icons(name){const icon=siteIcon(name);icons[name]=icon??null;return icon}});return{ok:true,...result,measurements:[...widths.values()].filter(m=>m.text.isWellFormed()),icons}}catch(e){return{ok:false,error:{name:e.name,message:String(e.message),code:e.code}}}};
window.decorate=async({svg:rawSvg,engine,type})=>{
 canvas.replaceChildren();const preview=document.createElement('div');canvas.append(preview);preview.innerHTML=rawSvg;
 const svg=preview.querySelector('svg');if(!svg)throw Error('Renderer returned no SVG element');
 const rawDoc=new DOMParser().parseFromString(rawSvg,'image/svg+xml');
 if(engine!=='mermaid'){for(const[k,v]of Object.entries(vars))svg.style.setProperty(k,v);svg.setAttribute('data-pele-theme','dark');}
 svg.style.backgroundColor=window.host.background;svg.style.textRendering=window.host.textRendering;svg.style.webkitFontSmoothing=window.host.fontSmoothing;
 await document.fonts.ready;const vb=svg.getAttribute('viewBox')?.trim().split(/[ ,]+/).map(Number),box=svg.getBoundingClientRect();
 const result={ok:true,svg:new XMLSerializer().serializeToString(svg),rawSvg,rawXmlValid:!rawDoc.querySelector('parsererror'),type,width:vb?.[2]??box.width,height:vb?.[3]??box.height,dimensionsFrom:vb?'viewBox':'browser viewport'};
 canvas.replaceChildren();return result;
};
window.draw=async({source,id})=>{canvas.replaceChildren();mermaid.initialize({startOnLoad:false,securityLevel:'strict',suppressErrorRendering:true,theme:'dark'});try{const result=await mermaid.render(id,source);return await window.decorate({svg:result.svg,engine:'mermaid',type:result.diagramType})}catch(e){return{ok:false,error:{name:e.name,message:String(e.message)}}}finally{canvas.replaceChildren()}};
window.ready=true;
</script></html>`;
const server=createServer((req,res)=>{
 const url=new URL(req.url,'http://localhost').pathname;
 if(url==='/'){res.setHeader('content-type','text/html;charset=utf-8');res.end(driver);return}
 if(url==='/playground.css'){res.setHeader('content-type','text/css');res.end(css);return}
 if(url==='/pele-host.mjs'){res.setHeader('content-type','text/javascript');res.end(readFileSync(join(scratch,'pele-host.mjs')));return}
 for(const[prefix,dir]of[['/mermaid/',mermaidDist],['/fonts/',fontRoot]])if(url.startsWith(prefix)){
  const path=resolve(dir,decodeURIComponent(url.slice(prefix.length)));
  if(path.startsWith(dir+sep)&&existsSync(path)){res.setHeader('content-type',({'.css':'text/css','.woff2':'font/woff2','.woff':'font/woff','.map':'application/json'})[extname(path)]??'text/javascript');res.end(readFileSync(path));return}
 }
 res.writeHead(404).end();
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({headless:true}),context=await browser.newContext({viewport:{width:1440,height:1000},timezoneId:'UTC',deviceScaleFactor:1});
await context.addInitScript(({now})=>{const NativeDate=Date;class FixedDate extends NativeDate{constructor(...args){super(...(args.length?args:[now]))}static now(){return now}}window.Date=FixedDate;},{now});
let blocked=new Set(),warnings=[];
await context.route('**/*',route=>{const url=route.request().url();if(url.startsWith(origin+'/')||url.startsWith('data:')||url.startsWith('blob:'))return route.continue();blocked.add(url);return route.abort()});
let page;async function resetPage(){if(page)await page.close();page=await context.newPage();page.on('console',m=>{if(['warning','error'].includes(m.type()))warnings.push(m.text().replaceAll(origin,'<render-origin>'))});await page.goto(origin);await page.waitForFunction(()=>window.ready===true)}
await resetPage();const host=await page.evaluate(()=>window.host);writeFileSync(join(output,'playground-style.json'),JSON.stringify({theme:'dark',mermaidTheme:'dark',...host,sourceFiles:sourceFiles.map(path=>({path,sha256:sha(readFileSync(join(up,path)))}))},null,2)+'\n');
const results=[];
function save(r,engine,id){if(!r.ok)return{ok:false,error:r.error};const svg=`svg/${id}.${engine}.svg`,rawSvg=`svg/${id}.${engine}.raw.svg`;writeFileSync(join(output,svg),r.svg);writeFileSync(join(output,rawSvg),r.rawSvg);return{ok:true,type:r.type,width:r.width,height:r.height,dimensionsFrom:r.dimensionsFrom,svg,bytes:Buffer.byteLength(r.svg),sha256:sha(r.svg),rawSvg,rawBytes:Buffer.byteLength(r.rawSvg),rawSha256:sha(r.rawSvg),rawXmlValid:r.rawXmlValid,method:engine==='mermaid'?'Playground theme dark; HTML DOM insertion and XML serialization':'Playground dark CSS variables applied to unchanged raw SVG; XML serialization'}}
try{
 for(const[index,sample]of samples.entries()){
  const idPrefix=`sample-${index}-`,sourceFile=`sources/${sample.id}.mmd`;writeFileSync(join(output,sourceFile),sample.source);blocked=new Set();warnings=[];
  const originalPele=await page.evaluate(arg=>window.peleSample(arg),{source:sample.source,idPrefix,now});let rust,pele;
  if(originalPele.ok){
   const request={source:sample.source,idPrefix,now,fontFamily:host.fontFamily,fontFamilyMono:host.fontFamilyMono,maxWidth:host.maxWidth,measurements:originalPele.measurements,icons:originalPele.icons};
   for(let attempt=0;attempt<12;attempt++){
    const run=spawnSync(binary,[],{input:JSON.stringify(request),encoding:'utf8',maxBuffer:32*1024*1024,timeout:30000});
    if(run.status!==0){rust={ok:false,error:{message:cleanError(run.error??run.stderr)}};break}
    rust=JSON.parse(run.stdout);if(!rust.missingMeasurements?.length)break;
    request.measurements.push(...await page.evaluate(m=>window.measureRequests(m),rust.missingMeasurements));
    if(attempt===11)rust={ok:false,error:{message:'Host text measurement requests did not converge'}};
   }
   pele=await page.evaluate(arg=>window.decorate(arg),{svg:originalPele.svg,type:originalPele.type,engine:'pele'});
   if(rust.ok)rust=await page.evaluate(arg=>window.decorate(arg),{svg:rust.svg,type:rust.type,engine:'rust'});
  }else{pele=originalPele;rust={ok:false,error:{message:'Playground host measurement unavailable because Pele failed; Rust not evaluated in this host context'}}}
  let mermaid,timer;try{mermaid=await Promise.race([page.evaluate(arg=>window.draw(arg),{source:sample.source,id:`mermaid-${index}`}),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('Rendering exceeded 30 seconds')),30000)})])}catch(e){mermaid={ok:false,error:{name:'BrowserRenderError',message:cleanError(e)}};await resetPage()}finally{clearTimeout(timer)}
  const row={...sample,sourceFile,sourceSha256:sha(sample.source),rust:save(rust,'rust',sample.id),pele:save(pele,'pele',sample.id),mermaid:save(mermaid,'mermaid',sample.id),rustPeleEqual:rust.ok&&pele.ok&&rust.rawSvg===pele.rawSvg&&rust.svg===pele.svg&&rust.width===pele.width&&rust.height===pele.height,blockedExternalResources:[...blocked],warnings:[...new Set(warnings)]};results.push(row);
  console.log(`${index+1}/${samples.length} ${sample.id}: rust=${rust.ok} pele=${pele.ok} mermaid=${mermaid.ok} equal=${row.rustPeleEqual}`);
 }
 const metadata={createdAt:new Date().toISOString(),pele:{version:'1.6.0',commit},rust:{version:'zebra-mermaid 0.1.0',binarySha256:sha(readFileSync(binary))},mermaid:{version:mermaidVersion,browser:browser.version()},selection:{method:'Unmodified website/src/lib/examples.ts sampleGroups(), including OWN examples and original corpus selection/order',sourceSha256:sha(readFileSync(join(up,'website/src/lib/examples.ts'))),groups:groups.length,samples:samples.length},settings:{playgroundTheme:'dark',playgroundCss:host,now,timezone:'UTC',rustPeleColorMode:'css',mermaidTheme:'dark, matching playground-mermaid.ts (source frontmatter may override)',icons:'Pele/Rust: original Playground siteIcon; Mermaid: original Playground defaults without extra registration',textMeasurement:'Same real browser canvas font widths supplied through the Rust TextMeasurer host interface; parsing, layout and SVG remain independently executed',externalNetwork:'blocked and reported'}};
 writeFileSync(join(output,'results.json'),JSON.stringify({metadata,results},null,2)+'\n');
 const iconSource=readFileSync(join(up,'website/src/lib/lucide-icons.ts'),'utf8');writeFileSync(join(output,'NOTICE.md'),'# Sample 与主题来源\n\nPele 源码、CSS 与 Sample 使用根目录 THIRD_PARTY_NOTICES.md 中的许可。Playground 图标许可如下：\n\n```text\n'+iconSource.slice(0,iconSource.indexOf('export ')).replace(/^\/\/ ?/gm,'')+'\n```\n');
}finally{await browser.close();await new Promise(r=>server.close(r))}
console.log('Saved docs/render-comparison/results.json');
