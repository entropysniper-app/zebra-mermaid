#!/usr/bin/env node
// Capture one unmodified sample per type for reliable Markdown previews, including HTML labels.
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createRequire} from 'node:module';
const root=fileURLToPath(new URL('../..',import.meta.url)),dir=resolve(root,'docs/render-comparison');
const require=createRequire(import.meta.url),{chromium}=require(process.env.PLAYWRIGHT_MODULE??'playwright');
const report=JSON.parse(readFileSync(resolve(dir,'results.json'),'utf8'));
const selected=report.results.filter((r,i,all)=>all.findIndex(x=>x.type===r.type)===i);
mkdirSync(resolve(dir,'previews'),{recursive:true});
const browser=await chromium.launch({headless:true});
try{
 const page=await browser.newPage({viewport:{width:760,height:480},deviceScaleFactor:2});
 await page.goto(pathToFileURL(resolve(dir,'index.html')).href);
 await page.setContent('<!doctype html><html><meta charset="utf-8"><style>body{margin:0;background:white}#frame{box-sizing:border-box;width:760px;height:480px;padding:20px;display:flex;align-items:center;justify-content:center;background:white}img{max-width:100%;max-height:100%;object-fit:contain}</style><div id="frame"><img id="diagram" alt="Rendered sample"></div></html>');
 await page.evaluate(bg=>{document.body.style.backgroundColor=bg;document.getElementById('frame').style.backgroundColor=bg},report.metadata.settings.playgroundCss.background);
 for(const row of selected)for(const engine of ['rust','pele','mermaid']){
  const value=row[engine];if(!value.ok)continue;
  await page.evaluate(async url=>{const image=document.getElementById('diagram');image.src=url;await image.decode();},pathToFileURL(resolve(dir,value.svg)).href);
  await page.locator('#frame').screenshot({path:resolve(dir,`previews/${row.id}.${engine}.png`)});
 }
}finally{await browser.close()}
const counts=Object.fromEntries(['rust','pele','mermaid'].map(e=>[e,report.results.filter(r=>r[e].ok).length]));
let section=`<!-- sample-comparison:start -->
## Rendering comparison

The same **${report.results.length} samples across ${report.metadata.selection.groups} diagram types** rendered by three engines. Each source is preserved unchanged.

| Renderer | Successful samples |
| --- | ---: |
| zebra-mermaid | **${counts.rust}/${report.results.length}** |
| Pele | **${counts.pele}/${report.results.length}** |
| Mermaid.js | **${counts.mermaid}/${report.results.length}** |

zebra-mermaid and Pele produce **identical SVG bytes and dimensions for ${report.results.filter(r=>r.rustPeleEqual).length}/${report.results.length} samples**. They share browser font measurements and the Playground's dark theme; parsing, layout, and SVG generation run independently. Mermaid.js uses its own layout with the dark theme.

[Browse all ${report.results.length} samples](docs/render-comparison/index.html) · [Summary and method](docs/render-comparison/summary.md) · [Sources, dimensions, and hashes](docs/render-comparison/results.json)

The table shows the first sample of each diagram type. Click a preview to open its SVG. Previews scale to fit their cells; actual dimensions are recorded in the gallery.

| Diagram / source | zebra-mermaid | Pele | Mermaid.js |
| --- | --- | --- | --- |
`;
for(const row of selected){const cell=e=>row[e].ok?`[<img src="docs/render-comparison/previews/${row.id}.${e}.png" width="240" alt="${row.id} ${e}">](docs/render-comparison/${row[e].svg})`:'Render failed';section+=`| **${row.type}**<br>[${row.id}](docs/render-comparison/${row.sourceFile}) | ${cell('rust')} | ${cell('pele')} | ${cell('mermaid')} |\n`}
section+='\n<!-- sample-comparison:end -->\n\n';
const readme=resolve(root,'README.md');let text=readFileSync(readme,'utf8');
if(text.includes('<!-- sample-comparison:start -->'))text=text.replace(/<!-- sample-comparison:start -->[\s\S]*?<!-- sample-comparison:end -->\s*/,()=>section);
else throw Error('README is missing the rendering comparison markers');
writeFileSync(readme,text);
console.log(`README updated: ${selected.length} rows, ${selected.length*3} PNG previews, ${report.results.length} complete samples`);
