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
let section=`<!-- sample-comparison:start -->\n## Sample 渲染对比\n\n使用固定 Pele 版本的 Playground \`sampleGroups()\` 原函数，完整选择 **${report.metadata.selection.groups} 类、${report.results.length} 个 Sample**，同一份源码分别由 Rust、Pele 和 Mermaid.js 渲染。\n\n| 引擎 | 版本 | 成功 / Sample 总数 |\n| --- | --- | ---: |\n| Rust / zebra-mermaid | 0.1.0 | **${counts.rust} / ${report.results.length}** |\n| Pele | 1.6.0 | **${counts.pele} / ${report.results.length}** |\n| Mermaid.js | 12.1.0，真实 Chromium 渲染 | **${counts.mermaid} / ${report.results.length}** |\n\nRust 与 Pele 的 **${report.results.filter(r=>r.rustPeleEqual).length}/${report.results.length} 份 SVG 字节及尺寸完全一致**。三者采用 Pele Playground 的默认深色显示环境：Rust/Pele 复用其 Flexoki CSS、系统字体栈、6px 圆角和 siteIcon；Mermaid 按 Playground 使用 \`theme: dark\`。Rust/Pele 共用真实浏览器宿主字体测量，Rust 仍独立执行解析、布局和 SVG 生成。Mermaid 保留自身布局，因此外观和尺寸可以不同。此结果只描述 Sample 渲染，不能代替全量原始测试 1:1 验收。\n\n- [完整 167 个 Sample 的三列可视化对照表](docs/render-comparison/index.html)：支持类型/源码筛选、图形放大、查看源码及打开 SVG。下载后直接打开 \`index.html\`；GitHub 网页通常显示 HTML 源码。\n- [33 类汇总表与复现说明](docs/render-comparison/summary.md) · [结果、尺寸与 SHA-256](docs/render-comparison/results.json) · [Playground 样式变量与来源](docs/render-comparison/playground-style.json)。\n\n下表展示每类在上游 Sample 顺序中的**第一个样例**，共 33 行。PNG 预览直接从对应 SVG 截取，点击图片打开 SVG；全部样例的源码和原始输出都保留。缩略图各自适配单元格，图形的实际尺寸见完整对照页。\n\n| 类型 / Sample 源码 | Rust | Pele | Mermaid.js |\n| --- | --- | --- | --- |\n`;
for(const row of selected){const cell=e=>row[e].ok?`[<img src="docs/render-comparison/previews/${row.id}.${e}.png" width="240" alt="${row.id} ${e}">](docs/render-comparison/${row[e].svg})`:'渲染失败';section+=`| **${row.type}**<br>[${row.id}](docs/render-comparison/${row.sourceFile}) | ${cell('rust')} | ${cell('pele')} | ${cell('mermaid')} |\n`}
section+=`\n样式直接来自上游 \`base.scss\`、\`pele-theme.scss\` 与 \`playground.scss\`；背景为 \`#1c1b1a\`，节点表面为 \`#282726\`，正文为 \`#cecdc3\`。固定画布宽 760px、扣除内边距后的布局宽 712px。时钟固定为 2026-10-19 00:00 UTC；保留样例自身的 front matter。Mermaid 有 ${report.results.filter(r=>r.mermaid.rawXmlValid===false).length} 份返回字符串含 HTML 标签，先按其支持的 DOM 插入方式呈现，再序列化为独立 SVG，原始字符串另存为 \`.mermaid.raw.svg\`。没有修改源码来强求三者视觉一致。\n<!-- sample-comparison:end -->\n\n`;
const readme=resolve(root,'README.md');let text=readFileSync(readme,'utf8');
if(text.includes('<!-- sample-comparison:start -->'))text=text.replace(/<!-- sample-comparison:start -->[\s\S]*?<!-- sample-comparison:end -->\s*/,()=>section);
else text=text.replace('## 文档',()=>section+'## 文档');
writeFileSync(readme,text);
console.log(`README updated: ${selected.length} rows, ${selected.length*3} PNG previews, ${report.results.length} complete samples`);
